//! 画面の解像度を変える。macOS は CoreGraphics の表示モードを直接使う。
//! 一覧づくりや画面・モードの探し方は OS に触れない関数に分け、テストできるようにしている。

use crate::model::{DisplayModeRef, DisplaySetting, ScreenRef};
use serde::Serialize;
use std::collections::BTreeMap;
use tauri::AppHandle;

/// OS から読んだ表示モード1つ。
#[derive(Debug, Clone, PartialEq)]
pub struct RawMode {
    pub width: u32,
    pub height: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub refresh: f64,
    /// デスクトップに使えるモードか（使えないモードは一覧に出さず、選ばない）。
    pub usable: bool,
    /// その画面の既定（推奨）のモードか。
    pub native: bool,
}

impl RawMode {
    fn same_size(&self, mode: &DisplayModeRef) -> bool {
        self.width == mode.width
            && self.height == mode.height
            && self.pixel_width == mode.pixel_width
            && self.pixel_height == mode.pixel_height
    }

    fn as_ref(&self, refresh: f64) -> DisplayModeRef {
        DisplayModeRef {
            width: self.width,
            height: self.height,
            pixel_width: self.pixel_width,
            pixel_height: self.pixel_height,
            refresh,
        }
    }
}

/// いまつながっている画面1つ。`id` はその場限りの番号。
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenInfo {
    pub id: u32,
    pub screen: ScreenRef,
}

/// 設定画面に出す選択肢1つ。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModeOption {
    #[serde(flatten)]
    pub mode: DisplayModeRef,
    pub hidpi: bool,
    pub current: bool,
    pub native: bool,
}

/// 設定画面に出す画面1つ。`modes` は見た目の解像度ごとに1つへまとめたもの、
/// `detailed` はリフレッシュレートや低解像度の版まで分けたもの。
#[derive(Debug, Clone, Serialize)]
pub struct DisplayInfo {
    pub screen: ScreenRef,
    pub main: bool,
    pub builtin: bool,
    pub current: Option<DisplayModeRef>,
    pub modes: Vec<ModeOption>,
    pub detailed: Vec<ModeOption>,
}

/// リフレッシュレートは 59.94 と 60.0 のように細かく揺れるので、小数2桁で比べる。
fn refresh_key(refresh: f64) -> i64 {
    (refresh * 100.0).round() as i64
}

fn same_refresh(a: f64, b: f64) -> bool {
    refresh_key(a) == refresh_key(b)
}

/// 大きい解像度から並べる。同じ見た目なら画素の多い（HiDPI の）方、速いリフレッシュレートを先に。
fn order(a: &DisplayModeRef, b: &DisplayModeRef) -> std::cmp::Ordering {
    (
        b.width,
        b.height,
        b.pixel_width,
        b.pixel_height,
        refresh_key(b.refresh),
    )
        .cmp(&(
            a.width,
            a.height,
            a.pixel_width,
            a.pixel_height,
            refresh_key(a.refresh),
        ))
}

fn is_current(mode: &DisplayModeRef, current: Option<&RawMode>, compare_refresh: bool) -> bool {
    current.is_some_and(|now| {
        now.same_size(mode) && (!compare_refresh || same_refresh(now.refresh, mode.refresh))
    })
}

/// 画面の縦横比。画素の最も多いモードを、その画面の本来の形とみなす。
fn panel_aspect(modes: &[RawMode]) -> Option<f64> {
    modes
        .iter()
        .filter(|mode| mode.usable && mode.pixel_height > 0)
        .max_by_key(|mode| u64::from(mode.pixel_width) * u64::from(mode.pixel_height))
        .map(|mode| f64::from(mode.pixel_width) / f64::from(mode.pixel_height))
}

/// 縦横比が画面と同じか（1.5% までの差は同じとみなす。1680×945 などの端数を拾うため）。
fn matches_aspect(mode: &RawMode, aspect: f64) -> bool {
    mode.height > 0
        && (f64::from(mode.width) / f64::from(mode.height) - aspect).abs() / aspect < 0.015
}

/// 見た目の解像度ごとに1つへまとめた一覧。同じ見た目に HiDPI と低解像度の版があれば、
/// 画素の多い方を残す。画面と縦横比の違うモード（黒帯や引き伸ばしになるもの）は詳しい一覧だけに出す。
/// リフレッシュレートは指定しない（使うときに最も高いものを選ぶ）。
pub fn summary_options(modes: &[RawMode], current: Option<&RawMode>) -> Vec<ModeOption> {
    let aspect = panel_aspect(modes);
    let fits = |mode: &RawMode| aspect.is_none_or(|aspect| matches_aspect(mode, aspect));
    // 縦横比の合うモードが1つも無いような画面では、絞らずに全部出す。
    let shaped = modes.iter().any(|mode| mode.usable && fits(mode));
    let mut best: BTreeMap<(u32, u32), &RawMode> = BTreeMap::new();
    for mode in modes
        .iter()
        .filter(|mode| mode.usable && (!shaped || fits(mode)))
    {
        let entry = best.entry((mode.width, mode.height)).or_insert(mode);
        let pixels = |m: &RawMode| u64::from(m.pixel_width) * u64::from(m.pixel_height);
        if pixels(mode) > pixels(entry) || (pixels(mode) == pixels(entry) && mode.native) {
            *entry = mode;
        }
    }
    let mut options: Vec<ModeOption> = best
        .values()
        .map(|mode| {
            let reference = mode.as_ref(0.0);
            // 同じ大きさのどれかが既定なら、まとめた方も既定として見せる。
            let native = modes
                .iter()
                .any(|other| other.usable && other.native && other.same_size(&reference));
            ModeOption {
                hidpi: mode.pixel_width > mode.width,
                current: is_current(&reference, current, false),
                native,
                mode: reference,
            }
        })
        .collect();
    options.sort_by(|a, b| order(&a.mode, &b.mode));
    options
}

/// リフレッシュレートと低解像度の版まで分けた一覧。重なるものは1つにする。
pub fn detailed_options(modes: &[RawMode], current: Option<&RawMode>) -> Vec<ModeOption> {
    let mut seen = BTreeMap::new();
    for mode in modes.iter().filter(|mode| mode.usable) {
        let key = (
            mode.width,
            mode.height,
            mode.pixel_width,
            mode.pixel_height,
            refresh_key(mode.refresh),
        );
        let entry = seen.entry(key).or_insert(ModeOption {
            mode: mode.as_ref(mode.refresh),
            hidpi: mode.pixel_width > mode.width,
            current: false,
            native: false,
        });
        entry.native |= mode.native;
    }
    let mut options: Vec<ModeOption> = seen
        .into_values()
        .map(|mut option| {
            option.current = is_current(&option.mode, current, true);
            option
        })
        .collect();
    options.sort_by(|a, b| order(&a.mode, &b.mode));
    options
}

/// 覚えている画面を、いまつながっている画面から探す。
/// 1. メーカー・型番・製造番号がそろう画面
/// 2. メーカー・型番がそろう画面（同じ型の別の個体や、製造番号を返さない画面）。複数なら名前の合う方
/// 3. メーカーや型番が分からないときは、名前が1つだけ合う画面
pub fn find_screen<'a>(target: &ScreenRef, screens: &'a [ScreenInfo]) -> Option<&'a ScreenInfo> {
    let known = target.vendor != 0 || target.model != 0;
    if known {
        if target.serial != 0 {
            if let Some(exact) = screens.iter().find(|info| {
                info.screen.vendor == target.vendor
                    && info.screen.model == target.model
                    && info.screen.serial == target.serial
            }) {
                return Some(exact);
            }
        }
        let same_model: Vec<&ScreenInfo> = screens
            .iter()
            .filter(|info| info.screen.vendor == target.vendor && info.screen.model == target.model)
            .collect();
        if let Some(found) = same_model
            .iter()
            .find(|info| info.screen.name == target.name)
            .or_else(|| same_model.first())
        {
            return Some(found);
        }
        return None;
    }
    let mut named = screens
        .iter()
        .filter(|info| !target.name.is_empty() && info.screen.name == target.name);
    match (named.next(), named.next()) {
        (Some(only), None) => Some(only),
        _ => None,
    }
}

/// 登録したモードに当たる、OS のモードを選ぶ。見た目と画素数が同じものに限り、
/// リフレッシュレートの指定があればそれに合うもの、無ければ最も高いもの（同じなら既定のもの）。
pub fn pick_mode(target: &DisplayModeRef, modes: &[RawMode]) -> Option<usize> {
    modes
        .iter()
        .enumerate()
        .filter(|(_, mode)| mode.usable && mode.same_size(target))
        .filter(|(_, mode)| target.refresh <= 0.0 || same_refresh(mode.refresh, target.refresh))
        .max_by_key(|(_, mode)| (refresh_key(mode.refresh), mode.native))
        .map(|(index, _)| index)
}

/// 次に切り替えるモード。いまのモードが登録の中にあればその次、無ければ先頭。
pub fn next_mode<'a>(
    modes: &'a [DisplayModeRef],
    current: Option<&RawMode>,
) -> Option<&'a DisplayModeRef> {
    let position = modes.iter().position(|mode| {
        current.is_some_and(|now| {
            now.same_size(mode) && (mode.refresh <= 0.0 || same_refresh(now.refresh, mode.refresh))
        })
    });
    match position {
        Some(index) => modes.get((index + 1) % modes.len()),
        None => modes.first(),
    }
}

/// 知らせに出すモードの書き方（例: `1920×1080`）。
pub fn mode_label(mode: &DisplayModeRef) -> String {
    if mode.refresh > 0.0 {
        format!("{}×{} {}Hz", mode.width, mode.height, mode.refresh.round())
    } else {
        format!("{}×{}", mode.width, mode.height)
    }
}

/// 設定画面の一覧。対応していない OS では空。
pub fn list(app: &AppHandle) -> Result<Vec<DisplayInfo>, String> {
    platform::list(app)
}

/// 解像度を変える。バックグラウンドのスレッドから呼ぶこと。
pub fn apply(app: &AppHandle, setting: &DisplaySetting) -> Result<(), String> {
    platform::apply(app, setting)
}

#[cfg(target_os = "macos")]
pub(crate) mod platform {
    use super::{find_screen, mode_label, next_mode, pick_mode, DisplayInfo, RawMode, ScreenInfo};
    use crate::model::{DisplaySetting, ScreenRef};
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;
    use objc2_foundation::{NSNumber, NSString};
    use std::{collections::HashMap, ffi::c_void, sync::mpsc, time::Duration};
    use tauri::AppHandle;

    type CFTypeRef = *const c_void;
    type CGDisplayModeRef = *const c_void;
    type CGDisplayConfigRef = *mut c_void;

    #[repr(C)]
    struct Opaque {
        _private: [u8; 0],
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFBooleanTrue: CFTypeRef;
        static kCFTypeDictionaryKeyCallBacks: Opaque;
        static kCFTypeDictionaryValueCallBacks: Opaque;
        fn CFDictionaryCreate(
            allocator: CFTypeRef,
            keys: *const CFTypeRef,
            values: *const CFTypeRef,
            count: isize,
            key_callbacks: *const Opaque,
            value_callbacks: *const Opaque,
        ) -> CFTypeRef;
        fn CFArrayGetCount(array: CFTypeRef) -> isize;
        fn CFArrayGetValueAtIndex(array: CFTypeRef, index: isize) -> CFTypeRef;
        fn CFRelease(object: CFTypeRef);
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        static kCGDisplayShowDuplicateLowResolutionModes: CFTypeRef;
        fn CGGetOnlineDisplayList(max: u32, displays: *mut u32, count: *mut u32) -> i32;
        fn CGMainDisplayID() -> u32;
        fn CGDisplayIsBuiltin(display: u32) -> u32;
        fn CGDisplayMirrorsDisplay(display: u32) -> u32;
        fn CGDisplayVendorNumber(display: u32) -> u32;
        fn CGDisplayModelNumber(display: u32) -> u32;
        fn CGDisplaySerialNumber(display: u32) -> u32;
        fn CGDisplayCopyAllDisplayModes(display: u32, options: CFTypeRef) -> CFTypeRef;
        fn CGDisplayCopyDisplayMode(display: u32) -> CGDisplayModeRef;
        fn CGDisplayModeRelease(mode: CGDisplayModeRef);
        fn CGDisplayModeGetWidth(mode: CGDisplayModeRef) -> usize;
        fn CGDisplayModeGetHeight(mode: CGDisplayModeRef) -> usize;
        fn CGDisplayModeGetPixelWidth(mode: CGDisplayModeRef) -> usize;
        fn CGDisplayModeGetPixelHeight(mode: CGDisplayModeRef) -> usize;
        fn CGDisplayModeGetRefreshRate(mode: CGDisplayModeRef) -> f64;
        fn CGDisplayModeGetIOFlags(mode: CGDisplayModeRef) -> u32;
        fn CGDisplayModeIsUsableForDesktopGUI(mode: CGDisplayModeRef) -> bool;
        fn CGBeginDisplayConfiguration(config: *mut CGDisplayConfigRef) -> i32;
        fn CGConfigureDisplayWithDisplayMode(
            config: CGDisplayConfigRef,
            display: u32,
            mode: CGDisplayModeRef,
            options: CFTypeRef,
        ) -> i32;
        fn CGCompleteDisplayConfiguration(config: CGDisplayConfigRef, option: u32) -> i32;
        fn CGCancelDisplayConfiguration(config: CGDisplayConfigRef) -> i32;
    }

    /// IOKit の `kDisplayModeDefaultFlag`。その画面の既定（推奨）のモードに付く。
    const DEFAULT_MODE_FLAG: u32 = 0x0000_0004;
    /// システム設定と同じく、ログアウトしても保つ。
    const CONFIGURE_PERMANENTLY: u32 = 2;

    /// 表示モードの配列。HiDPI のモードも含めるよう指定して取る。
    struct ModeList(CFTypeRef);

    impl ModeList {
        fn copy(display: u32) -> Option<Self> {
            unsafe {
                let keys = [kCGDisplayShowDuplicateLowResolutionModes];
                let values = [kCFBooleanTrue];
                let options = CFDictionaryCreate(
                    std::ptr::null(),
                    keys.as_ptr(),
                    values.as_ptr(),
                    1,
                    &kCFTypeDictionaryKeyCallBacks,
                    &kCFTypeDictionaryValueCallBacks,
                );
                let array = CGDisplayCopyAllDisplayModes(display, options);
                if !options.is_null() {
                    CFRelease(options);
                }
                (!array.is_null()).then_some(Self(array))
            }
        }

        fn refs(&self) -> Vec<CGDisplayModeRef> {
            unsafe {
                (0..CFArrayGetCount(self.0))
                    .map(|index| CFArrayGetValueAtIndex(self.0, index))
                    .collect()
            }
        }
    }

    impl Drop for ModeList {
        fn drop(&mut self) {
            unsafe { CFRelease(self.0) }
        }
    }

    fn read_mode(mode: CGDisplayModeRef) -> RawMode {
        unsafe {
            RawMode {
                width: CGDisplayModeGetWidth(mode) as u32,
                height: CGDisplayModeGetHeight(mode) as u32,
                pixel_width: CGDisplayModeGetPixelWidth(mode) as u32,
                pixel_height: CGDisplayModeGetPixelHeight(mode) as u32,
                refresh: CGDisplayModeGetRefreshRate(mode),
                usable: CGDisplayModeIsUsableForDesktopGUI(mode),
                native: CGDisplayModeGetIOFlags(mode) & DEFAULT_MODE_FLAG != 0,
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn read_all_modes(display: u32) -> Vec<RawMode> {
        ModeList::copy(display)
            .map(|list| list.refs().into_iter().map(read_mode).collect())
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn read_current_mode(display: u32) -> Option<RawMode> {
        current_mode(display)
    }

    fn current_mode(display: u32) -> Option<RawMode> {
        unsafe {
            let mode = CGDisplayCopyDisplayMode(display);
            if mode.is_null() {
                return None;
            }
            let raw = read_mode(mode);
            CGDisplayModeRelease(mode);
            Some(raw)
        }
    }

    /// 映っている画面の番号。ミラーリングで別の画面を映しているだけの画面は除く。
    fn online_ids() -> Vec<u32> {
        let mut ids = [0u32; 32];
        let mut count = 0u32;
        let status =
            unsafe { CGGetOnlineDisplayList(ids.len() as u32, ids.as_mut_ptr(), &mut count) };
        if status != 0 {
            return Vec::new();
        }
        ids[..count as usize]
            .iter()
            .copied()
            .filter(|&id| unsafe { CGDisplayMirrorsDisplay(id) } == 0)
            .collect()
    }

    /// 画面の名前は NSScreen からしか取れず、AppKit はメインスレッドで触る必要がある。
    pub(crate) fn screen_names_on_main_thread(mtm: MainThreadMarker) -> HashMap<u32, String> {
        NSScreen::screens(mtm)
            .iter()
            .filter_map(|screen| {
                Some((screen_number(&screen)?, screen.localizedName().to_string()))
            })
            .collect()
    }

    /// NSScreen の `NSScreenNumber` は CGDirectDisplayID と同じ番号。
    pub(crate) fn screen_number(screen: &NSScreen) -> Option<u32> {
        let description = screen.deviceDescription();
        let value = description.objectForKey(&NSString::from_str("NSScreenNumber"))?;
        let number = value.downcast_ref::<NSNumber>()?;
        Some(number.unsignedIntValue())
    }

    fn screen_names(app: &AppHandle) -> HashMap<u32, String> {
        if let Some(mtm) = MainThreadMarker::new() {
            return screen_names_on_main_thread(mtm);
        }
        let (sender, receiver) = mpsc::channel();
        let sent = app.run_on_main_thread(move || {
            if let Some(mtm) = MainThreadMarker::new() {
                let _ = sender.send(screen_names_on_main_thread(mtm));
            }
        });
        if sent.is_err() {
            return HashMap::new();
        }
        receiver
            .recv_timeout(Duration::from_secs(5))
            .unwrap_or_default()
    }

    pub(crate) fn describe(id: u32, names: &HashMap<u32, String>) -> ScreenInfo {
        let (vendor, model, serial) = unsafe {
            (
                CGDisplayVendorNumber(id),
                CGDisplayModelNumber(id),
                CGDisplaySerialNumber(id),
            )
        };
        let name = names
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("Display {vendor:04x}-{model:04x}"));
        ScreenInfo {
            id,
            screen: ScreenRef {
                vendor,
                model,
                serial,
                name,
            },
        }
    }

    pub(crate) fn screens_with(names: &HashMap<u32, String>) -> Vec<ScreenInfo> {
        online_ids()
            .into_iter()
            .map(|id| describe(id, names))
            .collect()
    }

    pub fn list(app: &AppHandle) -> Result<Vec<DisplayInfo>, String> {
        let names = screen_names(app);
        let main = unsafe { CGMainDisplayID() };
        let mut displays: Vec<DisplayInfo> = screens_with(&names)
            .into_iter()
            .map(|info| {
                let raw: Vec<RawMode> = ModeList::copy(info.id)
                    .map(|list| list.refs().into_iter().map(read_mode).collect())
                    .unwrap_or_default();
                let current = current_mode(info.id);
                DisplayInfo {
                    main: info.id == main,
                    builtin: unsafe { CGDisplayIsBuiltin(info.id) } != 0,
                    current: current.as_ref().map(|mode| mode.as_ref(mode.refresh)),
                    modes: super::summary_options(&raw, current.as_ref()),
                    detailed: super::detailed_options(&raw, current.as_ref()),
                    screen: info.screen,
                }
            })
            .collect();
        // メインの画面を先頭にする。
        displays.sort_by_key(|display| !display.main);
        Ok(displays)
    }

    pub fn apply(app: &AppHandle, setting: &DisplaySetting) -> Result<(), String> {
        let names = screen_names(app);
        let screens = screens_with(&names);
        let Some(info) = find_screen(&setting.screen, &screens) else {
            return Err(crate::error("display-not-found", &setting.screen.name));
        };
        let current = current_mode(info.id);
        let Some(target) = next_mode(&setting.modes, current.as_ref()) else {
            return Err(crate::error("invalid-display-setting", ""));
        };
        let missing = || {
            crate::error(
                "display-mode-missing",
                &format!("{} {}", info.screen.name, mode_label(target)),
            )
        };
        let list = ModeList::copy(info.id).ok_or_else(missing)?;
        let refs = list.refs();
        let raw: Vec<RawMode> = refs.iter().map(|&mode| read_mode(mode)).collect();
        let index = pick_mode(target, &raw).ok_or_else(missing)?;
        unsafe {
            let mut config: CGDisplayConfigRef = std::ptr::null_mut();
            let status = CGBeginDisplayConfiguration(&mut config);
            if status != 0 {
                return Err(crate::error("display-change-failed", &status.to_string()));
            }
            let status =
                CGConfigureDisplayWithDisplayMode(config, info.id, refs[index], std::ptr::null());
            if status != 0 {
                CGCancelDisplayConfiguration(config);
                return Err(crate::error("display-change-failed", &status.to_string()));
            }
            let status = CGCompleteDisplayConfiguration(config, CONFIGURE_PERMANENTLY);
            if status != 0 {
                return Err(crate::error("display-change-failed", &status.to_string()));
            }
        }
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) mod platform {
    use super::DisplayInfo;
    use crate::model::DisplaySetting;
    use tauri::AppHandle;

    pub fn list(_app: &AppHandle) -> Result<Vec<DisplayInfo>, String> {
        Ok(Vec::new())
    }

    pub fn apply(_app: &AppHandle, _setting: &DisplaySetting) -> Result<(), String> {
        Err(crate::error("unsupported-feature", ""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(width: u32, height: u32, scale: u32, refresh: f64) -> RawMode {
        RawMode {
            width,
            height,
            pixel_width: width * scale,
            pixel_height: height * scale,
            refresh,
            usable: true,
            native: false,
        }
    }

    fn reference(width: u32, height: u32, scale: u32, refresh: f64) -> DisplayModeRef {
        raw(width, height, scale, refresh).as_ref(refresh)
    }

    fn screen(id: u32, vendor: u32, model: u32, serial: u32, name: &str) -> ScreenInfo {
        ScreenInfo {
            id,
            screen: ScreenRef {
                vendor,
                model,
                serial,
                name: name.into(),
            },
        }
    }

    #[test]
    fn summary_keeps_one_entry_per_size_and_prefers_hidpi() {
        let mut native = raw(1512, 982, 2, 120.0);
        native.native = true;
        let mut unusable = raw(800, 600, 1, 60.0);
        unusable.usable = false;
        let modes = vec![
            raw(1512, 982, 1, 60.0),
            raw(1512, 982, 2, 60.0),
            native.clone(),
            raw(1800, 1169, 2, 120.0),
            raw(1024, 665, 2, 120.0),
            // 縦横比の違うモードは、まとめた一覧には出さない。
            raw(1024, 640, 2, 120.0),
            unusable,
        ];
        let options = summary_options(&modes, Some(&native));
        let sizes: Vec<(u32, u32, bool)> = options
            .iter()
            .map(|option| (option.mode.width, option.mode.height, option.hidpi))
            .collect();
        assert_eq!(
            sizes,
            vec![(1800, 1169, true), (1512, 982, true), (1024, 665, true)]
        );
        let middle = &options[1];
        assert_eq!(middle.mode.refresh, 0.0);
        assert!(middle.current);
        assert!(middle.native);
        assert!(!options[0].current);
    }

    #[test]
    fn detailed_list_separates_refresh_rates_and_low_resolution() {
        let current = raw(1920, 1080, 1, 60.0);
        let modes = vec![
            raw(1920, 1080, 2, 60.0),
            raw(1920, 1080, 1, 60.0),
            raw(1920, 1080, 1, 59.94),
            raw(1920, 1080, 1, 59.9400001),
            raw(1920, 1080, 1, 30.0),
        ];
        let options = detailed_options(&modes, Some(&current));
        let listed: Vec<(u32, f64, bool)> = options
            .iter()
            .map(|option| (option.mode.pixel_width, option.mode.refresh, option.current))
            .collect();
        assert_eq!(
            listed,
            vec![
                (3840, 60.0, false),
                (1920, 60.0, true),
                (1920, 59.94, false),
                (1920, 30.0, false)
            ]
        );
    }

    #[test]
    fn finds_the_same_screen_after_reconnecting() {
        let screens = vec![
            screen(5, 0x610, 0xa050, 0, "Built-in Retina Display"),
            screen(9, 0x10ac, 0x4321, 777, "DELL U2720Q"),
        ];
        let saved = ScreenRef {
            vendor: 0x10ac,
            model: 0x4321,
            serial: 777,
            name: "DELL U2720Q".into(),
        };
        assert_eq!(find_screen(&saved, &screens).map(|info| info.id), Some(9));
    }

    #[test]
    fn falls_back_to_the_same_model_then_to_the_name() {
        let screens = vec![
            screen(1, 0x10ac, 0x4321, 111, "DELL U2720Q"),
            screen(2, 0x10ac, 0x4321, 222, "DELL U2720Q (2)"),
        ];
        let other_unit = ScreenRef {
            vendor: 0x10ac,
            model: 0x4321,
            serial: 999,
            name: "DELL U2720Q (2)".into(),
        };
        assert_eq!(
            find_screen(&other_unit, &screens).map(|info| info.id),
            Some(2)
        );

        let unknown = ScreenRef {
            name: "DELL U2720Q".into(),
            ..ScreenRef::default()
        };
        assert_eq!(find_screen(&unknown, &screens).map(|info| info.id), Some(1));
    }

    #[test]
    fn a_disconnected_screen_is_not_found() {
        let screens = vec![screen(5, 0x610, 0xa050, 0, "Built-in Retina Display")];
        let saved = ScreenRef {
            vendor: 0x10ac,
            model: 0x4321,
            serial: 777,
            name: "DELL U2720Q".into(),
        };
        assert!(find_screen(&saved, &screens).is_none());
        let by_name = ScreenRef {
            name: "Projector".into(),
            ..ScreenRef::default()
        };
        assert!(find_screen(&by_name, &screens).is_none());
    }

    #[test]
    fn picks_the_fastest_matching_mode_unless_a_rate_was_chosen() {
        let mut native = raw(1920, 1080, 2, 60.0);
        native.native = true;
        let modes = vec![
            raw(1920, 1080, 1, 120.0),
            raw(1920, 1080, 2, 30.0),
            native,
            raw(1920, 1080, 2, 60.0),
            raw(2560, 1440, 1, 60.0),
        ];
        assert_eq!(pick_mode(&reference(1920, 1080, 2, 0.0), &modes), Some(2));
        assert_eq!(pick_mode(&reference(1920, 1080, 2, 30.0), &modes), Some(1));
        assert_eq!(pick_mode(&reference(1920, 1080, 2, 144.0), &modes), None);
        assert_eq!(pick_mode(&reference(1280, 720, 2, 0.0), &modes), None);
    }

    #[test]
    fn cycles_through_the_registered_modes() {
        let modes = vec![reference(1920, 1080, 2, 0.0), reference(1280, 720, 2, 0.0)];
        let first = raw(1920, 1080, 2, 60.0);
        let second = raw(1280, 720, 2, 60.0);
        let other = raw(2560, 1440, 1, 60.0);
        assert_eq!(next_mode(&modes, Some(&first)), Some(&modes[1]));
        assert_eq!(next_mode(&modes, Some(&second)), Some(&modes[0]));
        assert_eq!(next_mode(&modes, Some(&other)), Some(&modes[0]));
        assert_eq!(next_mode(&modes, None), Some(&modes[0]));
        assert_eq!(next_mode(&[], Some(&first)), None);
    }

    /// 実際の画面と表示モードを読むだけ（何も変えない）。`cargo test -- --ignored --nocapture` で見る。
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore]
    fn prints_the_connected_displays() {
        let screens = platform::screens_with(&std::collections::HashMap::new());
        assert!(!screens.is_empty());
        for info in screens {
            let raw = platform::read_all_modes(info.id);
            let current = platform::read_current_mode(info.id);
            println!("{:?} current={:?}", info.screen, current);
            for option in summary_options(&raw, current.as_ref()) {
                println!("  {:?}", option);
            }
            println!(
                "  ({} detailed)",
                detailed_options(&raw, current.as_ref()).len()
            );
        }
    }

    #[test]
    fn labels_include_the_rate_only_when_chosen() {
        assert_eq!(mode_label(&reference(1920, 1080, 2, 0.0)), "1920×1080");
        assert_eq!(
            mode_label(&reference(1920, 1080, 1, 59.94)),
            "1920×1080 60Hz"
        );
    }
}
