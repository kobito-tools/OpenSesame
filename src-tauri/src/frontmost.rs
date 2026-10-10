//! 割り当てたアプリが既に前面にあるとき、もう一度押したら隠す。
//!
//! 前面のアプリの実体（macOSは .app、Windowsは実行ファイル）を登録したパスと比べる。
//! Windowsでショートカット(.lnk)を登録した場合は一致しないので、通常どおり開く。

use std::{fs, path::Path};
use tauri::AppHandle;

/// 隠したら真。前面に無い、または判定できないときは偽を返し、呼び出し側が開く。
/// バックグラウンドスレッドから呼ぶこと。
pub fn hide_if_frontmost(app: &AppHandle, target: &str) -> bool {
    platform::hide_if_frontmost(app, target)
}

/// シンボリックリンクや表記ゆれを吸収して同じ実体かを比べる。
fn same_path(left: &Path, right: &Path) -> bool {
    let (Ok(left), Ok(right)) = (fs::canonicalize(left), fs::canonicalize(right)) else {
        return false;
    };
    // Windowsのパスは大文字と小文字を区別しない。
    if cfg!(windows) {
        left.to_string_lossy().to_lowercase() == right.to_string_lossy().to_lowercase()
    } else {
        left == right
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::same_path;
    use objc2_app_kit::NSWorkspace;
    use std::{path::Path, sync::mpsc, time::Duration};
    use tauri::AppHandle;

    /// NSWorkspaceの前面アプリはメインスレッドのランループで更新されるので、照会もそこで行う。
    pub fn hide_if_frontmost(app: &AppHandle, target: &str) -> bool {
        let (sender, receiver) = mpsc::channel();
        let target = target.to_string();
        if app
            .run_on_main_thread(move || {
                let _ = sender.send(hide_on_main_thread(&target));
            })
            .is_err()
        {
            return false;
        }
        receiver
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or(false)
    }

    fn hide_on_main_thread(target: &str) -> bool {
        let Some(front) = NSWorkspace::sharedWorkspace().frontmostApplication() else {
            return false;
        };
        let Some(path) = front.bundleURL().and_then(|url| url.path()) else {
            return false;
        };
        if !same_path(Path::new(&path.to_string()), Path::new(target)) {
            return false;
        }
        front.hide()
    }
}

#[cfg(windows)]
mod platform {
    use super::same_path;
    use std::path::Path;
    use tauri::AppHandle;
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId, ShowWindow, SW_MINIMIZE,
        },
    };

    /// Windowsには「アプリを隠す」が無いので、前面のウィンドウを最小化する。
    pub fn hide_if_frontmost(_app: &AppHandle, target: &str) -> bool {
        unsafe {
            let window = GetForegroundWindow();
            if window.is_null() {
                return false;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(window, &mut pid);
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if process.is_null() {
                return false;
            }
            let mut buffer = [0u16; 1024];
            let mut size = buffer.len() as u32;
            let queried = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size);
            CloseHandle(process);
            if queried == 0 {
                return false;
            }
            let path = String::from_utf16_lossy(&buffer[..size as usize]);
            if !same_path(Path::new(&path), Path::new(target)) {
                return false;
            }
            ShowWindow(window, SW_MINIMIZE);
            true
        }
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
    use tauri::AppHandle;

    pub fn hide_if_frontmost(_app: &AppHandle, _target: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_the_resolved_paths() {
        let dir = std::env::temp_dir();
        assert!(same_path(&dir, &dir.join(".")));
        assert!(!same_path(&dir, &dir.join("opensesame-nonexistent")));
    }
}
