import type { Language } from "./i18n";

export type Platform = "macos" | "windows";

export interface ActivationConfig {
  macos: string[];
  windows: string[];
}

export interface PopupConfig {
  position: "cursor-monitor-center";
}

/** 設定画面もポップアップも同じキーボード配列で描画する。 */
export type KeyboardLayout = "keyboard-jis" | "keyboard-us";

export interface WindowActionBinding {
  action: string;
  name: string;
  key?: string;
  /** Shiftを押している間に出る裏盤面への割り当て。 */
  shift?: boolean;
}

export interface TargetIcon {
  kind: "system" | "custom";
  value?: string;
}

/** 画面を覚えておく手がかり。つなぎ直しても同じ画面を探せるよう、メーカー・型番・製造番号と名前を持つ。 */
export interface ScreenRef {
  vendor: number;
  model: number;
  serial: number;
  name: string;
}

/** 表示モード。`width`・`height` は見た目の解像度、`pixel_*` は実際の画素数。`refresh` が 0 なら最も高いもの。 */
export interface DisplayModeRef {
  width: number;
  height: number;
  pixel_width: number;
  pixel_height: number;
  refresh: number;
}

/** 解像度の切り替え。`modes` が2つ以上なら、押すたびに順に切り替える。 */
export interface DisplaySetting {
  screen: ScreenRef;
  modes: DisplayModeRef[];
}

export type WallpaperFit = "fill" | "fit" | "stretch" | "center";

/** 壁紙の対象と表示の仕方。`screen` が無ければすべての画面。 */
export interface WallpaperSetting {
  screen?: ScreenRef;
  fit: WallpaperFit;
}

/** 設定画面に出す表示モードの選択肢。 */
export interface ModeOption extends DisplayModeRef {
  hidpi: boolean;
  current: boolean;
  native: boolean;
}

/** つながっている画面。`modes` は見た目の解像度ごとにまとめた一覧、`detailed` はリフレッシュレートまで分けた一覧。 */
export interface DisplayInfo {
  screen: ScreenRef;
  main: boolean;
  builtin: boolean;
  current?: DisplayModeRef | null;
  modes: ModeOption[];
  detailed: ModeOption[];
}

export interface LauncherItem {
  id: string;
  type: "application" | "folder" | "website" | "command" | "display" | "wallpaper";
  name: string;
  key: string;
  /** Shiftを押している間に出る裏盤面への割り当て。 */
  shift?: boolean;
  target: {
    /** Webサイトは "url"、コマンドは "command"、解像度は "display"、それ以外はファイルシステム上の "path"。 */
    kind: "path" | "url" | "command" | "display";
    value: string;
    /** コマンドを動かすフォルダ。省略時はホームフォルダ。 */
    working_dir?: string;
  };
  icon: TargetIcon;
  /** アプリが既に前面にあるときは、開く代わりに隠す。 */
  hide_when_frontmost?: boolean;
  /** 解像度の切り替えの中身（種類が display のとき）。 */
  display?: DisplaySetting;
  /** 壁紙の対象と表示の仕方（種類が wallpaper のとき）。 */
  wallpaper?: WallpaperSetting;
}

export interface LauncherConfig {
  version: number;
  language: Language;
  activation: ActivationConfig;
  keyboard_layout: KeyboardLayout;
  popup: PopupConfig;
  window_actions: WindowActionBinding[];
  items: LauncherItem[];
}

/** 保存時の検査を通らない設定。`item_id` が無いものは設定全体にかかわる。 */
export interface ConfigProblem {
  item_id?: string | null;
  name: string;
  error: string;
}

export interface RuntimeInfo {
  platform: Platform;
  config_path: string;
  /** キーボード監視の起動に失敗した場合のメッセージ。 */
  input_error?: string | null;
}
