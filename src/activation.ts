import type { Platform } from "./types";

/**
 * 起動キーに使える修飾キー。設定ファイルにはこの正規名で保存する。
 * 名前はブラウザの `KeyboardEvent.code` と同じなので、録音した値をそのまま使える。
 */
export const MODIFIER_CODES = [
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
  "ShiftLeft",
  "ShiftRight",
] as const;

export type ModifierCode = (typeof MODIFIER_CODES)[number];

const LABELS: Record<Platform, Record<ModifierCode, string>> = {
  macos: {
    ControlLeft: "左Control",
    ControlRight: "右Control",
    AltLeft: "左Option",
    AltRight: "右Option",
    MetaLeft: "左Command",
    MetaRight: "右Command",
    ShiftLeft: "左Shift",
    ShiftRight: "右Shift",
  },
  windows: {
    ControlLeft: "左Ctrl",
    ControlRight: "右Ctrl",
    AltLeft: "左Alt",
    AltRight: "右Alt",
    MetaLeft: "左Win",
    MetaRight: "右Win",
    ShiftLeft: "左Shift",
    ShiftRight: "右Shift",
  },
};

const LABELS_EN: Record<Platform, Record<ModifierCode, string>> = {
  macos: {
    ControlLeft: "Left Control",
    ControlRight: "Right Control",
    AltLeft: "Left Option",
    AltRight: "Right Option",
    MetaLeft: "Left Command",
    MetaRight: "Right Command",
    ShiftLeft: "Left Shift",
    ShiftRight: "Right Shift",
  },
  windows: {
    ControlLeft: "Left Ctrl",
    ControlRight: "Right Ctrl",
    AltLeft: "Left Alt",
    AltRight: "Right Alt",
    MetaLeft: "Left Win",
    MetaRight: "Right Win",
    ShiftLeft: "Left Shift",
    ShiftRight: "Right Shift",
  },
};

/** 旧設定（LeftControl / LeftOption など）を正規名へ寄せる。 */
const ALIASES: Record<string, ModifierCode> = {
  leftcontrol: "ControlLeft",
  rightcontrol: "ControlRight",
  leftctrl: "ControlLeft",
  rightctrl: "ControlRight",
  leftoption: "AltLeft",
  rightoption: "AltRight",
  leftalt: "AltLeft",
  rightalt: "AltRight",
  leftcommand: "MetaLeft",
  rightcommand: "MetaRight",
  leftwin: "MetaLeft",
  rightwin: "MetaRight",
  leftmeta: "MetaLeft",
  rightmeta: "MetaRight",
  leftshift: "ShiftLeft",
  rightshift: "ShiftRight",
};

export function isModifierCode(code: string): code is ModifierCode {
  return (MODIFIER_CODES as readonly string[]).includes(code);
}

export function normalizeModifier(value: string): ModifierCode | undefined {
  const direct = MODIFIER_CODES.find((code) => code.toLowerCase() === value.toLowerCase());
  return direct ?? ALIASES[value.toLowerCase().replace(/[\s_-]/g, "")];
}

export function modifierLabel(platform: Platform, language: string, value: string): string {
  const code = normalizeModifier(value);
  if (!code) return value;
  return (language === "en" ? LABELS_EN : LABELS)[platform][code];
}

/** 表示順は修飾キーの定義順に揃え、毎回同じ並びで見せる。 */
export function sortModifiers(values: string[]): ModifierCode[] {
  const normalized = values
    .map(normalizeModifier)
    .filter((code): code is ModifierCode => Boolean(code));
  return MODIFIER_CODES.filter((code) => normalized.includes(code));
}

export function activationLabel(platform: Platform, language: string, values: string[]): string {
  const sorted = sortModifiers(values);
  return sorted.map((code) => modifierLabel(platform, language, code)).join(" + ");
}
