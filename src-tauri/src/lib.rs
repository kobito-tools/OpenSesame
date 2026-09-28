mod icons;
mod input;
mod keys;
mod model;
mod storage;
#[cfg(target_os = "macos")]
mod tap_macos;
mod window_layout;

use icons::{cache_system_icon, copy_custom_icon};
use input::{ActivationKeys, InputError};
use keys::{normalize_modifier, ASSIGNABLE_KEYS};
use model::{
    LauncherConfig, LauncherItem, RuntimeInfo, TargetIcon, TargetPath, TargetType, KEYBOARD_LAYOUTS,
    LANGUAGES,
};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, State, WindowEvent,
};
use uuid::Uuid;

struct AppState {
    config: Arc<RwLock<LauncherConfig>>,
    config_path: PathBuf,
    data_dir: PathBuf,
    activation: ActivationKeys,
    input_error: InputError,
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<LauncherConfig, String> {
    state
        .config
        .read()
        .map(|config| config.clone())
        .map_err(|_| "設定のロックを取得できません".to_string())
}

#[tauri::command]
fn save_config(
    config: LauncherConfig,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<LauncherConfig, String> {
    {
        let current = state
            .config
            .read()
            .map_err(|_| "設定のロックを取得できません".to_string())?;
        validate_config(&config, &current)?;
    }
    storage::save(&state.config_path, &config)?;
    *state
        .config
        .write()
        .map_err(|_| "設定のロックを取得できません".to_string())? = config.clone();
    // 起動キーはイベントごとの解析を避けるため、保存時に正規名の集合へ反映する。
    *state
        .activation
        .write()
        .map_err(|_| "起動キーのロックを取得できません".to_string())? =
        activation_set(&config);
    app.emit("config-changed", &config)
        .map_err(|error| format!("画面を更新できません: {error}"))?;
    Ok(config)
}

/// `key` を渡すとそのキーに割り当てる。省略時は空いているキーを自動で選ぶ。
#[tauri::command]
async fn select_target(
    target_type: String,
    key: Option<String>,
    state: State<'_, AppState>,
) -> Result<Option<LauncherItem>, String> {
    let (path, item_type) = if target_type == "folder" {
        let selection = rfd::AsyncFileDialog::new().pick_folder().await;
        let Some(selection) = selection else {
            return Ok(None);
        };
        (selection.path().to_path_buf(), TargetType::Folder)
    } else {
        let selection = rfd::AsyncFileDialog::new().pick_file().await;
        let Some(selection) = selection else {
            return Ok(None);
        };
        (selection.path().to_path_buf(), TargetType::Application)
    };

    let id = Uuid::new_v4().to_string();
    let icon_path = state
        .data_dir
        .join("icons/system")
        .join(format!("{id}.png"));
    let icon_source = path.clone();
    let icon_destination = icon_path.clone();
    let cached = tauri::async_runtime::spawn_blocking(move || {
        cache_system_icon(&icon_source, &icon_destination)
    })
    .await
    .ok()
    .and_then(Result::ok)
    .map(|_| icon_path.to_string_lossy().into_owned());

    let name = display_name(&path);
    let key = match key {
        Some(requested) => {
            validate_key(&requested)?;
            requested
        }
        None => next_available_key(&state.config)?,
    };
    Ok(Some(LauncherItem {
        id,
        target_type: item_type,
        name,
        key,
        target: TargetPath {
            kind: "path".into(),
            value: path.to_string_lossy().into_owned(),
        },
        icon: TargetIcon {
            kind: "system".into(),
            value: cached,
        },
    }))
}

#[tauri::command]
async fn select_custom_icon(
    item_id: String,
    state: State<'_, AppState>,
) -> Result<Option<LauncherItem>, String> {
    let selection = rfd::AsyncFileDialog::new()
        .add_filter("PNG image", &["png"])
        .pick_file()
        .await;
    let Some(selection) = selection else {
        return Ok(None);
    };

    let destination = state
        .data_dir
        .join("icons/custom")
        .join(format!("{item_id}.png"));
    let source = selection.path().to_path_buf();
    let destination_copy = destination.clone();
    tauri::async_runtime::spawn_blocking(move || copy_custom_icon(&source, &destination_copy))
        .await
        .map_err(|error| format!("アイコン処理が中断されました: {error}"))??;

    let config = state
        .config
        .read()
        .map_err(|_| "設定のロックを取得できません".to_string())?;
    let mut item = config
        .items
        .iter()
        .find(|item| item.id == item_id)
        .cloned()
        .ok_or_else(|| "対象の登録項目が見つかりません".to_string())?;
    item.icon = TargetIcon {
        kind: "custom".into(),
        value: Some(destination.to_string_lossy().into_owned()),
    };
    Ok(Some(item))
}

/// 権限の付与先を自力で探すのは難しいので、該当のシステム設定を直接開く。
#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let target = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";
    #[cfg(not(target_os = "macos"))]
    let target = "ms-settings:privacy";
    open::that(target).map_err(|failure| error("open-settings-failed", &failure.to_string()))
}

#[tauri::command]
fn runtime_info(state: State<'_, AppState>) -> RuntimeInfo {
    #[cfg(target_os = "windows")]
    let platform = "windows";
    #[cfg(not(target_os = "windows"))]
    let platform = "macos";

    RuntimeInfo {
        platform,
        config_path: state.config_path.to_string_lossy().into_owned(),
        input_error: state
            .input_error
            .lock()
            .ok()
            .and_then(|error| error.clone()),
    }
}

/// 表示言語を切り替えられるよう、UIに出る失敗はコードで返してフロント側で翻訳する。
fn error(code: &str, argument: &str) -> String {
    if argument.is_empty() {
        format!("applauncher-error:{code}")
    } else {
        format!("applauncher-error:{code}:{argument}")
    }
}

/// 対象の存在確認は新規登録やパス変更のときだけ行う。
/// 登録済みの対象が後から消えても、名前やキーの変更・削除はできるようにするため。
fn validate_config(config: &LauncherConfig, previous: &LauncherConfig) -> Result<(), String> {
    let mut keys = HashSet::new();
    if !LANGUAGES.contains(&config.language.as_str()) {
        return Err(error("invalid-language", ""));
    }
    if !KEYBOARD_LAYOUTS.contains(&config.keyboard_layout.as_str()) {
        return Err(error("invalid-layout", ""));
    }
    validate_activation(&config.activation.macos)?;
    validate_activation(&config.activation.windows)?;
    for action in &config.window_actions {
        if let Some(key) = &action.key {
            validate_key(key)?;
            if !keys.insert(key.as_str()) {
                return Err(error("duplicate-key", key));
            }
        }
    }
    for item in &config.items {
        if item.name.trim().is_empty() {
            return Err(error("empty-name", ""));
        }
        validate_key(&item.key)?;
        if !keys.insert(item.key.as_str()) {
            return Err(error("duplicate-key", &item.key));
        }
        let already_registered = previous
            .items
            .iter()
            .any(|saved| saved.id == item.id && saved.target.value == item.target.value);
        if !already_registered && !Path::new(&item.target.value).exists() {
            return Err(error("missing-target", &item.target.value));
        }
    }
    Ok(())
}

/// 設定ファイルの表記ゆれを吸収し、フックが比較する正規名の集合にする。
fn activation_set(config: &LauncherConfig) -> HashSet<String> {
    config
        .activation
        .current()
        .iter()
        .filter_map(|value| normalize_modifier(value))
        .map(str::to_string)
        .collect()
}

fn validate_key(key: &str) -> Result<(), String> {
    if key == keys::RESERVED_KEY || key == "Dot" {
        return Err(error("reserved-key", ""));
    }
    if !ASSIGNABLE_KEYS.contains(&key) {
        return Err(error("unassignable-key", key));
    }
    Ok(())
}

fn validate_activation(values: &[String]) -> Result<(), String> {
    if values.is_empty() {
        return Err(error("activation-empty", ""));
    }
    if values.len() > 3 {
        return Err(error("activation-too-many", ""));
    }
    let mut seen = HashSet::new();
    for value in values {
        let normalized =
            normalize_modifier(value).ok_or_else(|| error("activation-unknown", value))?;
        if !seen.insert(normalized) {
            return Err(error("activation-duplicate", normalized));
        }
    }
    Ok(())
}

fn next_available_key(config: &Arc<RwLock<LauncherConfig>>) -> Result<String, String> {
    const CANDIDATES: &[&str] = &[
        "KeyA", "KeyS", "KeyD", "KeyF", "KeyG", "KeyH", "KeyJ", "KeyK", "KeyL", "KeyQ", "KeyW",
        "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP", "KeyZ", "KeyX", "KeyC",
        "KeyV", "KeyB", "KeyN", "KeyM", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6",
        "Digit7", "Digit8", "Digit9", "Digit0",
    ];
    let current = config
        .read()
        .map_err(|_| "設定のロックを取得できません".to_string())?;
    CANDIDATES
        .iter()
        .find(|candidate| {
            current.items.iter().all(|item| item.key != **candidate)
                && current
                    .window_actions
                    .iter()
                    .all(|action| action.key.as_deref() != Some(**candidate))
        })
        .map(|value| (*value).to_string())
        .ok_or_else(|| "登録可能なキーが残っていません".to_string())
}

fn display_name(path: &Path) -> String {
    path.file_stem()
        .or_else(|| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

/// 透過ウィンドウのままではCSSのbackdrop-filterがデスクトップに届かず、
/// 壁紙が派手なときに背景と同化してしまう。OS側で実際にぼかす。
#[cfg(target_os = "macos")]
fn apply_vibrancy_to_windows(app: &tauri::AppHandle) {
    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
    if let Some(window) = app.get_webview_window("popup") {
        let _ = apply_vibrancy(
            &window,
            NSVisualEffectMaterial::HudWindow,
            Some(NSVisualEffectState::Active),
            Some(POPUP_CORNER_RADIUS),
        );
    }
    if let Some(window) = app.get_webview_window("settings") {
        // 角丸はウィンドウ装飾側が持つので指定しない。
        let _ = apply_vibrancy(
            &window,
            NSVisualEffectMaterial::WindowBackground,
            Some(NSVisualEffectState::Active),
            None,
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn apply_vibrancy_to_windows(_app: &tauri::AppHandle) {}

/// 他のアプリがフルスクリーンのとき、通常のウィンドウはそのSpaceに入れず
/// ポップアップが裏に隠れる。全Spaceに参加させ、フルスクリーンの補助ウィンドウとして
/// メニューバーより上に重ねる。
#[cfg(target_os = "macos")]
pub(crate) fn float_popup_over_fullscreen(window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSPopUpMenuWindowLevel, NSWindow, NSWindowCollectionBehavior};
    let Ok(pointer) = window.ns_window() else {
        return;
    };
    // ns_windowはTauriが保持するNSWindowを指し、ここはメインスレッドで呼ばれる。
    let ns_window = unsafe { &*pointer.cast::<NSWindow>() };
    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    ns_window.setLevel(NSPopUpMenuWindowLevel);
}

/// macOS 10.14以降、Dockに出る通常アプリのウィンドウは他アプリのフルスクリーンの上に
/// 出られない。設定画面を閉じている間はDockから外し、常駐のアクセサリとして振る舞う。
#[cfg(target_os = "macos")]
fn set_dock_visible(app: &tauri::AppHandle, visible: bool) {
    let policy = if visible {
        tauri::ActivationPolicy::Regular
    } else {
        tauri::ActivationPolicy::Accessory
    };
    let _ = app.set_activation_policy(policy);
}

#[cfg(not(target_os = "macos"))]
fn set_dock_visible(_app: &tauri::AppHandle, _visible: bool) {}

/// 設定画面を閉じても常駐し続けるので、終了と再表示の入口をトレイに置く。
/// Windowsではこれが無いとタスクマネージャー以外で終了できない。
fn build_tray(app: &tauri::App, language: &str) -> tauri::Result<()> {
    let (open_label, quit_label) = if language == "en" {
        ("Open Settings", "Quit OpenSesame!")
    } else {
        ("設定を開く", "OpenSesame! を終了")
    };
    let open = MenuItem::with_id(app, "open-settings", open_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("OpenSesame!")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open-settings" => show_settings_window(app),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

pub(crate) fn show_settings_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        set_dock_visible(app, true);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// CSS側の角丸と揃える。
#[cfg(target_os = "macos")]
const POPUP_CORNER_RADIUS: f64 = 22.0;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_local_data_dir()
                .map_err(|error| format!("ユーザーデータ領域を取得できません: {error}"))?;
            fs::create_dir_all(&data_dir)?;
            let config_path = data_dir.join("config.json");
            let loaded = storage::load_or_create(&config_path)?;
            let activation: ActivationKeys = Arc::new(RwLock::new(activation_set(&loaded)));
            let input_error: InputError = Arc::new(Mutex::new(None));
            let language = loaded.language.clone();
            let config = Arc::new(RwLock::new(loaded));
            app.manage(AppState {
                config: config.clone(),
                config_path,
                data_dir,
                activation: activation.clone(),
                input_error: input_error.clone(),
            });
            input::start(app.handle().clone(), config, activation, input_error);
            apply_vibrancy_to_windows(app.handle());
            // トレイが作れない環境でも本体の機能は動くので、起動は止めない。
            let _ = build_tray(app, &language);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                if window.label() == "settings" {
                    set_dock_visible(window.app_handle(), false);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            select_target,
            select_custom_icon,
            runtime_info,
            open_accessibility_settings
        ])
        .build(tauri::generate_context!())
        .expect("error while building OpenSesame!")
        .run(|_app, _event| {
            // macOSでDockのアイコンを押したら、隠した設定画面を出し直す。
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = _event {
                show_settings_window(_app);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_activation_uses_left_side() {
        let config = LauncherConfig::default();
        assert_eq!(config.activation.macos, ["ControlLeft", "AltLeft"]);
        assert_eq!(config.activation.windows, ["ControlLeft", "AltLeft"]);
    }

    #[test]
    fn errors_carry_a_translatable_code() {
        let mut config = LauncherConfig::default();
        config.activation.macos.clear();
        assert_eq!(
            validate_config(&config, &LauncherConfig::default()).unwrap_err(),
            "applauncher-error:activation-empty"
        );
        let mut config = LauncherConfig::default();
        config.keyboard_layout = "grouped".into();
        assert_eq!(
            validate_config(&config, &LauncherConfig::default()).unwrap_err(),
            "applauncher-error:invalid-layout"
        );
    }

    #[test]
    fn rejects_unknown_language() {
        let mut config = LauncherConfig::default();
        config.language = "fr".into();
        assert!(validate_config(&config, &LauncherConfig::default()).is_err());
    }

    #[test]
    fn activation_must_not_be_empty() {
        let mut config = LauncherConfig::default();
        config.activation.macos.clear();
        assert!(validate_config(&config, &LauncherConfig::default()).is_err());
    }

    #[test]
    fn symbols_and_arrows_are_assignable() {
        for key in ["Semicolon", "Enter", "ArrowRight", "IntlYen", "Backquote"] {
            assert!(validate_key(key).is_ok(), "{key} should be assignable");
        }
    }

    #[test]
    fn period_is_reserved() {
        let mut config = LauncherConfig::default();
        config.items.push(LauncherItem {
            id: "test".into(),
            target_type: TargetType::Folder,
            name: "Test".into(),
            key: keys::RESERVED_KEY.into(),
            target: TargetPath {
                kind: "path".into(),
                value: "/".into(),
            },
            icon: TargetIcon {
                kind: "system".into(),
                value: None,
            },
        });
        assert!(validate_config(&config, &LauncherConfig::default()).is_err());
    }

    fn item_at(id: &str, key: &str, path: &str) -> LauncherItem {
        LauncherItem {
            id: id.into(),
            target_type: TargetType::Folder,
            name: id.into(),
            key: key.into(),
            target: TargetPath {
                kind: "path".into(),
                value: path.into(),
            },
            icon: TargetIcon {
                kind: "system".into(),
                value: None,
            },
        }
    }

    #[test]
    fn registered_items_stay_editable_after_their_target_disappears() {
        let mut saved = LauncherConfig::default();
        saved.items.push(item_at("gone-1", "KeyA", "/nonexistent/opensesame-1"));
        saved.items.push(item_at("gone-2", "KeyS", "/nonexistent/opensesame-2"));
        let mut edited = saved.clone();
        edited.language = "en".into();
        edited.items[0].name = "Renamed".into();
        edited.items[0].key = "KeyD".into();
        assert!(validate_config(&edited, &saved).is_ok());
        edited.items.remove(1);
        assert!(validate_config(&edited, &saved).is_ok());
    }

    #[test]
    fn new_items_must_point_to_an_existing_target() {
        let saved = LauncherConfig::default();
        let mut edited = saved.clone();
        edited.items.push(item_at("new", "KeyA", "/nonexistent/opensesame"));
        assert_eq!(
            validate_config(&edited, &saved).unwrap_err(),
            "applauncher-error:missing-target:/nonexistent/opensesame"
        );
    }
}
