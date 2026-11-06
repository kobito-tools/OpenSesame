use tauri::AppHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

/// 大きさを変えずに画面中央へ寄せる操作。現在のウィンドウの大きさが要るので、
/// 画面の領域だけで決まる `layout_rect` とは別に扱う。
const CENTER: &str = "center";

/// 画面より大きいウィンドウは、はみ出さないよう左上を作業領域の角に合わせる。
#[cfg_attr(not(any(target_os = "macos", windows)), allow(dead_code))]
fn centered(width: i32, height: i32, area: Rect) -> Rect {
    Rect {
        x: area.x + ((area.width - width) / 2).max(0),
        y: area.y + ((area.height - height) / 2).max(0),
        width,
        height,
    }
}

fn layout_rect(action: &str, area: Rect) -> Result<Rect, String> {
    let half_width = area.width / 2;
    let half_height = area.height / 2;
    let third = area.width / 3;
    let two_thirds = area.width - third;

    Ok(match action {
        "left-half" => Rect {
            width: half_width,
            ..area
        },
        "right-half" => Rect {
            x: area.x + half_width,
            width: area.width - half_width,
            ..area
        },
        "top-half" => Rect {
            height: half_height,
            ..area
        },
        "bottom-half" => Rect {
            y: area.y + half_height,
            height: area.height - half_height,
            ..area
        },
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
        "left-third" => Rect {
            width: third,
            ..area
        },
        "center-third" => Rect {
            x: area.x + third,
            width: third,
            ..area
        },
        "right-third" => Rect {
            x: area.x + third * 2,
            width: area.width - third * 2,
            ..area
        },
        "left-two-thirds" => Rect {
            width: two_thirds,
            ..area
        },
        "right-two-thirds" => Rect {
            x: area.x + third,
            width: area.width - third,
            ..area
        },
        "maximize" => area,
        _ => return Err(format!("未対応のウィンドウ整形です: {action}")),
    })
}

/// 1台のディスプレイ。座標はどちらも左上原点・下向きのy（AccessibilityやAppleScriptの
/// ウィンドウ位置と同じ、論理ポイントのグローバル座標）。
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Screen {
    /// ディスプレイ全体。どの画面に重なっているかの判定に使う。
    frame: Rect,
    /// メニューバーとDockを除いた作業領域。整形先の計算に使う。
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
    Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn overlap(a: Rect, b: Rect) -> i64 {
    let width = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
    let height = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);
    if width <= 0 || height <= 0 {
        0
    } else {
        width as i64 * height as i64
    }
}

/// ウィンドウがいちばん重なっている画面を選ぶ。どの画面にも重ならないときは、
/// ウィンドウの中心にいちばん近い画面を選ぶ。
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
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
            .max(center_x - (frame.x + frame.width) as i64 * 2)
            .max(0);
        let dy = (frame.y as i64 * 2 - center_y)
            .max(center_y - (frame.y + frame.height) as i64 * 2)
            .max(0);
        dx * dx + dy * dy
    })
}

/// 整形後の位置と大きさ。`window` は整形前のウィンドウ（中央寄せで大きさを保つのに使う）。
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn target_rect(action: &str, window: Rect, area: Rect) -> Result<Rect, String> {
    if action == CENTER {
        Ok(centered(window.width, window.height, area))
    } else {
        layout_rect(action, area)
    }
}

pub fn apply(app: &AppHandle, action: &str) -> Result<(), String> {
    platform::apply(app, action)
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{flip_to_top_left, screen_for_window, target_rect, Rect, Screen};
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;
    use std::{process::Command, sync::mpsc, time::Duration};
    use tauri::AppHandle;

    const FRONT_WINDOW: &str =
        "tell application \"System Events\" to tell first application process whose frontmost is true to tell front window";

    /// バックグラウンドスレッドから呼ぶこと。
    /// ディスプレイの照会はAppKitに触れるのでメインスレッドへ回し、
    /// 時間のかかるosascriptは呼び出し元のスレッドで動かす。
    ///
    /// 整形先は、マウスカーソルではなく対象のウィンドウがいちばん重なっている画面にする。
    pub fn apply(app: &AppHandle, action: &str) -> Result<(), String> {
        let window = front_window_frame()?;
        let screens = screens(app)?;
        let screen = screen_for_window(window, &screens)
            .ok_or_else(|| "対象ディスプレイを取得できません".to_string())?;
        let target = target_rect(action, window, screen.visible)?;
        run(&format!(
            "{FRONT_WINDOW} to set {{position, size}} to {{{{{}, {}}}, {{{}, {}}}}}",
            target.x, target.y, target.width, target.height
        ))?;
        Ok(())
    }

    fn front_window_frame() -> Result<Rect, String> {
        let output = run(&format!(
            "{FRONT_WINDOW}
                set {{x, y}} to position
                set {{w, h}} to size
            end tell
            return (x as text) & \",\" & (y as text) & \",\" & (w as text) & \",\" & (h as text)"
        ))?;
        parse_frame(&output).ok_or_else(|| format!("ウィンドウの位置を読み取れません: {output}"))
    }

    fn parse_frame(text: &str) -> Option<Rect> {
        let values: Vec<i32> = text
            .trim()
            .split(',')
            .map(|value| value.trim().parse::<f64>().ok().map(|v| v.round() as i32))
            .collect::<Option<_>>()?;
        match values[..] {
            [x, y, width, height] => Some(Rect { x, y, width, height }),
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

    fn screens(app: &AppHandle) -> Result<Vec<Screen>, String> {
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
        fn parses_the_applescript_frame() {
            assert_eq!(
                parse_frame("-1440, -200, 800, 600\n"),
                Some(Rect { x: -1440, y: -200, width: 800, height: 600 })
            );
            assert_eq!(parse_frame("10,20,30"), None);
            assert_eq!(parse_frame(""), None);
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::{centered, layout_rect, Rect, CENTER};
    use std::mem::size_of;
    use tauri::AppHandle;
    use windows_sys::Win32::{
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        },
        UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowRect, SetWindowPos, ShowWindow, SWP_NOACTIVATE,
            SWP_NOZORDER, SW_RESTORE,
        },
    };

    pub fn apply(_app: &AppHandle, action: &str) -> Result<(), String> {
        unsafe {
            let window = GetForegroundWindow();
            if window.is_null() {
                return Err("操作対象のウィンドウが見つかりません".into());
            }
            let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);
            let mut info = MONITORINFO {
                cbSize: size_of::<MONITORINFO>() as u32,
                rcMonitor: Default::default(),
                rcWork: Default::default(),
                dwFlags: 0,
            };
            if GetMonitorInfoW(monitor, &mut info) == 0 {
                return Err("ディスプレイの作業領域を取得できません".into());
            }
            let work = info.rcWork;
            let area = Rect {
                x: work.left,
                y: work.top,
                width: work.right - work.left,
                height: work.bottom - work.top,
            };
            // 最大化中は元の大きさに戻してから測る。
            ShowWindow(window, SW_RESTORE);
            let target = if action == CENTER {
                let mut frame = Default::default();
                if GetWindowRect(window, &mut frame) == 0 {
                    return Err("ウィンドウの大きさを取得できません".into());
                }
                centered(frame.right - frame.left, frame.bottom - frame.top, area)
            } else {
                layout_rect(action, area)?
            };
            if SetWindowPos(
                window,
                std::ptr::null_mut(),
                target.x,
                target.y,
                target.width,
                target.height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            ) == 0
            {
                return Err("ウィンドウの位置とサイズを変更できません".into());
            }
        }
        Ok(())
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
    use tauri::AppHandle;
    pub fn apply(_app: &AppHandle, _action: &str) -> Result<(), String> {
        Err("このOSではウィンドウ整形を利用できません".into())
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
}
