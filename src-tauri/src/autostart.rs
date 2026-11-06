//! ログイン時に OpenSesame! を起動するかどうか。
//!
//! 入り切りの状態は設定ファイルには持たず、OS に登録されているかどうか
//! （macOS は `~/Library/LaunchAgents` の plist、Windows はスタートアップのレジストリ）を
//! そのまま正とする。設定画面の表示と実際の登録が食い違わないようにするため。

use tauri::{plugin::TauriPlugin, AppHandle, Runtime};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

/// ログイン時の起動であることを本体に知らせる引数。設定画面を出さずに常駐だけする。
pub const AUTOSTART_FLAG: &str = "--autostart";

/// macOS では LaunchAgent の名前（plist のファイル名と Label）になるので、バンドルIDにそろえる。
/// Windows ではスタートアップ アプリの一覧に出る名前になるので、アプリ名のままにする。
#[cfg(target_os = "macos")]
const LAUNCHER_NAME: &str = "io.github.kobito-tools.opensesame";
#[cfg(not(target_os = "macos"))]
const LAUNCHER_NAME: &str = "OpenSesame!";

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_autostart::Builder::new()
        .macos_launcher(MacosLauncher::LaunchAgent)
        .app_name(LAUNCHER_NAME)
        .arg(AUTOSTART_FLAG)
        .build()
}

/// ログイン時に起動されたか。
pub fn launched_at_login() -> bool {
    std::env::args().skip(1).any(|arg| arg == AUTOSTART_FLAG)
}

/// 登録済みなら、いま動いている実行ファイルの場所で書き直す。
/// `setup.sh` でアプリケーションフォルダへ移したあとも、古い場所を指したままにしないため。
/// 開発中のビルド（`tauri dev`）では書き直さない（登録が開発用の実行ファイルを指してしまう）。
pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    if cfg!(debug_assertions) {
        return;
    }
    let manager = app.autolaunch();
    if manager.is_enabled().unwrap_or(false) {
        if let Err(error) = manager.enable() {
            eprintln!("ログイン時の起動の登録を書き直せません: {error}");
        }
    }
}

#[tauri::command]
pub fn get_launch_at_login(app: AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|failure| crate::error("autostart-failed", &failure.to_string()))
}

/// 登録・解除したあと、実際の状態を読み直して返す。
#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|failure| crate::error("autostart-failed", &failure.to_string()))?;
    manager
        .is_enabled()
        .map_err(|failure| crate::error("autostart-failed", &failure.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_name_matches_the_bundle_identifier_on_macos() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        if cfg!(target_os = "macos") {
            assert_eq!(config["identifier"], LAUNCHER_NAME);
        } else {
            assert_eq!(config["productName"], LAUNCHER_NAME);
        }
    }
}
