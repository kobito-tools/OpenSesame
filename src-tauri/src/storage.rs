use crate::{
    keys::normalize_modifier,
    model::{
        default_keyboard_layout, default_window_actions, ActivationConfig, LauncherConfig,
        CONFIG_VERSION, KEYBOARD_LAYOUTS,
    },
};
use serde_json::{Map, Value};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn load_or_create(path: &Path) -> Result<LauncherConfig, String> {
    if !path.exists() {
        let config = LauncherConfig::default();
        save(path, &config)?;
        return Ok(config);
    }

    match load(path) {
        Ok(config) => Ok(config),
        Err(_) => {
            // 壊れた設定のままでは起動すらできなくなる。退避して初期設定で立ち上げる。
            let _ = fs::rename(path, broken_path(path));
            let config = LauncherConfig::default();
            save(path, &config)?;
            Ok(config)
        }
    }
}

fn load(path: &Path) -> Result<LauncherConfig, String> {
    let bytes = fs::read(path).map_err(|error| format!("設定の読み込みに失敗しました: {error}"))?;
    let mut raw: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("設定ファイルが不正です: {error}"))?;
    let stored_version = raw.get("version").and_then(Value::as_u64).unwrap_or(0) as u32;
    let migrated = stored_version < CONFIG_VERSION;
    if migrated {
        migrate(&mut raw);
    }
    let dropped = drop_removed_items(&mut raw);
    let mut config: LauncherConfig = serde_json::from_value(raw)
        .map_err(|error| format!("設定ファイルが不正です: {error}"))?;
    fill_window_actions(&mut config);
    if migrated {
        config.version = CONFIG_VERSION;
    }
    if migrated || dropped {
        save(path, &config)?;
    }
    Ok(config)
}

/// 旧バージョンの設定をそのまま読めるよう、JSONの段階で形を合わせる。
///
/// - v3: 起動キー名を `ControlLeft` / `AltLeft` 形式へ正規化
/// - v4: `settings_layout` と `popup.layout` を `keyboard_layout` へ統合
fn migrate(raw: &mut Value) {
    let Some(object) = raw.as_object_mut() else {
        return;
    };
    migrate_activation(object);
    migrate_keyboard_layout(object);
}

fn migrate_activation(object: &mut Map<String, Value>) {
    let defaults = ActivationConfig::default();
    let Some(activation) = object.get_mut("activation").and_then(Value::as_object_mut) else {
        return;
    };
    for (platform, fallback) in [("macos", &defaults.macos), ("windows", &defaults.windows)] {
        let normalized: Vec<Value> = activation
            .get(platform)
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .filter_map(normalize_modifier)
                    .map(|code| Value::String(code.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let values = if normalized.is_empty() {
            fallback.iter().cloned().map(Value::String).collect()
        } else {
            normalized
        };
        activation.insert(platform.to_string(), Value::Array(values));
    }
}

fn migrate_keyboard_layout(object: &mut Map<String, Value>) {
    let legacy_popup_layout = object
        .get("popup")
        .and_then(Value::as_object)
        .and_then(|popup| popup.get("layout"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let legacy_settings_layout = object
        .get("settings_layout")
        .and_then(Value::as_str)
        .map(str::to_string);
    let layout = object
        .get("keyboard_layout")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or(legacy_settings_layout)
        .or(legacy_popup_layout)
        .filter(|value| KEYBOARD_LAYOUTS.contains(&value.as_str()))
        .unwrap_or_else(default_keyboard_layout);
    object.insert("keyboard_layout".into(), Value::String(layout));
    object.remove("settings_layout");
    if let Some(popup) = object.get_mut("popup").and_then(Value::as_object_mut) {
        popup.remove("layout");
        popup.remove("show_labels");
    }
}

/// 種類から外した「シリーズの操作」（`kobito-action`）の項目を捨てる。
/// 1件のために設定全体を壊れた扱いにせず、ほかの項目はそのまま読む。捨てたら `true`。
fn drop_removed_items(raw: &mut Value) -> bool {
    let Some(items) = raw.get_mut("items").and_then(Value::as_array_mut) else {
        return false;
    };
    let before = items.len();
    items.retain(|item| item.get("type").and_then(Value::as_str) != Some("kobito-action"));
    items.len() != before
}

/// 新しい版で増えたウィンドウ整形を、既存の割り当てを保ったまま組み込みの順に並べ直す。
/// 増えるたびに設定の版を上げずに済むよう、読み込みのたびに行う。
fn fill_window_actions(config: &mut LauncherConfig) {
    let mut saved = std::mem::take(&mut config.window_actions);
    for default in default_window_actions() {
        let binding = match saved.iter().position(|binding| binding.action == default.action) {
            Some(index) => saved.remove(index),
            None => default,
        };
        config.window_actions.push(binding);
    }
    // 組み込みに無い操作は捨てずに末尾へ残す。
    config.window_actions.append(&mut saved);
}

pub fn save(path: &Path, config: &LauncherConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("設定ディレクトリを作成できません: {error}"))?;
    }

    let bytes = serde_json::to_vec_pretty(config)
        .map_err(|error| format!("設定を変換できません: {error}"))?;
    let temporary = temporary_path(path);
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| format!("一時設定ファイルを作成できません: {error}"))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("設定を書き込めません: {error}"))?;

    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("旧設定を置換できません: {error}"))?;
    }

    fs::rename(&temporary, path).map_err(|error| format!("設定を確定できません: {error}"))
}

fn temporary_path(path: &Path) -> PathBuf {
    with_suffix(path, ".tmp")
}

/// 退避先は上書きしないよう時刻を付ける。
fn broken_path(path: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    with_suffix(path, &format!(".broken-{stamp}"))
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_from_a_broken_config() {
        let dir = std::env::temp_dir().join(format!("opensesame-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, b"{ not json").unwrap();
        let config = load_or_create(&path).unwrap();
        assert_eq!(config.version, CONFIG_VERSION);
        let backups = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".broken-"))
            .count();
        assert_eq!(backups, 1);
        fs::remove_dir_all(&dir).unwrap();
    }

    fn temp_config(contents: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("opensesame-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, contents).unwrap();
        (dir, path)
    }

    /// コマンドの種類を足す前（v4）の設定ファイルが、そのまま読めて退避もされないこと。
    #[test]
    fn reads_a_config_saved_before_commands_existed() {
        let (dir, path) = temp_config(
            r#"{
                "version": 4,
                "language": "en",
                "activation": { "macos": ["ControlLeft", "AltLeft"], "windows": ["ControlLeft", "AltLeft"] },
                "keyboard_layout": "keyboard-us",
                "popup": { "position": "cursor-monitor-center" },
                "window_actions": [],
                "items": [
                    { "id": "a", "type": "application", "name": "Mail", "key": "KeyM",
                      "target": { "kind": "path", "value": "/System/Applications/Mail.app" },
                      "icon": { "kind": "system" }, "hide_when_frontmost": true },
                    { "id": "f", "type": "folder", "name": "Home", "key": "KeyH", "shift": true,
                      "target": { "kind": "path", "value": "/Users" },
                      "icon": { "kind": "custom", "value": "/icons/f.png" } },
                    { "id": "w", "type": "website", "name": "docs.rs", "key": "KeyD",
                      "target": { "kind": "url", "value": "https://docs.rs" },
                      "icon": { "kind": "system" } }
                ]
            }"#,
        );
        let config = load_or_create(&path).unwrap();
        assert_eq!(config.language, "en");
        assert_eq!(config.items.len(), 3);
        assert!(config.items.iter().all(|item| item.target.working_dir.is_none()));
        assert!(config.items[0].hide_when_frontmost);
        assert!(config.items[1].shift);
        let names: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["config.json"]);
        // 作業フォルダの無い項目は、書き戻しても元と同じ形のまま。
        save(&path, &config).unwrap();
        assert!(!fs::read_to_string(&path).unwrap().contains("working_dir"));
        fs::remove_dir_all(&dir).unwrap();
    }

    /// v2 の設定ファイルも、ファイルから読んで現在の版へ移せること。
    #[test]
    fn reads_and_upgrades_a_v2_config_file() {
        let (dir, path) = temp_config(
            r#"{
                "version": 2,
                "activation": { "macos": ["LeftControl", "LeftOption"], "windows": ["LeftControl", "LeftAlt"] },
                "popup": { "position": "cursor-monitor-center", "layout": "keyboard-us" },
                "items": [
                    { "id": "f", "type": "folder", "name": "Root", "key": "KeyR",
                      "target": { "kind": "path", "value": "/" }, "icon": { "kind": "system" } }
                ]
            }"#,
        );
        let config = load_or_create(&path).unwrap();
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.keyboard_layout, "keyboard-us");
        assert_eq!(config.items[0].name, "Root");
        let saved: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["version"], CONFIG_VERSION);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn command_items_survive_a_save_and_load() {
        use crate::model::{LauncherItem, TargetIcon, TargetPath, TargetType};
        let (dir, path) = temp_config("{}");
        let mut config = LauncherConfig::default();
        config.items.push(LauncherItem {
            id: "c".into(),
            target_type: TargetType::Command,
            name: "Notes".into(),
            key: "KeyN".into(),
            shift: false,
            target: TargetPath {
                kind: "command".into(),
                value: "uv run notes.py".into(),
                working_dir: Some("/tmp".into()),
                ..Default::default()
            },
            icon: TargetIcon {
                kind: "system".into(),
                value: None,
            },
            hide_when_frontmost: false,
        });
        save(&path, &config).unwrap();
        let raw: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(raw["items"][0]["type"], "command");
        assert_eq!(raw["items"][0]["target"]["working_dir"], "/tmp");
        let loaded = load_or_create(&path).unwrap();
        assert!(matches!(loaded.items[0].target_type, TargetType::Command));
        assert_eq!(loaded.items[0].target.value, "uv run notes.py");
        assert_eq!(loaded.items[0].target.working_dir.as_deref(), Some("/tmp"));
        fs::remove_dir_all(&dir).unwrap();
    }

    /// コマンドを割り当てられるようになった版の設定ファイルが、
    /// そのまま読めて、書き戻しても項目の形が変わらないこと。
    #[test]
    fn reads_a_config_with_commands_unchanged() {
        let original = r#"{
            "version": 4,
            "language": "ja",
            "activation": { "macos": ["ControlLeft", "AltLeft"], "windows": ["ControlLeft", "AltLeft"] },
            "keyboard_layout": "keyboard-jis",
            "popup": { "position": "cursor-monitor-center" },
            "window_actions": [],
            "items": [
                { "id": "a", "type": "application", "name": "Mail", "key": "KeyM", "shift": false,
                  "target": { "kind": "path", "value": "/System/Applications/Mail.app" },
                  "icon": { "kind": "system" }, "hide_when_frontmost": true },
                { "id": "w", "type": "website", "name": "docs.rs", "key": "KeyD", "shift": true,
                  "target": { "kind": "url", "value": "https://docs.rs" },
                  "icon": { "kind": "system", "value": "/icons/w.png" }, "hide_when_frontmost": false },
                { "id": "c", "type": "command", "name": "Notes", "key": "KeyN", "shift": false,
                  "target": { "kind": "command", "value": "uv run notes.py", "working_dir": "/tmp" },
                  "icon": { "kind": "system" }, "hide_when_frontmost": false }
            ]
        }"#;
        let (dir, path) = temp_config(original);
        let config = load_or_create(&path).unwrap();
        assert_eq!(config.items.len(), 3);
        // 読むだけでは書き換えも退避もしない。
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        save(&path, &config).unwrap();
        let before: Value = serde_json::from_str(original).unwrap();
        let after: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(after["items"], before["items"]);
        fs::remove_dir_all(&dir).unwrap();
    }

    /// 種類から外した「シリーズの操作」（`kobito-action`）の項目は、読み込むときに捨てる。
    /// ほかの項目はそのまま残り、捨てたあとの形で書き戻す。
    #[test]
    fn drops_items_of_the_removed_kobito_action_type() {
        let (dir, path) = temp_config(
            r#"{
                "version": 4,
                "language": "ja",
                "activation": { "macos": ["ControlLeft", "AltLeft"], "windows": ["ControlLeft", "AltLeft"] },
                "keyboard_layout": "keyboard-jis",
                "popup": { "position": "cursor-monitor-center" },
                "window_actions": [],
                "items": [
                    { "id": "a", "type": "application", "name": "Mail", "key": "KeyM", "shift": false,
                      "target": { "kind": "path", "value": "/System/Applications/Mail.app" },
                      "icon": { "kind": "system" }, "hide_when_frontmost": true },
                    { "id": "s", "type": "kobito-action", "name": "Memo › 新しいメモ", "key": "KeyN", "shift": false,
                      "target": { "kind": "kobito-action", "app": "memo", "action": "new" },
                      "icon": { "kind": "system" }, "hide_when_frontmost": false },
                    { "id": "c", "type": "command", "name": "Notes", "key": "KeyC", "shift": true,
                      "target": { "kind": "command", "value": "uv run notes.py", "working_dir": "/tmp" },
                      "icon": { "kind": "system" }, "hide_when_frontmost": false }
                ]
            }"#,
        );
        let config = load_or_create(&path).unwrap();
        let ids: Vec<&str> = config.items.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, ["a", "c"]);
        assert_eq!(config.language, "ja");
        assert!(config.items[0].hide_when_frontmost);
        assert_eq!(config.items[1].target.working_dir.as_deref(), Some("/tmp"));
        // 退避はせず、捨てたあとの形で書き戻す。
        let names: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["config.json"]);
        let saved: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["items"].as_array().unwrap().len(), 2);
        assert!(!fs::read_to_string(&path).unwrap().contains("kobito-action"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn migrates_v2_config() {
        let mut raw: Value = serde_json::from_str(
            r#"{
                "version": 2,
                "activation": { "macos": ["LeftControl", "LeftOption"], "windows": ["LeftControl", "LeftAlt"] },
                "popup": { "position": "cursor-monitor-center", "show_labels": true, "icon_size": 56, "layout": "grouped" },
                "settings_layout": "keyboard-us",
                "window_actions": [],
                "items": []
            }"#,
        )
        .unwrap();
        migrate(&mut raw);
        let config: LauncherConfig = serde_json::from_value(raw).unwrap();
        assert_eq!(config.activation.macos, ["ControlLeft", "AltLeft"]);
        assert_eq!(config.activation.windows, ["ControlLeft", "AltLeft"]);
        assert_eq!(config.keyboard_layout, "keyboard-us");
        assert_eq!(config.language, "ja");
    }

    #[test]
    fn falls_back_to_default_layout_when_only_list_layouts_exist() {
        let mut raw: Value = serde_json::from_str(
            r#"{
                "version": 3,
                "activation": { "macos": ["ControlLeft"], "windows": ["ControlLeft"] },
                "popup": { "position": "cursor-monitor-center", "icon_size": 56, "layout": "alphabetical" },
                "settings_layout": "alphabetical",
                "items": []
            }"#,
        )
        .unwrap();
        migrate(&mut raw);
        let config: LauncherConfig = serde_json::from_value(raw).unwrap();
        assert_eq!(config.keyboard_layout, default_keyboard_layout());
    }
    #[test]
    fn adds_new_window_actions_and_keeps_existing_keys() {
        use crate::model::WindowActionBinding;
        let mut config = LauncherConfig::default();
        config.window_actions = vec![
            WindowActionBinding {
                action: "maximize".into(),
                name: "全画面表示".into(),
                key: Some("KeyM".into()),
                shift: false,
            },
            WindowActionBinding {
                action: "legacy".into(),
                name: "旧操作".into(),
                key: None,
                shift: false,
            },
        ];
        fill_window_actions(&mut config);
        let actions: Vec<&str> = config.window_actions.iter().map(|b| b.action.as_str()).collect();
        let defaults = default_window_actions();
        assert_eq!(actions.len(), defaults.len() + 1);
        assert!(actions.contains(&"top-left-quarter") && actions.contains(&"center"));
        assert_eq!(actions.last(), Some(&"legacy"));
        let maximize = config.window_actions.iter().find(|b| b.action == "maximize").unwrap();
        assert_eq!(maximize.key.as_deref(), Some("KeyM"));
    }
}
