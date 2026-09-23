use tauri::AppHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
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
    use super::{layout_rect, Rect};
    use std::{process::Command, sync::mpsc, time::Duration};
    use tauri::AppHandle;

    /// バックグラウンドスレッドから呼ぶこと。
    /// ディスプレイの照会はAppKitに触れるのでメインスレッドへ回し、
    /// 時間のかかるosascriptは呼び出し元のスレッドで動かす。
    pub fn apply(app: &AppHandle, action: &str) -> Result<(), String> {
        let area = monitor_area(app)?;
        let target = layout_rect(action, area)?;
        let script = format!(
            "tell application \"System Events\" to tell first application process whose frontmost is true to tell front window to set {{position, size}} to {{{{{}, {}}}, {{{}, {}}}}}",
            target.x, target.y, target.width, target.height
        );
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
    use super::{layout_rect, Rect};
    use std::mem::size_of;
    use tauri::AppHandle;
    use windows_sys::Win32::{
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        },
        UI::WindowsAndMessaging::{
            GetForegroundWindow, SetWindowPos, ShowWindow, SWP_NOACTIVATE, SWP_NOZORDER, SW_RESTORE,
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
            let target = layout_rect(
                action,
                Rect {
                    x: work.left,
                    y: work.top,
                    width: work.right - work.left,
                    height: work.bottom - work.top,
                },
            )?;
            ShowWindow(window, SW_RESTORE);
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
}
