//! 同じアプリが複数動かないようにする。
//!
//! 新しい版のインストーラーやdmgから直接起動すると、旧版と新版が並んで動き、
//! キーフックが二重になる。バージョンの新しい方を残し、同じ版どうしなら
//! 後から起動した方をユーザーの意図とみなして引き継ぐ。

use std::time::Duration;

/// 終了を待つ時間。macOSでは通常の終了要求に応じなければ、その後に強制終了する。
const QUIT_TIMEOUT: Duration = Duration::from_secs(3);

type Version = [u64; 3];

/// 他のインスタンスより古ければ自分が身を引き、そうでなければ他を終了させる。
/// キーフックを張る前に呼ぶこと。
pub fn take_over_or_exit() {
    let own = parse_version(env!("CARGO_PKG_VERSION"));
    let others = platform::others();
    if others.is_empty() {
        return;
    }
    if others
        .iter()
        .any(|other| is_newer(other.version, own))
    {
        std::process::exit(0);
    }
    platform::stop(others);
}

/// 版が読めない相手は古いとみなす。
fn is_newer(other: Option<Version>, own: Option<Version>) -> bool {
    match (other, own) {
        (Some(other), Some(own)) => other > own,
        (Some(_), None) => true,
        (None, _) => false,
    }
}

/// "1.2.3" や "1.2.3-beta" の数字の部分だけを見る。
fn parse_version(text: &str) -> Option<Version> {
    let core = text.split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|part| part.parse::<u64>().ok());
    Some([parts.next()??, parts.next().flatten().unwrap_or(0), parts.next().flatten().unwrap_or(0)])
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{parse_version, Version, QUIT_TIMEOUT};
    use objc2::rc::Retained;
    use objc2_app_kit::NSRunningApplication;
    use objc2_foundation::{NSBundle, NSString};
    use std::{
        thread,
        time::{Duration, Instant},
    };

    pub struct Other {
        pub version: Option<Version>,
        app: Retained<NSRunningApplication>,
    }

    extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }

    /// NSRunningApplicationの終了状態はランループが回るまで更新されないので、
    /// 起動処理の途中でも確かめられるよう、プロセスの有無を直接見る。
    fn is_alive(pid: i32) -> bool {
        unsafe { kill(pid, 0) == 0 }
    }

    pub fn others() -> Vec<Other> {
        // `tauri dev` のようにバンドル外で動くときは識別子が無いので、何もしない。
        let Some(identifier) = NSBundle::mainBundle().bundleIdentifier() else {
            return Vec::new();
        };
        let own_pid = std::process::id() as i32;
        NSRunningApplication::runningApplicationsWithBundleIdentifier(&identifier)
            .iter()
            .filter(|app| app.processIdentifier() != own_pid)
            .map(|app| Other {
                version: bundle_version(&app),
                app,
            })
            .collect()
    }

    fn bundle_version(app: &NSRunningApplication) -> Option<Version> {
        let url = app.bundleURL()?;
        let bundle = NSBundle::bundleWithURL(&url)?;
        let value = bundle
            .objectForInfoDictionaryKey(&NSString::from_str("CFBundleShortVersionString"))?;
        let text = value.downcast::<NSString>().ok()?;
        parse_version(&text.to_string())
    }

    pub fn stop(others: Vec<Other>) {
        for other in &others {
            other.app.terminate();
        }
        let deadline = Instant::now() + QUIT_TIMEOUT;
        while Instant::now() < deadline
            && others
                .iter()
                .any(|other| is_alive(other.app.processIdentifier()))
        {
            thread::sleep(Duration::from_millis(100));
        }
        // 応答なしになった旧版は終了要求を処理できないので、強制的に止める。
        for other in &others {
            if is_alive(other.app.processIdentifier()) {
                other.app.forceTerminate();
            }
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::{Version, QUIT_TIMEOUT};
    use std::{ffi::c_void, os::windows::ffi::OsStrExt, path::Path, ptr};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
        Storage::FileSystem::{
            GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
        },
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
                PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
            },
        },
    };

    pub struct Other {
        pub version: Option<Version>,
        process: HANDLE,
    }

    impl Drop for Other {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.process) };
        }
    }

    fn wide(text: &std::ffi::OsStr) -> Vec<u16> {
        text.encode_wide().chain(Some(0)).collect()
    }

    /// 自分と同じ実行ファイル名のプロセスを探す。インストール先が違っても見つけられる。
    pub fn others() -> Vec<Other> {
        let Ok(own_path) = std::env::current_exe() else {
            return Vec::new();
        };
        let Some(own_name) = own_path.file_name().map(|name| name.to_string_lossy().to_lowercase())
        else {
            return Vec::new();
        };
        let own_pid = std::process::id();
        let mut found = Vec::new();
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return found;
            }
            let mut entry = PROCESSENTRY32W {
                dwSize: size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut more = Process32FirstW(snapshot, &mut entry) != 0;
            while more {
                let length = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(0);
                let name = String::from_utf16_lossy(&entry.szExeFile[..length]).to_lowercase();
                if name == own_name && entry.th32ProcessID != own_pid {
                    if let Some(other) = open(entry.th32ProcessID) {
                        found.push(other);
                    }
                }
                more = Process32NextW(snapshot, &mut entry) != 0;
            }
            CloseHandle(snapshot);
        }
        found
    }

    unsafe fn open(pid: u32) -> Option<Other> {
        let process = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | PROCESS_SYNCHRONIZE,
            0,
            pid,
        );
        if process.is_null() {
            return None;
        }
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        let version = if QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size) != 0
        {
            let path = String::from_utf16_lossy(&buffer[..size as usize]);
            file_version(Path::new(&path))
        } else {
            None
        };
        Some(Other { version, process })
    }

    /// 実行ファイルに埋め込まれたバージョン情報を読む。Tauriはアプリの版をここへ書き込む。
    fn file_version(path: &Path) -> Option<Version> {
        let name = wide(path.as_os_str());
        unsafe {
            let size = GetFileVersionInfoSizeW(name.as_ptr(), ptr::null_mut());
            if size == 0 {
                return None;
            }
            let mut data = vec![0u8; size as usize];
            if GetFileVersionInfoW(name.as_ptr(), 0, size, data.as_mut_ptr().cast()) == 0 {
                return None;
            }
            let root = wide(std::ffi::OsStr::new("\\"));
            let mut info: *mut c_void = ptr::null_mut();
            let mut length = 0u32;
            if VerQueryValueW(data.as_ptr().cast(), root.as_ptr(), &mut info, &mut length) == 0
                || info.is_null()
                || (length as usize) < size_of::<VS_FIXEDFILEINFO>()
            {
                return None;
            }
            let info = &*(info as *const VS_FIXEDFILEINFO);
            Some([
                (info.dwFileVersionMS >> 16) as u64,
                (info.dwFileVersionMS & 0xFFFF) as u64,
                (info.dwFileVersionLS >> 16) as u64,
            ])
        }
    }

    /// Windowsのアプリには外から穏やかに終了を頼む手段が無いので、そのまま止める。
    /// 設定は保存のたびに書き出しているので失われない。
    pub fn stop(others: Vec<Other>) {
        for other in &others {
            unsafe {
                TerminateProcess(other.process, 0);
            }
        }
        for other in &others {
            unsafe {
                WaitForSingleObject(other.process, QUIT_TIMEOUT.as_millis() as u32);
            }
        }
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
    use super::Version;

    pub struct Other {
        pub version: Option<Version>,
    }

    pub fn others() -> Vec<Other> {
        Vec::new()
    }

    pub fn stop(_others: Vec<Other>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_and_prerelease_versions() {
        assert_eq!(parse_version("0.1.2"), Some([0, 1, 2]));
        assert_eq!(parse_version("1.2.3-beta.1"), Some([1, 2, 3]));
        assert_eq!(parse_version("2"), Some([2, 0, 0]));
        assert_eq!(parse_version("abc"), None);
    }

    #[test]
    fn only_a_strictly_newer_instance_wins() {
        let own = Some([0, 1, 3]);
        assert!(is_newer(Some([0, 2, 0]), own));
        assert!(!is_newer(Some([0, 1, 3]), own));
        assert!(!is_newer(Some([0, 1, 2]), own));
        assert!(!is_newer(None, own));
    }
}
