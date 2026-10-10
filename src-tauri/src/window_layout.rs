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
/// macOSは同じ計算をAppleScript側で行うので、Windowsとテストだけが使う。
#[cfg_attr(not(windows), allow(dead_code))]
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

pub fn apply(app: &AppHandle, action: &str) -> Result<(), String> {
    platform::apply(app, action)
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{layout_rect, Rect, CENTER};
    use std::{process::Command, sync::mpsc, time::Duration};
    use tauri::AppHandle;

    /// バックグラウンドスレッドから呼ぶこと。
    /// ディスプレイの照会はAppKitに触れるのでメインスレッドへ回し、
    /// 時間のかかるosascriptは呼び出し元のスレッドで動かす。
    pub fn apply(app: &AppHandle, action: &str) -> Result<(), String> {
        let area = monitor_area(app)?;
        let script = if action == CENTER {
            center_script(area)
        } else {
            let target = layout_rect(action, area)?;
            format!(
                "tell application \"System Events\" to tell first application process whose frontmost is true to tell front window to set {{position, size}} to {{{{{}, {}}}, {{{}, {}}}}}",
                target.x, target.y, target.width, target.height
            )
        };
        let output = Command::new("/usr/bin/osascript")
            .args(["-e", &script])
            .output()
            .map_err(|error| format!("ウィンドウ整形を実行できません: {error}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// 大きさの取得と移動を1回のosascriptで済ませ、待ち時間を増やさない。
    /// 計算は `super::centered` と同じ。
    fn center_script(area: Rect) -> String {
        format!(
            "tell application \"System Events\"
                tell first application process whose frontmost is true
                    tell front window
                        set {{w, h}} to size
                        set x to {x} + ({width} - w) div 2
                        if x < {x} then set x to {x}
                        set y to {y} + ({height} - h) div 2
                        if y < {y} then set y to {y}
                        set position to {{x, y}}
                    end tell
                end tell
            end tell",
            x = area.x,
            y = area.y,
            width = area.width,
            height = area.height
        )
    }

    fn monitor_area(app: &AppHandle) -> Result<Rect, String> {
        let (sender, receiver) = mpsc::channel();
        let handle = app.clone();
        app.run_on_main_thread(move || {
            let _ = sender.send(current_area(&handle));
        })
        .map_err(|error| error.to_string())?;
        receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "対象ディスプレイを取得できません".to_string())?
    }

    fn current_area(app: &AppHandle) -> Result<Rect, String> {
        let cursor = app.cursor_position().map_err(|error| error.to_string())?;
        let monitor = app
            .monitor_from_point(cursor.x, cursor.y)
            .map_err(|error| error.to_string())?
            .or_else(|| app.primary_monitor().ok().flatten())
            .ok_or_else(|| "対象ディスプレイを取得できません".to_string())?;
        let scale = monitor.scale_factor();
        let position = monitor.position();
        let size = monitor.size();

        // Accessibility window coordinates use logical points. Reserve the menu-bar area.
        Ok(Rect {
            x: (position.x as f64 / scale).round() as i32,
            y: (position.y as f64 / scale).round() as i32 + 25,
            width: (size.width as f64 / scale).round() as i32,
            height: (size.height as f64 / scale).round() as i32 - 25,
        })
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
}
