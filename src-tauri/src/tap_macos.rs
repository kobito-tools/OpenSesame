//! macOS用のキーボードフック。
//!
//! rdevの`grab`は押下のたびにText Services Managerでキーの文字表記を引くが、
//! このAPIはメインスレッド必須のため、専用スレッドで動く`grab`は最初の押下で
//! 必ずSIGTRAPになる。表記は使わないので、CGEventTapを自前で張って回避する。
//! 併せて、タップがOSに無効化されたときの復帰も行う。

use crate::input::{Machine, Verdict};
use std::{
    cell::RefCell,
    ffi::c_void,
    ptr,
    sync::{Arc, Mutex},
};

type CFRef = *const c_void;
type CGEventRef = *mut c_void;
type CGEventTapProxy = *const c_void;
type CGEventTapCallBack =
    extern "C" fn(CGEventTapProxy, u32, CGEventRef, *mut c_void) -> CGEventRef;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFRef;
    fn CGEventTapEnable(tap: CFRef, enable: bool);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(allocator: CFRef, port: CFRef, order: isize) -> CFRef;
    fn CFRunLoopGetMain() -> CFRef;
    fn CFRunLoopAddSource(run_loop: CFRef, source: CFRef, mode: CFRef);
    fn CFRelease(cf: CFRef);
    static kCFRunLoopCommonModes: CFRef;
}

const TAP_HID: u32 = 0;
const TAP_HEAD_INSERT: u32 = 0;
const TAP_ACTIVE: u32 = 0;

const EVENT_KEY_DOWN: u32 = 10;
const EVENT_KEY_UP: u32 = 11;
const EVENT_FLAGS_CHANGED: u32 = 12;
const EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;
const EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFF_FFFF;

const FIELD_KEYCODE: u32 = 9;

/// タップの寿命はアプリと同じなので、メインスレッドに置いたまま解放しない。
struct TapContext {
    machine: Arc<Mutex<Machine>>,
    tap: RefCell<CFRef>,
}

extern "C" fn callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    // SAFETY: install()で確保したTapContextを指し、アプリ終了まで有効。
    let context = unsafe { &*(user_info as *const TapContext) };

    // 重い処理や無操作が続くとOSがタップを切るので、その場で張り直す。
    if event_type == EVENT_TAP_DISABLED_BY_TIMEOUT
        || event_type == EVENT_TAP_DISABLED_BY_USER_INPUT
    {
        let tap = *context.tap.borrow();
        if !tap.is_null() {
            unsafe { CGEventTapEnable(tap, true) };
        }
        return event;
    }

    let code = unsafe { CGEventGetIntegerValueField(event, FIELD_KEYCODE) };
    let Some(name) = crate::keys::key_name_from_macos_code(code) else {
        return event;
    };

    let verdict = match event_type {
        EVENT_KEY_DOWN => with_machine(context, |machine| machine.on_press(name)),
        EVENT_KEY_UP => with_machine(context, |machine| machine.on_release(name)),
        EVENT_FLAGS_CHANGED => {
            // 修飾キーは押下と解放が同じイベントで届く。
            // どちらかはデバイス固有のフラグビットが立っているかで判別する。
            let flags = unsafe { CGEventGetFlags(event) };
            let Some(mask) = crate::keys::macos_modifier_mask(name) else {
                return event;
            };
            if flags & mask != 0 {
                with_machine(context, |machine| machine.on_press(name))
            } else {
                with_machine(context, |machine| machine.on_release(name))
            }
        }
        _ => Verdict::Pass,
    };

    match verdict {
        Verdict::Swallow => ptr::null_mut(),
        Verdict::Pass => event,
    }
}

fn with_machine(context: &TapContext, action: impl FnOnce(&mut Machine) -> Verdict) -> Verdict {
    match context.machine.lock() {
        Ok(mut machine) => action(&mut machine),
        Err(_) => Verdict::Pass,
    }
}

/// メインスレッドで呼ぶこと。成功すると以降のキーイベントがコールバックへ届く。
pub fn install(machine: Arc<Mutex<Machine>>) -> Result<(), String> {
    let context = Box::into_raw(Box::new(TapContext {
        machine,
        tap: RefCell::new(ptr::null()),
    }));

    let mask = (1u64 << EVENT_KEY_DOWN) | (1u64 << EVENT_KEY_UP) | (1u64 << EVENT_FLAGS_CHANGED);
    let tap = unsafe {
        CGEventTapCreate(
            TAP_HID,
            TAP_HEAD_INSERT,
            TAP_ACTIVE,
            mask,
            callback,
            context as *mut c_void,
        )
    };
    if tap.is_null() {
        // SAFETY: 直前にinto_rawしたポインタで、まだ誰も触っていない。
        unsafe { drop(Box::from_raw(context)) };
        return Err("event tap could not be created".into());
    }

    let source = unsafe { CFMachPortCreateRunLoopSource(ptr::null(), tap, 0) };
    if source.is_null() {
        unsafe {
            CFRelease(tap);
            drop(Box::from_raw(context));
        }
        return Err("run loop source could not be created".into());
    }

    // アプリのメインループにぶら下げる。コールバックもメインスレッドで動くので、
    // ウィンドウ操作を含めてAppKitに触れても安全になる。
    unsafe {
        (*context).tap.replace(tap);
        CFRunLoopAddSource(CFRunLoopGetMain(), source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        CFRelease(source);
    }
    Ok(())
}
