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
    /// Shiftを押している間に出る裏盤面への割り当て。
    #[serde(default)]
    pub shift: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    Application,
    Folder,
    Website,
    /// シェルで1行のコマンドを動かす。`target.value` がコマンドそのもの。
    Command,
    /// 画面の解像度を変える。中身は `LauncherItem::display`。`target.value` は画面の名前（表示用）。
    Display,
    /// 壁紙を変える。`target.value` が画像のパス、表示の仕方などは `LauncherItem::wallpaper`。
    Wallpaper,
}

/// 画面を覚えておくための手がかり。つなぎ直すと番号（CGDirectDisplayID）が変わることがあるので、
/// メーカー・型番・製造番号と名前で探し直す（`display::find_screen`）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScreenRef {
    #[serde(default)]
    pub vendor: u32,
    #[serde(default)]
    pub model: u32,
    #[serde(default)]
    pub serial: u32,
    #[serde(default)]
    pub name: String,
}

/// 表示モード。`width`・`height` は見た目の解像度（ポイント）、`pixel_*` は実際の画素数。
/// 両方がそろえば HiDPI かどうかまで決まる。`refresh` が 0 なら、使えるうちで最も高いものを選ぶ。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayModeRef {
    pub width: u32,
    pub height: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    #[serde(default)]
    pub refresh: f64,
}

/// 解像度の切り替え。`modes` が2つ以上なら、押すたびに次のモードへ巡回する。
/// `step` があれば `modes` は空で、押したときの解像度から1段階隣へ変える（`display::step_mode`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplaySetting {
    pub screen: ScreenRef,
    #[serde(default)]
    pub modes: Vec<DisplayModeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<DisplayStep>,
}

/// 解像度を1段階変える向き。`Up` は大きい解像度（文字が小さく、作業領域が広い）の方。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayStep {
    Up,
    Down,
}

/// 壁紙の対象と表示の仕方。`screen` が無ければすべての画面。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallpaperSetting {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen: Option<ScreenRef>,
    #[serde(default = "default_wallpaper_fit")]
    pub fit: String,
    /// 画像の外にできる余白の色（`#RRGGBB`）。無ければ macOS に任せる。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
}

impl Default for WallpaperSetting {
    fn default() -> Self {
        Self {
            screen: None,
            fit: default_wallpaper_fit(),
            fill_color: None,
        }
    }
}

/// 壁紙の表示の仕方。並びは設定画面の並び。
pub const WALLPAPER_FITS: &[&str] = &["fill", "fit", "stretch", "center"];

/// アプリに入っている壁紙。`target.kind` が `builtin` のとき、`target.value` はこの名前。
/// 画像はアプリの `wallpapers/<名前>.jpg`（src-tauri/wallpapers/）。
pub const BUILTIN_WALLPAPERS: &[&str] = &["kobito-day", "kobito-evening", "kobito-night"];

pub fn default_wallpaper_fit() -> String {
    "fill".into()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetPath {
    pub kind: String,
    pub value: String,
    /// コマンドを動かすフォルダ。コマンド以外では使わない。省略時はホームフォルダ。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
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
    /// Shiftを押している間に出る裏盤面への割り当て。
    #[serde(default)]
    pub shift: bool,
    pub target: TargetPath,
    pub icon: TargetIcon,
    /// アプリが既に前面にあるときは、開く代わりに隠す（Windowsは最小化）。
    #[serde(default)]
    pub hide_when_frontmost: bool,
    /// 解像度の切り替えの中身。種類が `display` のときだけ使う。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<DisplaySetting>,
    /// 壁紙の対象と表示の仕方。種類が `wallpaper` のときだけ使う。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallpaper: Option<WallpaperSetting>,
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
    // 設定画面の分類の順に並べる。保存済みの設定もこの順に並べ直す（`storage::fill_window_actions`）。
    [
        ("left-half", "画面左半分"),
        ("right-half", "画面右半分"),
        ("top-half", "画面上半分"),
        ("bottom-half", "画面下半分"),
        ("top-left-quarter", "画面左上4分の1"),
        ("top-right-quarter", "画面右上4分の1"),
        ("bottom-left-quarter", "画面左下4分の1"),
        ("bottom-right-quarter", "画面右下4分の1"),
        ("left-third", "画面左3分の1"),
        ("center-third", "画面中央3分の1"),
        ("right-third", "画面右3分の1"),
        ("top-third", "画面上3分の1"),
        ("middle-third", "画面中段3分の1"),
        ("bottom-third", "画面下3分の1"),
        ("left-two-thirds", "画面左3分の2"),
        ("center-two-thirds", "画面中央3分の2"),
        ("right-two-thirds", "画面右3分の2"),
        ("top-two-thirds", "画面上3分の2"),
        ("middle-two-thirds", "画面中段3分の2"),
        ("bottom-two-thirds", "画面下3分の2"),
        ("expand-left", "左端を広げる"),
        ("shrink-left", "左端を縮める"),
        ("expand-right", "右端を広げる"),
        ("shrink-right", "右端を縮める"),
        ("expand-top", "上端を広げる"),
        ("shrink-top", "上端を縮める"),
        ("expand-bottom", "下端を広げる"),
        ("shrink-bottom", "下端を縮める"),
        ("center", "画面中央"),
        ("almost-maximize", "ほぼ全画面"),
        ("maximize", "全画面表示"),
        ("next-display", "隣の画面へ移す"),
        ("restore", "整形前に戻す"),
    ]
    .into_iter()
    .map(|(action, name)| WindowActionBinding {
        action: action.into(),
        name: name.into(),
        key: None,
        shift: false,
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
