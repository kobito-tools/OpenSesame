use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationConfig {
    pub macos: Vec<String>,
    pub windows: Vec<String>,
}

impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            macos: vec!["ControlLeft".into(), "AltLeft".into()],
            windows: vec!["ControlLeft".into(), "AltLeft".into()],
        }
    }
}

impl ActivationConfig {
    /// 実行中のOS向けの起動キー。
    pub fn current(&self) -> &Vec<String> {
        #[cfg(target_os = "windows")]
        {
            &self.windows
        }
        #[cfg(not(target_os = "windows"))]
        {
            &self.macos
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupConfig {
    #[serde(default = "default_position")]
    pub position: String,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowActionBinding {
    pub action: String,
    pub name: String,
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    Application,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetPath {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetIcon {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherItem {
    pub id: String,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub name: String,
    pub key: String,
    pub target: TargetPath,
    pub icon: TargetIcon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub version: u32,
    #[serde(default = "default_language")]
    pub language: String,
    pub activation: ActivationConfig,
    /// 設定画面とポップアップで共通のキーボード配列。
    #[serde(default = "default_keyboard_layout")]
    pub keyboard_layout: String,
    #[serde(default)]
    pub popup: PopupConfig,
    #[serde(default = "default_window_actions")]
    pub window_actions: Vec<WindowActionBinding>,
    pub items: Vec<LauncherItem>,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            language: default_language(),
            activation: ActivationConfig::default(),
            keyboard_layout: default_keyboard_layout(),
            popup: PopupConfig::default(),
            window_actions: default_window_actions(),
            items: Vec::new(),
        }
    }
}

/// v4でポップアップをキーボード配列のみにし、表示言語を追加した。
pub const CONFIG_VERSION: u32 = 4;

pub const LANGUAGES: &[&str] = &["ja", "en"];
pub const KEYBOARD_LAYOUTS: &[&str] = &["keyboard-jis", "keyboard-us"];

fn default_position() -> String {
    "cursor-monitor-center".into()
}

pub fn default_language() -> String {
    "ja".into()
}

pub fn default_keyboard_layout() -> String {
    "keyboard-jis".into()
}

pub fn default_window_actions() -> Vec<WindowActionBinding> {
    [
        ("left-half", "画面左半分"),
        ("right-half", "画面右半分"),
        ("top-half", "画面上半分"),
        ("bottom-half", "画面下半分"),
        ("left-third", "画面左3分の1"),
        ("center-third", "画面中央3分の1"),
        ("right-third", "画面右3分の1"),
        ("left-two-thirds", "画面左3分の2"),
        ("right-two-thirds", "画面右3分の2"),
        ("maximize", "全画面表示"),
    ]
    .into_iter()
    .map(|(action, name)| WindowActionBinding {
        action: action.into(),
        name: name.into(),
        key: None,
    })
    .collect()
}

#[derive(Debug, Serialize)]
pub struct RuntimeInfo {
    pub platform: &'static str,
    pub config_path: String,
    /// キーボード監視が起動できなかった場合の理由。
    pub input_error: Option<String>,
}
