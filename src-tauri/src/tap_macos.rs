//! macOS用のキーボードフック。
//!
//! rdevの`grab`は押下のたびにText Services Managerでキーの文字表記を引くが、
//! このAPIはメインスレッド必須のため、専用スレッドで動く`grab`は最初の押下で
//! 必ずSIGTRAPになる。表記は使わないので、CGEventTapを自前で張って回避する。
//! 併せて、タップがOSに無効化されたときの復帰も行う。
//!
//! タップは専用のスレッドの実行ループに置く。メインスレッドに置くと、設定画面やパネルの描画で
//! メインスレッドが忙しい間、Mac 全体のキー入力がタップの順番待ちで遅れるため。
//! コールバックは AppKit に触れない（パネルの操作は popup のスレッド経由でメインスレッドへ送る）。
//!
//! 無効化の通知を受けて即座に張り直すのはタイムアウトのときだけにする。
//! 権限の取り消しなどで切られたときに張り直すと、すぐまた切られて通知が届き、
//! メインスレッドが止まらなくなる。そちらは監視スレッドが間隔を空けて復帰させる。

use crate::input::{Machine, Verdict};
use std::{
    cell::RefCell,
    ffi::c_void,
    ptr,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
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
    fn CGEventTapIsEnabled(tap: CFRef) -> bool;
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGEventSourceKeyState(state_id: i32, key: u16) -> bool;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(allocator: CFRef, port: CFRef, order: isize) -> CFRef;
    fn CFRunLoopGetCurrent() -> CFRef;
    fn CFRunLoopRun();
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

/// キーボードの実際の状態。合成イベントの影響を受けない。
const SOURCE_STATE_HID_SYSTEM: i32 = 1;
/// macOSのキーコードの上限（kVK_*は0x7F以下）。
const MAX_KEYCODE: i64 = 0x7F;

/// タップの寿命はアプリと同じなので、タップのスレッドに置いたまま解放しない。
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

    // コールバックが遅れてOSにタップを切られたときは、その場で張り直す。
    if event_type == EVENT_TAP_DISABLED_BY_TIMEOUT {
        let tap = *context.tap.borrow();
        if !tap.is_null() {
            unsafe { CGEventTapEnable(tap, true) };
        }
        return event;
    }
    // それ以外の理由で切られたときはここでは触らず、監視スレッドに任せる。
    if event_type == EVENT_TAP_DISABLED_BY_USER_INPUT {
        return event;
    }

    let code = unsafe { CGEventGetIntegerValueField(event, FIELD_KEYCODE) };
    let Some(name) = crate::keys::key_name_from_macos_code(code) else {
        return event;
    };

    // 取りこぼした「離した」イベントの分を、実際のキーの状態で埋め合わせる。
    // いま届いたキーは、これから下の処理で押下か解放かを判断するので対象にしない。
    if matches!(
        event_type,
        EVENT_KEY_DOWN | EVENT_KEY_UP | EVENT_FLAGS_CHANGED
    ) {
        let flags = unsafe { CGEventGetFlags(event) };
        if let Ok(mut machine) = context.machine.lock() {
            machine.forget_released(|key| key == name || still_down(key, flags));
        }
    }

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

/// キーがまだ押されているか。押されていないと言い切れるときだけ偽を返す。
/// 修飾キーはイベントのフラグとキーボードの状態の両方が「離した」と言うときだけ離したと見なす。
/// 他のアプリが合成したイベントはフラグを持たないことがあり、
/// セキュア入力の最中はキーボードの状態が読めないことがあるため、片方だけでは外さない。
fn still_down(name: &str, flags: u64) -> bool {
    // CapsLockのビットはロック状態で、押しているかどうかではない。
    if name == "CapsLock" {
        return true;
    }
    if let Some(mask) = crate::keys::macos_modifier_mask(name) {
        if flags & mask != 0 {
            return true;
        }
    }
    (0..=MAX_KEYCODE)
        .filter(|&code| crate::keys::key_name_from_macos_code(code) == Some(name))
        .any(|code| unsafe { CGEventSourceKeyState(SOURCE_STATE_HID_SYSTEM, code as u16) })
}

fn with_machine(context: &TapContext, action: impl FnOnce(&mut Machine) -> Verdict) -> Verdict {
    match context.machine.lock() {
        Ok(mut machine) => action(&mut machine),
        Err(_) => Verdict::Pass,
    }
}

/// 張ったタップ。監視スレッドから状態の確認と張り直しに使う。
#[derive(Clone, Copy)]
pub struct Tap(CFRef);

// SAFETY: タップはアプリ終了まで解放せず、CGEventTapEnable/IsEnabledはスレッドを問わない。
unsafe impl Send for Tap {}

impl Tap {
    pub fn is_enabled(self) -> bool {
        unsafe { CGEventTapIsEnabled(self.0) }
    }

    pub fn enable(self) {
        unsafe { CGEventTapEnable(self.0, true) };
    }
}

#[link(name = "System")]
extern "C" {
    fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
}

/// 利用者の操作にすぐ応える仕事の優先度（QOS_CLASS_USER_INTERACTIVE）。
const QOS_CLASS_USER_INTERACTIVE: u32 = 0x21;

/// タップ専用のスレッドを立て、そこでタップを張る。成功すると以降のキーイベントがコールバックへ届く。
/// どのスレッドから呼んでもよい。
pub fn install(machine: Arc<Mutex<Machine>>) -> Result<Tap, String> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("opensesame-keys".into())
        .spawn(move || {
            unsafe { pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0) };
            let installed = install_here(machine);
            let ok = installed.is_ok();
            let _ = sender.send(installed);
            if ok {
                // このスレッドの実行ループを回し続ける（アプリが終わるまで戻らない）。
                unsafe { CFRunLoopRun() };
            }
        })
        .map_err(|error| error.to_string())?;
    receiver
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "the key thread did not respond".to_string())?
}

/// 呼んだスレッドの実行ループにタップを張る。
fn install_here(machine: Arc<Mutex<Machine>>) -> Result<Tap, String> {
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

    // 呼んだスレッド（タップ専用）の実行ループにぶら下げる。
    unsafe {
        (*context).tap.replace(tap);
        CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        CFRelease(source);
    }
    Ok(Tap(tap))
}
