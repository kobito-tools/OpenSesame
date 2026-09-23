use crate::{
    keys::normalize_modifier,
    model::{
        default_keyboard_layout, ActivationConfig, LauncherConfig, CONFIG_VERSION,
        KEYBOARD_LAYOUTS,
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
    let mut config: LauncherConfig = serde_json::from_value(raw)
        .map_err(|error| format!("設定ファイルが不正です: {error}"))?;
    if migrated {
        config.version = CONFIG_VERSION;
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
}
