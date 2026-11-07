use std::sync::Mutex;
use tauri::AppHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    fn right(self) -> i32 {
        self.x + self.width
    }

    fn bottom(self) -> i32 {
        self.y + self.height
    }

    fn from_edges(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }
}

/// 大きさを変えずに画面中央へ寄せる操作。現在のウィンドウの大きさが要るので、
/// 画面の領域だけで決まる `layout_rect` とは別に扱う。
const CENTER: &str = "center";
/// 位置と大きさの比率を保って、次の画面（左から順に巡回）へ移す。
const NEXT_DISPLAY: &str = "next-display";
/// このアプリで整形する前の位置と大きさに戻す。
const RESTORE: &str = "restore";

/// 端の調整で1回に動かす量。作業領域の幅・高さのこの分の1ずつ動かす。
const EDGE_STEPS: i32 = 12;
/// 端の調整で縮めるときの最小の大きさ（1回分の量の何倍か）。
const EDGE_MIN_STEPS: i32 = 2;
/// ほぼ最大化で周りに空ける余白（作業領域の幅・高さに対する割合、片側分）。
const ALMOST_MAXIMIZE_MARGIN: f64 = 0.04;

/// 画面より大きいウィンドウは、はみ出さないよう左上を作業領域の角に合わせる。
fn centered(width: i32, height: i32, area: Rect) -> Rect {
    Rect {
        x: area.x + ((area.width - width) / 2).max(0),
        y: area.y + ((area.height - height) / 2).max(0),
        width,
        height,
    }
}

/// 長さ `length` を3等分したときの区切り（端数は最後の区画に入れる）。
fn thirds(length: i32) -> (i32, i32) {
    let third = length / 3;
    (third, third * 2)
}

/// 画面の領域だけで決まる整形。
fn layout_rect(action: &str, area: Rect) -> Result<Rect, String> {
    let half_width = area.width / 2;
    let half_height = area.height / 2;
    let (col1, col2) = thirds(area.width);
    let (row1, row2) = thirds(area.height);
    let columns = |from: i32, to: i32| Rect {
        x: area.x + from,
        width: to - from,
        ..area
    };
    let rows = |from: i32, to: i32| Rect {
        y: area.y + from,
        height: to - from,
        ..area
    };
    // 真ん中3分の2は、左右（上下）に6分の1ずつ空ける。
    let sixth_width = area.width / 6;
    let sixth_height = area.height / 6;

    Ok(match action {
        "left-half" => columns(0, half_width),
        "right-half" => columns(half_width, area.width),
        "top-half" => rows(0, half_height),
        "bottom-half" => rows(half_height, area.height),
        "top-left-quarter" => Rect {
            width: half_width,
            height: half_height,
            ..area
        },
        "top-right-quarter" => Rect {
            x: area.x + half_width,
            width: area.width - half_width,
            height: half_height,
            ..area
        },
        "bottom-left-quarter" => Rect {
            y: area.y + half_height,
            width: half_width,
            height: area.height - half_height,
            ..area
        },
        "bottom-right-quarter" => Rect {
            x: area.x + half_width,
            y: area.y + half_height,
            width: area.width - half_width,
            height: area.height - half_height,
        },
        "left-third" => columns(0, col1),
        "center-third" => columns(col1, col2),
        "right-third" => columns(col2, area.width),
        "left-two-thirds" => columns(0, col2),
        "center-two-thirds" => columns(sixth_width, area.width - sixth_width),
        "right-two-thirds" => columns(col1, area.width),
        "top-third" => rows(0, row1),
        "middle-third" => rows(row1, row2),
        "bottom-third" => rows(row2, area.height),
        "top-two-thirds" => rows(0, row2),
        "middle-two-thirds" => rows(sixth_height, area.height - sixth_height),
        "bottom-two-thirds" => rows(row1, area.height),
        "almost-maximize" => {
            let margin_x = (area.width as f64 * ALMOST_MAXIMIZE_MARGIN).round() as i32;
            let margin_y = (area.height as f64 * ALMOST_MAXIMIZE_MARGIN).round() as i32;
            Rect::from_edges(
                area.x + margin_x,
                area.y + margin_y,
                area.right() - margin_x,
                area.bottom() - margin_y,
            )
        }
        "maximize" => area,
        _ => return Err(format!("未対応のウィンドウ整形です: {action}")),
    })
}

/// 端の調整（`expand-left`、`shrink-bottom` など）。その辺だけを作業領域の1/12ずつ動かす。
/// 広げるときは作業領域の端で、縮めるときは最小の大きさで止める。
/// 既に止まる位置を越えているウィンドウは、その辺を動かさない（逆向きには動かさない）。
/// 端の調整ではない操作には `None` を返す。
fn adjust_edge(action: &str, window: Rect, area: Rect) -> Option<Rect> {
    let (direction, edge) = action.split_once('-')?;
    let grow = match direction {
        "expand" => true,
        "shrink" => false,
        _ => return None,
    };
    let step_x = (area.width / EDGE_STEPS).max(1);
    let step_y = (area.height / EDGE_STEPS).max(1);
    let min_width = step_x * EDGE_MIN_STEPS;
    let min_height = step_y * EDGE_MIN_STEPS;
    let (mut left, mut top, mut right, mut bottom) =
        (window.x, window.y, window.right(), window.bottom());
    match edge {
        "left" if grow => left = (left - step_x).max(area.x).min(left),
        "left" => left = (left + step_x).min(right - min_width).max(left),
        "right" if grow => right = (right + step_x).min(area.right()).max(right),
        "right" => right = (right - step_x).max(left + min_width).min(right),
        "top" if grow => top = (top - step_y).max(area.y).min(top),
        "top" => top = (top + step_y).min(bottom - min_height).max(top),
        "bottom" if grow => bottom = (bottom + step_y).min(area.bottom()).max(bottom),
        "bottom" => bottom = (bottom - step_y).max(top + min_height).min(bottom),
        _ => return None,
    }
    Some(Rect::from_edges(left, top, right, bottom))
}

/// 1台のディスプレイ。座標はどちらも左上原点・下向きのy（AccessibilityやAppleScriptの
/// ウィンドウ位置と同じ、論理ポイントのグローバル座標）。Windowsでは物理ピクセル。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Screen {
    /// ディスプレイ全体。どの画面に重なっているかの判定に使う。
    frame: Rect,
    /// メニューバーとDock（タスクバー）を除いた作業領域。整形先の計算に使う。
    visible: Rect,
}

/// macOSのNSScreenの矩形（左下原点・上向きのy）を左上原点の座標へ直す。
/// 基準はメニューバーのある主ディスプレイ（`NSScreen.screens` の先頭）の高さ。
/// 主ディスプレイより上に置いた画面は負のyになる。
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn flip_to_top_left(x: f64, y: f64, width: f64, height: f64, primary_height: f64) -> Rect {
    let left = x.round() as i32;
    let top = (primary_height - (y + height)).round() as i32;
    let right = (x + width).round() as i32;
    let bottom = (primary_height - y).round() as i32;
    Rect::from_edges(left, top, right, bottom)
}

fn overlap(a: Rect, b: Rect) -> i64 {
    let width = a.right().min(b.right()) - a.x.max(b.x);
    let height = a.bottom().min(b.bottom()) - a.y.max(b.y);
    if width <= 0 || height <= 0 {
        0
    } else {
        width as i64 * height as i64
    }
}

/// ウィンドウがいちばん重なっている画面を選ぶ。どの画面にも重ならないときは、
/// ウィンドウの中心にいちばん近い画面を選ぶ。
fn screen_for_window(window: Rect, screens: &[Screen]) -> Option<Screen> {
    let best = screens
        .iter()
        .copied()
        .max_by_key(|screen| overlap(window, screen.frame))?;
    if overlap(window, best.frame) > 0 {
        return Some(best);
    }
    let center_x = window.x as i64 * 2 + window.width as i64;
    let center_y = window.y as i64 * 2 + window.height as i64;
    screens.iter().copied().min_by_key(|screen| {
        let frame = screen.frame;
        // 中心から矩形までの距離（2倍のスケールで整数のまま比べる）。
        let dx = (frame.x as i64 * 2 - center_x)
            .max(center_x - frame.right() as i64 * 2)
            .max(0);
        let dy = (frame.y as i64 * 2 - center_y)
            .max(center_y - frame.bottom() as i64 * 2)
            .max(0);
        dx * dx + dy * dy
    })
}

/// 左から右へ（同じ横位置なら上から下へ）並べたときの次の画面。最後の画面の次は最初へ戻る。
/// 画面が1台だけのときは `None`。
fn next_screen(current: Screen, screens: &[Screen]) -> Option<Screen> {
    if screens.len() < 2 {
        return None;
    }
    let mut ordered = screens.to_vec();
    ordered.sort_by_key(|screen| (screen.frame.x, screen.frame.y));
    let index = ordered.iter().position(|screen| *screen == current)?;
    Some(ordered[(index + 1) % ordered.len()])
}

/// 作業領域 `from` の中での位置と大きさの比率を保って、作業領域 `to` へ写す。
/// 写した先からはみ出す分は、大きさを縮めてから内側へ寄せる。
fn move_between_areas(window: Rect, from: Rect, to: Rect) -> Rect {
    let scale = |value: i32, from_length: i32, to_length: i32| {
        (value as i64 * to_length as i64 / from_length.max(1) as i64) as i32
    };
    let width = scale(window.width, from.width, to.width).clamp(1, to.width.max(1));
    let height = scale(window.height, from.height, to.height).clamp(1, to.height.max(1));
    let x = (to.x + scale(window.x - from.x, from.width, to.width)).clamp(to.x, to.right() - width);
    let y = (to.y + scale(window.y - from.y, from.height, to.height)).clamp(to.y, to.bottom() - height);
    Rect { x, y, width, height }
}

/// 1つの画面の中で決まる整形後の位置と大きさ。`window` は整形前のウィンドウ。
fn target_rect(action: &str, window: Rect, area: Rect) -> Result<Rect, String> {
    if action == CENTER {
        return Ok(centered(window.width, window.height, area));
    }
    if let Some(rect) = adjust_edge(action, window, area) {
        return Ok(rect);
    }
    layout_rect(action, area)
}

/// 整形後の位置と大きさ。基準は、ウィンドウがいちばん重なっている画面の作業領域。
/// `restore` は履歴で決まるのでここでは扱わない。
fn plan(action: &str, window: Rect, screens: &[Screen]) -> Result<Rect, String> {
    let screen = screen_for_window(window, screens)
        .ok_or_else(|| "対象ディスプレイを取得できません".to_string())?;
    if action == NEXT_DISPLAY {
        let next = next_screen(screen, screens)
            .ok_or_else(|| "移動先のディスプレイがありません".to_string())?;
        return Ok(move_between_areas(window, screen.visible, next.visible));
    }
    target_rect(action, window, screen.visible)
}

/// 位置と大きさを比べるときの許容差。座標の丸めで1〜2ずれることがある。
const SAME_FRAME_TOLERANCE: i32 = 2;

fn same_frame(a: Rect, b: Rect) -> bool {
    (a.x - b.x).abs() <= SAME_FRAME_TOLERANCE
        && (a.y - b.y).abs() <= SAME_FRAME_TOLERANCE
        && (a.width - b.width).abs() <= SAME_FRAME_TOLERANCE
        && (a.height - b.height).abs() <= SAME_FRAME_TOLERANCE
}

/// 「元に戻す」のために、整形する前の位置と大きさをウィンドウごとに覚えておく。
/// 続けて整形しても最初の位置を保ち、利用者が手で動かした後は、その位置を新しい起点にする。
struct History {
    entries: Vec<HistoryEntry>,
}

struct HistoryEntry {
    window: String,
    /// 最後にこのアプリが整形した結果。ウィンドウがまだここにあれば、続けて整形したとみなす。
    applied: Rect,
    /// 整形を始める前の位置と大きさ。
    original: Rect,
}

/// 覚えておくウィンドウの数。古いものから捨てる。
const HISTORY_LIMIT: usize = 32;

impl History {
    const fn new() -> History {
        History { entries: Vec::new() }
    }

    fn take(&mut self, window: &str) -> Option<HistoryEntry> {
        let index = self.entries.iter().position(|entry| entry.window == window)?;
        Some(self.entries.remove(index))
    }

    /// `before` から `after` へ整形したことを記録する。
    fn record(&mut self, window: &str, before: Rect, after: Rect) {
        let original = match self.take(window) {
            Some(entry) if same_frame(entry.applied, before) => entry.original,
            _ => before,
        };
        if same_frame(original, after) {
            // 元の位置へ戻っただけなら、戻す先は無い。
            return;
        }
        self.entries.push(HistoryEntry {
            window: window.to_string(),
            applied: after,
            original,
        });
        if self.entries.len() > HISTORY_LIMIT {
            self.entries.remove(0);
        }
    }

    /// ウィンドウが最後に整形した位置のままなら、整形前の位置と大きさを返して履歴から外す。
    /// 手で動かした後や、覚えていないウィンドウには `None`。
    fn restore(&mut self, window: &str, current: Rect) -> Option<Rect> {
        let entry = self.take(window)?;
        same_frame(entry.applied, current).then_some(entry.original)
    }
}

static HISTORY: Mutex<History> = Mutex::new(History::new());

/// 操作対象の最前面のウィンドウ。`key` は「元に戻す」の履歴でウィンドウを見分けるのに使う。
#[cfg_attr(not(any(target_os = "macos", windows)), allow(dead_code))]
struct FrontWindow {
    key: String,
    frame: Rect,
    #[cfg(windows)]
    handle: windows_sys::Win32::Foundation::HWND,
}

/// バックグラウンドスレッドから呼ぶこと。
#[cfg_attr(not(any(target_os = "macos", windows)), allow(unused_variables, unreachable_code))]
pub fn apply(app: &AppHandle, action: &str) -> Result<(), String> {
    let front = platform::front_window()?;
    if action == RESTORE {
        let original = HISTORY
            .lock()
            .map_err(|error| error.to_string())?
            .restore(&front.key, front.frame)
            .ok_or_else(|| "戻す位置がありません".to_string())?;
        platform::set_frame(&front, original)?;
        return Ok(());
    }
    let screens = platform::screens(app)?;
    let target = plan(action, front.frame, &screens)?;
    let actual = platform::set_frame(&front, target)?;
    if let Ok(mut history) = HISTORY.lock() {
        history.record(&front.key, front.frame, actual);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{flip_to_top_left, FrontWindow, Rect, Screen};
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;
    use std::{process::Command, sync::mpsc, time::Duration};
    use tauri::AppHandle;

    /// 最前面のアプリのプロセスIDと、その前面ウィンドウの位置・大きさを読む。
    pub fn front_window() -> Result<FrontWindow, String> {
        let output = run(
            "tell application \"System Events\"
                tell first application process whose frontmost is true
                    set processId to unix id
                    tell front window
                        set {x, y} to position
                        set {w, h} to size
                    end tell
                end tell
            end tell
            return (processId as text) & \",\" & (x as text) & \",\" & (y as text) & \",\" & (w as text) & \",\" & (h as text)",
        )?;
        let (pid, frame) = parse_window(&output)
            .ok_or_else(|| format!("ウィンドウの位置を読み取れません: {output}"))?;
        Ok(FrontWindow {
            key: pid.to_string(),
            frame,
        })
    }

    /// 位置と大きさを変え、アプリが実際に受け入れた位置と大きさを返す
    /// （最小の大きさや文字単位の大きさに合わせて、アプリが値を変えることがある）。
    pub fn set_frame(window: &FrontWindow, target: Rect) -> Result<Rect, String> {
        let output = run(&format!(
            "tell application \"System Events\"
                tell first application process whose unix id is {pid}
                    tell front window
                        set {{position, size}} to {{{{{x}, {y}}}, {{{width}, {height}}}}}
                        set {{x, y}} to position
                        set {{w, h}} to size
                    end tell
                end tell
            end tell
            return \"0,\" & (x as text) & \",\" & (y as text) & \",\" & (w as text) & \",\" & (h as text)",
            pid = window.key,
            x = target.x,
            y = target.y,
            width = target.width,
            height = target.height,
        ))?;
        Ok(parse_window(&output).map_or(target, |(_, frame)| frame))
    }

    /// `pid,x,y,width,height` を読む。座標は小数で返ることがあるので丸める。
    fn parse_window(text: &str) -> Option<(i64, Rect)> {
        let values: Vec<i64> = text
            .trim()
            .split(',')
            .map(|value| value.trim().parse::<f64>().ok().map(|v| v.round() as i64))
            .collect::<Option<_>>()?;
        match values[..] {
            [pid, x, y, width, height] => Some((
                pid,
                Rect {
                    x: x as i32,
                    y: y as i32,
                    width: width as i32,
                    height: height as i32,
                },
            )),
            _ => None,
        }
    }

    fn run(script: &str) -> Result<String, String> {
        let output = Command::new("/usr/bin/osascript")
            .args(["-e", script])
            .output()
            .map_err(|error| format!("ウィンドウ整形を実行できません: {error}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// ディスプレイの照会はAppKitに触れるのでメインスレッドへ回す。
    pub fn screens(app: &AppHandle) -> Result<Vec<Screen>, String> {
        let (sender, receiver) = mpsc::channel();
        app.run_on_main_thread(move || {
            let _ = sender.send(screens_on_main_thread());
        })
        .map_err(|error| error.to_string())?;
        let screens = receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "対象ディスプレイを取得できません".to_string())?;
        if screens.is_empty() {
            Err("対象ディスプレイを取得できません".into())
        } else {
            Ok(screens)
        }
    }

    /// NSScreenは左下原点なので、主ディスプレイ（先頭）の高さを基準に左上原点へ直す。
    /// visibleFrameはメニューバーとDockを除いた領域で、画面ごとの配置にも追従する。
    fn screens_on_main_thread() -> Vec<Screen> {
        let Some(mtm) = MainThreadMarker::new() else {
            return Vec::new();
        };
        let screens = NSScreen::screens(mtm);
        let Some(primary) = screens.firstObject() else {
            return Vec::new();
        };
        let primary_height = primary.frame().size.height;
        screens
            .iter()
            .map(|screen| {
                let convert = |rect: objc2_foundation::NSRect| {
                    flip_to_top_left(
                        rect.origin.x,
                        rect.origin.y,
                        rect.size.width,
                        rect.size.height,
                        primary_height,
                    )
                };
                Screen {
                    frame: convert(screen.frame()),
                    visible: convert(screen.visibleFrame()),
                }
            })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parses_the_applescript_window() {
            assert_eq!(
                parse_window("512, -1440, -200, 800.4, 600\n"),
                Some((512, Rect { x: -1440, y: -200, width: 800, height: 600 }))
            );
            assert_eq!(parse_window("10,20,30,40"), None);
            assert_eq!(parse_window(""), None);
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::{FrontWindow, Rect, Screen};
    use std::mem::size_of;
    use tauri::AppHandle;
    use windows_sys::{
        core::BOOL,
        Win32::{
            Foundation::{LPARAM, RECT},
            Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
            UI::WindowsAndMessaging::{
                GetForegroundWindow, GetWindowRect, SetWindowPos, ShowWindow, SWP_NOACTIVATE,
                SWP_NOZORDER, SW_RESTORE,
            },
        },
    };

    fn to_rect(rect: RECT) -> Rect {
        Rect::from_edges(rect.left, rect.top, rect.right, rect.bottom)
    }

    fn window_rect(handle: windows_sys::Win32::Foundation::HWND) -> Result<Rect, String> {
        let mut frame: RECT = unsafe { std::mem::zeroed() };
        if unsafe { GetWindowRect(handle, &mut frame) } == 0 {
            return Err("ウィンドウの大きさを取得できません".into());
        }
        Ok(to_rect(frame))
    }

    pub fn front_window() -> Result<FrontWindow, String> {
        let handle = unsafe { GetForegroundWindow() };
        if handle.is_null() {
            return Err("操作対象のウィンドウが見つかりません".into());
        }
        // 最大化中は元の大きさに戻してから測る。
        unsafe { ShowWindow(handle, SW_RESTORE) };
        Ok(FrontWindow {
            key: format!("{:p}", handle),
            frame: window_rect(handle)?,
            handle,
        })
    }

    pub fn set_frame(window: &FrontWindow, target: Rect) -> Result<Rect, String> {
        let moved = unsafe {
            SetWindowPos(
                window.handle,
                std::ptr::null_mut(),
                target.x,
                target.y,
                target.width,
                target.height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
        if moved == 0 {
            return Err("ウィンドウの位置とサイズを変更できません".into());
        }
        Ok(window_rect(window.handle).unwrap_or(target))
    }

    unsafe extern "system" fn collect(
        monitor: HMONITOR,
        _hdc: HDC,
        _clip: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let screens = &mut *(data as *mut Vec<Screen>);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            rcMonitor: std::mem::zeroed(),
            rcWork: std::mem::zeroed(),
            dwFlags: 0,
        };
        if GetMonitorInfoW(monitor, &mut info) != 0 {
            screens.push(Screen {
                frame: to_rect(info.rcMonitor),
                visible: to_rect(info.rcWork),
            });
        }
        1
    }

    pub fn screens(_app: &AppHandle) -> Result<Vec<Screen>, String> {
        let mut screens: Vec<Screen> = Vec::new();
        unsafe {
            EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null(),
                Some(collect),
                &mut screens as *mut Vec<Screen> as LPARAM,
            );
        }
        if screens.is_empty() {
            Err("ディスプレイの作業領域を取得できません".into())
        } else {
            Ok(screens)
        }
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
    use super::{FrontWindow, Rect, Screen};
    use tauri::AppHandle;

    const UNSUPPORTED: &str = "このOSではウィンドウ整形を利用できません";

    pub fn front_window() -> Result<FrontWindow, String> {
        Err(UNSUPPORTED.into())
    }

    pub fn set_frame(_window: &FrontWindow, _target: Rect) -> Result<Rect, String> {
        Err(UNSUPPORTED.into())
    }

    pub fn screens(_app: &AppHandle) -> Result<Vec<Screen>, String> {
        Err(UNSUPPORTED.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect {
        x: 0,
        y: 25,
        width: 1200,
        height: 900,
    };

    #[test]
    fn calculates_thirds_without_gaps() {
        assert_eq!(layout_rect("left-third", SCREEN).unwrap().width, 400);
        assert_eq!(layout_rect("center-third", SCREEN).unwrap().x, 400);
        let right = layout_rect("right-third", SCREEN).unwrap();
        assert_eq!(right.x + right.width, 1200);
    }

    #[test]
    fn calculates_halves_and_full_screen() {
        assert_eq!(layout_rect("right-half", SCREEN).unwrap().x, 600);
        assert_eq!(layout_rect("bottom-half", SCREEN).unwrap().y, 475);
        assert_eq!(layout_rect("maximize", SCREEN).unwrap(), SCREEN);
    }

    #[test]
    fn quarters_tile_the_screen() {
        let top_left = layout_rect("top-left-quarter", SCREEN).unwrap();
        let bottom_right = layout_rect("bottom-right-quarter", SCREEN).unwrap();
        assert_eq!((top_left.x, top_left.y, top_left.width, top_left.height), (0, 25, 600, 450));
        assert_eq!((bottom_right.x, bottom_right.y), (600, 475));
        assert_eq!(bottom_right.x + bottom_right.width, 1200);
        assert_eq!(bottom_right.y + bottom_right.height, 925);
        assert_eq!(layout_rect("top-right-quarter", SCREEN).unwrap().y, 25);
        assert_eq!(layout_rect("bottom-left-quarter", SCREEN).unwrap().x, 0);
    }

    #[test]
    fn centering_keeps_the_size_and_stays_on_screen() {
        assert_eq!(
            centered(600, 400, SCREEN),
            Rect { x: 300, y: 275, width: 600, height: 400 }
        );
        let oversized = centered(1600, 1000, SCREEN);
        assert_eq!((oversized.x, oversized.y), (0, 25));
    }

    /// 主ディスプレイ（1440x900、メニューバー25、Dock下70）。NSScreen座標で与えて変換する。
    fn primary() -> Screen {
        Screen {
            frame: flip_to_top_left(0.0, 0.0, 1440.0, 900.0, 900.0),
            visible: flip_to_top_left(0.0, 70.0, 1440.0, 805.0, 900.0),
        }
    }

    /// 2台目（1920x1080、メニューバー25）を、主ディスプレイから見た左下原点の位置に置く。
    fn second_at(x: f64, y: f64) -> Screen {
        Screen {
            frame: flip_to_top_left(x, y, 1920.0, 1080.0, 900.0),
            visible: flip_to_top_left(x, y, 1920.0, 1055.0, 900.0),
        }
    }

    #[test]
    fn flips_nsscreen_coordinates_to_top_left() {
        let main = primary();
        assert_eq!(main.frame, Rect { x: 0, y: 0, width: 1440, height: 900 });
        assert_eq!(main.visible, Rect { x: 0, y: 25, width: 1440, height: 805 });
        // 右に下端をそろえて置いた背の高い画面は、上に180はみ出す。
        let right = second_at(1440.0, 0.0);
        assert_eq!(right.frame, Rect { x: 1440, y: -180, width: 1920, height: 1080 });
        assert_eq!(right.visible, Rect { x: 1440, y: -155, width: 1920, height: 1055 });
        // 上に置いた画面は負のy、左に置いた画面は負のx。
        assert_eq!(second_at(0.0, 900.0).frame, Rect { x: 0, y: -1080, width: 1920, height: 1080 });
        assert_eq!(second_at(-1920.0, -300.0).frame, Rect { x: -1920, y: 120, width: 1920, height: 1080 });
    }

    #[test]
    fn picks_the_screen_the_window_overlaps_most() {
        for second in [
            second_at(1440.0, 0.0),     // 右
            second_at(-1920.0, -300.0), // 左（下にずらして配置）
            second_at(-240.0, 900.0),   // 上
        ] {
            let screens = [primary(), second];
            let inside = Rect {
                x: second.frame.x + 100,
                y: second.frame.y + 100,
                width: 800,
                height: 600,
            };
            assert_eq!(screen_for_window(inside, &screens), Some(second));
            let on_primary = Rect { x: 200, y: 200, width: 800, height: 600 };
            assert_eq!(screen_for_window(on_primary, &screens), Some(primary()));
        }
    }

    #[test]
    fn straddling_window_goes_to_the_larger_share() {
        let screens = [primary(), second_at(1440.0, 0.0)];
        // 左端がまだ画面1に残っていても、大半が画面2なら画面2。
        let mostly_right = Rect { x: 1240, y: 100, width: 800, height: 600 };
        assert_eq!(screen_for_window(mostly_right, &screens), Some(screens[1]));
        let mostly_left = Rect { x: 1000, y: 100, width: 800, height: 600 };
        assert_eq!(screen_for_window(mostly_left, &screens), Some(screens[0]));
    }

    #[test]
    fn offscreen_window_goes_to_the_nearest_screen() {
        let screens = [primary(), second_at(1440.0, 0.0)];
        let far_right = Rect { x: 4000, y: 0, width: 300, height: 300 };
        assert_eq!(screen_for_window(far_right, &screens), Some(screens[1]));
        let far_left = Rect { x: -900, y: 300, width: 300, height: 300 };
        assert_eq!(screen_for_window(far_left, &screens), Some(screens[0]));
        assert_eq!(screen_for_window(far_left, &[]), None);
    }

    #[test]
    fn layouts_land_inside_the_chosen_screen() {
        let screens = [primary(), second_at(-1920.0, -300.0)];
        let window = Rect { x: -1500, y: 400, width: 700, height: 500 };
        let area = screen_for_window(window, &screens).unwrap().visible;
        assert_eq!(area, Rect { x: -1920, y: 145, width: 1920, height: 1055 });
        let right = target_rect("right-half", window, area).unwrap();
        assert_eq!((right.x, right.y, right.width), (-960, 145, 960));
        let centered = target_rect(CENTER, window, area).unwrap();
        assert_eq!(centered, Rect { x: -1310, y: 422, width: 700, height: 500 });
    }

    #[test]
    fn two_thirds_and_vertical_thirds_cover_the_screen() {
        let center = layout_rect("center-two-thirds", SCREEN).unwrap();
        assert_eq!((center.x, center.width), (200, 800));
        assert_eq!((center.y, center.height), (25, 900));
        let top = layout_rect("top-third", SCREEN).unwrap();
        let middle = layout_rect("middle-third", SCREEN).unwrap();
        let bottom = layout_rect("bottom-third", SCREEN).unwrap();
        assert_eq!((top.x, top.y, top.width, top.height), (0, 25, 1200, 300));
        assert_eq!((middle.y, middle.height), (325, 300));
        assert_eq!((bottom.y, bottom.bottom()), (625, 925));
        assert_eq!(layout_rect("top-two-thirds", SCREEN).unwrap().bottom(), 625);
        let middle_two = layout_rect("middle-two-thirds", SCREEN).unwrap();
        assert_eq!((middle_two.y, middle_two.height), (175, 600));
        let bottom_two = layout_rect("bottom-two-thirds", SCREEN).unwrap();
        assert_eq!((bottom_two.y, bottom_two.bottom()), (325, 925));
    }

    #[test]
    fn thirds_absorb_the_remainder_in_the_last_part() {
        let odd = Rect { x: -1001, y: -700, width: 1001, height: 700 };
        let right = layout_rect("right-third", odd).unwrap();
        assert_eq!((right.x, right.right()), (-1001 + 666, 0));
        let bottom = layout_rect("bottom-third", odd).unwrap();
        assert_eq!((bottom.y, bottom.bottom()), (-700 + 466, 0));
        let center = layout_rect("center-two-thirds", odd).unwrap();
        assert_eq!(center.x - odd.x, odd.right() - center.right());
    }

    #[test]
    fn almost_maximize_leaves_an_even_margin() {
        let rect = layout_rect("almost-maximize", SCREEN).unwrap();
        assert_eq!(rect, Rect { x: 48, y: 61, width: 1104, height: 828 });
        assert!(layout_rect("unknown", SCREEN).is_err());
    }

    #[test]
    fn edges_move_one_twelfth_at_a_time() {
        // 1200x900 の1/12は100x75。
        let window = Rect { x: 300, y: 200, width: 600, height: 400 };
        let edge = |action: &str| target_rect(action, window, SCREEN).unwrap();
        assert_eq!(edge("expand-left"), Rect { x: 200, y: 200, width: 700, height: 400 });
        assert_eq!(edge("shrink-left"), Rect { x: 400, y: 200, width: 500, height: 400 });
        assert_eq!(edge("expand-right"), Rect { x: 300, y: 200, width: 700, height: 400 });
        assert_eq!(edge("shrink-right"), Rect { x: 300, y: 200, width: 500, height: 400 });
        assert_eq!(edge("expand-top"), Rect { x: 300, y: 125, width: 600, height: 475 });
        assert_eq!(edge("shrink-top"), Rect { x: 300, y: 275, width: 600, height: 325 });
        assert_eq!(edge("expand-bottom"), Rect { x: 300, y: 200, width: 600, height: 475 });
        assert_eq!(edge("shrink-bottom"), Rect { x: 300, y: 200, width: 600, height: 325 });
    }

    #[test]
    fn edges_stop_at_the_screen_and_the_minimum_size() {
        let window = Rect { x: 40, y: 50, width: 260, height: 160 };
        let edge = |action: &str, window: Rect| target_rect(action, window, SCREEN).unwrap();
        // 広げるときは作業領域の端で止まる。
        assert_eq!(edge("expand-left", window).x, 0);
        assert_eq!(edge("expand-top", window).y, 25);
        let wide = Rect { x: 1150, y: 850, width: 30, height: 30 };
        assert_eq!(edge("expand-right", wide).right(), 1200);
        assert_eq!(edge("expand-bottom", wide).bottom(), 925);
        // 縮めるときは 1/12 の2つ分（200x150）で止まり、反対の辺は動かない。
        let narrower = edge("shrink-left", window);
        assert_eq!((narrower.x, narrower.right()), (100, 300));
        assert_eq!(edge("shrink-left", narrower), narrower);
        let shorter = edge("shrink-bottom", window);
        assert_eq!((shorter.y, shorter.height), (50, 150));
        // 既に止まる位置を越えていたら、逆向きに動かさない。
        let tiny = Rect { x: 500, y: 500, width: 50, height: 50 };
        assert_eq!(edge("shrink-right", tiny), tiny);
        let outside = Rect { x: -30, y: 0, width: 400, height: 300 };
        assert_eq!(edge("expand-left", outside), outside);
        assert_eq!(edge("expand-top", outside), outside);
        // 繰り返すと少しずつ広がり、端に着いたら止まる。
        let mut growing = Rect { x: 500, y: 300, width: 200, height: 200 };
        for _ in 0..10 {
            growing = edge("expand-right", growing);
        }
        assert_eq!((growing.x, growing.right()), (500, 1200));
    }

    #[test]
    fn edges_use_the_screen_the_window_is_on() {
        let screens = [primary(), second_at(-1920.0, -300.0)];
        let window = Rect { x: -1500, y: 400, width: 700, height: 500 };
        // 左の画面（1920x1055、1/12は160x87）を基準にする。
        let rect = plan("expand-left", window, &screens).unwrap();
        assert_eq!(rect, Rect { x: -1660, y: 400, width: 860, height: 500 });
        let rect = plan("expand-bottom", window, &screens).unwrap();
        assert_eq!(rect.bottom(), 987);
        let rect = plan("center-two-thirds", window, &screens).unwrap();
        assert_eq!((rect.x, rect.width, rect.y), (-1600, 1280, 145));
    }

    #[test]
    fn next_display_cycles_left_to_right() {
        let left = second_at(-1920.0, -300.0);
        let right = second_at(1440.0, 0.0);
        let screens = [primary(), right, left];
        assert_eq!(next_screen(left, &screens), Some(primary()));
        assert_eq!(next_screen(primary(), &screens), Some(right));
        assert_eq!(next_screen(right, &screens), Some(left));
        assert_eq!(next_screen(primary(), &[primary()]), None);
        let window = Rect { x: 100, y: 100, width: 300, height: 300 };
        assert!(plan(NEXT_DISPLAY, window, &[primary()]).is_err());
    }

    #[test]
    fn next_display_keeps_the_relative_position_and_size() {
        let screens = [primary(), second_at(1440.0, 0.0)];
        // 画面1（作業領域 0,25 1440x805）の右半分を、画面2（1440,-155 1920x1055）へ。
        let right_half = Rect { x: 720, y: 25, width: 720, height: 805 };
        let moved = plan(NEXT_DISPLAY, right_half, &screens).unwrap();
        assert_eq!(moved, Rect { x: 2400, y: -155, width: 960, height: 1055 });
        // 戻すと元の位置に戻り、画面1へ巡回する。
        assert_eq!(plan(NEXT_DISPLAY, moved, &screens).unwrap(), right_half);
        // はみ出していたウィンドウは、移動先の作業領域に収める。
        let overhanging = Rect { x: 1000, y: 700, width: 600, height: 400 };
        let moved = plan(NEXT_DISPLAY, overhanging, &screens).unwrap();
        assert!(moved.x >= 1440 && moved.right() <= 3360);
        assert!(moved.y >= -155 && moved.bottom() <= 900);
    }

    #[test]
    fn every_built_in_action_is_supported() {
        let screens = [primary(), second_at(1440.0, 0.0)];
        let window = Rect { x: 200, y: 200, width: 800, height: 500 };
        for binding in crate::model::default_window_actions() {
            if binding.action == RESTORE {
                continue;
            }
            let rect = plan(&binding.action, window, &screens)
                .unwrap_or_else(|error| panic!("{}: {error}", binding.action));
            assert!(rect.width > 0 && rect.height > 0, "{}", binding.action);
        }
    }

    #[test]
    fn history_restores_the_frame_before_the_first_layout() {
        let mut history = History::new();
        let original = Rect { x: 100, y: 100, width: 500, height: 400 };
        let half = Rect { x: 0, y: 25, width: 600, height: 900 };
        let edge = Rect { x: 0, y: 25, width: 700, height: 900 };
        history.record("a", original, half);
        history.record("a", half, edge);
        // 続けて整形しても、戻す先は最初の位置。
        assert_eq!(history.restore("a", Rect { x: 1, ..edge }), Some(original));
        // 戻した後は、もう戻す先は無い。
        assert_eq!(history.restore("a", original), None);
    }

    #[test]
    fn history_starts_over_after_a_manual_move() {
        let mut history = History::new();
        let original = Rect { x: 100, y: 100, width: 500, height: 400 };
        let half = Rect { x: 0, y: 25, width: 600, height: 900 };
        history.record("a", original, half);
        // 手で動かしてから整形したら、その位置が新しい戻す先。
        let moved = Rect { x: 300, y: 300, width: 600, height: 500 };
        history.record("a", moved, half);
        assert_eq!(history.restore("a", half), Some(moved));
        // 整形した位置から手で動かしていたら戻さない。
        history.record("b", original, half);
        assert_eq!(history.restore("b", moved), None);
        // 覚えていないウィンドウ、元の位置へ戻っただけの記録は戻さない。
        assert_eq!(history.restore("c", half), None);
        history.record("d", original, original);
        assert_eq!(history.restore("d", original), None);
    }

    #[test]
    fn history_keeps_windows_apart_and_forgets_the_oldest() {
        let mut history = History::new();
        let frame = |n: i32| Rect { x: n, y: 0, width: 100, height: 100 };
        history.record("a", frame(1), frame(10));
        history.record("b", frame(2), frame(20));
        assert_eq!(history.restore("a", frame(10)), Some(frame(1)));
        assert_eq!(history.restore("b", frame(20)), Some(frame(2)));
        for n in 0..=HISTORY_LIMIT as i32 {
            history.record(&n.to_string(), frame(n * 10), frame(n * 10 + 5));
        }
        assert_eq!(history.entries.len(), HISTORY_LIMIT);
        assert_eq!(history.restore("0", frame(5)), None);
        assert_eq!(history.restore("1", frame(15)), Some(frame(10)));
    }
}
