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
}

export interface TargetIcon {
  kind: "system" | "custom";
  value?: string;
}

export interface LauncherItem {
  id: string;
  type: "application" | "folder" | "website";
  name: string;
  key: string;
  target: {
    /** Webサイトは "url"、それ以外はファイルシステム上の "path"。 */
    kind: "path" | "url";
    value: string;
  };
  icon: TargetIcon;
  /** アプリが既に前面にあるときは、開く代わりに隠す。 */
  hide_when_frontmost?: boolean;
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
