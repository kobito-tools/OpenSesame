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

export interface LauncherItem {
  id: string;
  type: "application" | "folder" | "website" | "command" | "kobito-action";
  name: string;
  key: string;
  /** Shiftを押している間に出る裏盤面への割り当て。 */
  shift?: boolean;
  target: {
    /**
     * Webサイトは "url"、コマンドは "command"、シリーズの操作は "kobito-action"、
     * それ以外はファイルシステム上の "path"。
     */
    kind: "path" | "url" | "command" | "kobito-action";
    /** パス・URL・コマンド。シリーズの操作には無い。 */
    value?: string;
    /** コマンドを動かすフォルダ。省略時はホームフォルダ。 */
    working_dir?: string;
    /** シリーズの操作の、アプリID。 */
    app?: string;
    /** シリーズの操作の、操作の id。 */
    action?: string;
  };
  icon: TargetIcon;
  /** アプリが既に前面にあるときは、開く代わりに隠す。 */
  hide_when_frontmost?: boolean;
}

/** シリーズのほかのアプリが起動情報に名乗った操作（設定画面で選ぶ候補）。 */
export interface SeriesAction {
  id: string;
  label: string;
  label_en?: string;
  role?: string;
}

export interface SeriesApp {
  app: string;
  name: string;
  name_en?: string;
  actions: SeriesAction[];
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

export interface RuntimeInfo {
  platform: Platform;
  config_path: string;
  /** キーボード監視の起動に失敗した場合のメッセージ。 */
  input_error?: string | null;
}
