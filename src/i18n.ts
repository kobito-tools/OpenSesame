export type Language = "ja" | "en";

/** 画面に出すアプリ名。tauri.conf.json の productName と揃える。 */
export const APP_NAME = "OpenSesame!";

export const LANGUAGES: Language[] = ["ja", "en"];

/** Rust側が返すエラーはこの接頭辞＋コードで届くので、表示時に翻訳する。 */
const ERROR_PREFIX = "applauncher-error:";

const ja = {
  "app.eyebrow": "OPEN SESAME",
  "app.title": "ショートカット設定",
  "app.subtitle": "起動キーを押したまま、登録したキーを入力します。設定画面は 起動キー + . で開きます。",

  "language.name": "言語",
  "language.ja": "日本語",
  "language.en": "English",

  "activation.title": "起動キー",
  "activation.hint": "カーソルを合わせて delete キーを押すと消去され、次に押した組み合わせが登録されます。",
  "activation.recording": "組み合わせを押してください…",
  "activation.empty": "未設定",
  "activation.tooMany": "起動キーは3つまでです",
  "activation.modifiersOnly": "起動キーには修飾キーのみ使用できます",

  "notice.title": "起動キーが効かないときは",
  "notice.accessibility": "システム設定 → プライバシーとセキュリティ → アクセシビリティで{app}を許可してください。",
  "notice.voiceover": "左Control + 左OptionはVoiceOverの操作キーと重なります。",
  "notice.inputError": "キーボード監視を開始できません",

  "permission.title": "アクセシビリティの許可が必要です",
  "permission.lead": "{app}がキー入力を受け取るには、macOSのアクセシビリティ許可が必要です。下のボタンで設定画面を開き、次の手順で許可してください。",
  "permission.step1": "開いた一覧の左下の「＋」を押し、アプリケーションから {app} を追加します。",
  "permission.step2": "{app} の横のスイッチをオンにします。",
  "permission.step3": "一覧に古い項目が残っている場合は、選んで「－」で削除してから追加し直します。",
  "permission.note": "許可すると数秒で自動的に監視が始まります。アプリの再起動は不要です。",
  "permission.button": "アクセシビリティ設定を開く",
  "permission.granted": "キーボード監視を開始しました",
  "permission.detail": "詳細",

  "layout.title": "キーボード配列",
  "layout.keyboard-jis": "JIS配列",
  "layout.keyboard-us": "英字配列",

  "pane.keyboard.title": "キーボード配列",
  "pane.keyboard.desc": "キーを選ぶと下の一覧の該当行へ移動します。",
  "pane.list.title": "登録項目",
  "pane.list.desc": "種類別に表示しています。キーの重複は自動的に防止されます。",

  "section.window": "ウィンドウ整形",
  "section.window.desc": "キーを設定した操作だけがポップアップに表示されます",
  "section.apps": "アプリ",
  "section.folders": "フォルダ",
  "section.count": "{count}件",

  "button.addApp": "＋ アプリ",
  "button.addFolder": "＋ フォルダ",
  "button.icon": "アイコン",
  "button.delete": "削除",
  "badge.builtin": "組み込み",
  "badge.offLayout": "この配列に無いキー",

  "aria.keyAssign": "割り当てキー",
  "aria.displayName": "表示名",

  "key.unassigned": "未登録",
  "key.reserved": "設定画面を開く（予約）",
  "key.group.letters": "文字",
  "key.group.digits": "数字",
  "key.group.symbols": "記号",
  "key.group.specials": "特殊キー",

  "item.windowHint": "現在のウィンドウを整形",

  "picker.title": "{key} キーに割り当てる",
  "picker.lead": "このキーはまだ空いています。割り当てる種類を選んでください。",
  "picker.window": "ウィンドウサイズ変更",
  "picker.windowDesc": "前面のウィンドウを整形します",
  "picker.app": "アプリの起動",
  "picker.appDesc": "アプリケーションを選びます",
  "picker.folder": "フォルダの選択",
  "picker.folderDesc": "フォルダを選びます",
  "picker.chooseAction": "{key} キーに割り当てるウィンドウ操作",
  "picker.back": "戻る",
  "picker.cancel": "キャンセル",
  "status.saved": "保存しました",
  "status.unregistered": "{key} は未登録です。下の一覧からキーを割り当てられます。",
  "footer.configPath": "設定はOSのユーザーデータ領域に保存されます",
  "popup.empty": "起動キー + . でアプリやウィンドウ操作を登録",

  "action.left-half": "画面左半分",
  "action.right-half": "画面右半分",
  "action.top-half": "画面上半分",
  "action.bottom-half": "画面下半分",
  "action.left-third": "画面左3分の1",
  "action.center-third": "画面中央3分の1",
  "action.right-third": "画面右3分の1",
  "action.left-two-thirds": "画面左3分の2",
  "action.right-two-thirds": "画面右3分の2",
  "action.maximize": "全画面表示",

  "error.invalid-layout": "キーボード配列の指定が不正です",
  "error.invalid-language": "言語の指定が不正です",
  "error.activation-empty": "起動キーを1つ以上選んでください",
  "error.activation-too-many": "起動キーは3つまでです",
  "error.activation-duplicate": "起動キー {0} が重複しています",
  "error.activation-unknown": "起動キーに使えないキーです: {0}",
  "error.reserved-key": ". は設定画面用の予約キーです",
  "error.unassignable-key": "割り当てできないキーです: {0}",
  "error.duplicate-key": "キー {0} が重複しています",
  "error.empty-name": "表示名は空にできません",
  "error.missing-target": "対象が見つかりません: {0}",
  "error.accessibility-denied": "アクセシビリティ権限が付与されていません",
  "error.event-tap-failed": "キーボード監視を開始できませんでした: {0}",
  "error.open-settings-failed": "システム設定を開けませんでした: {0}",
} as const;

export type MessageKey = keyof typeof ja;

const en: Record<MessageKey, string> = {
  "app.eyebrow": "OPEN SESAME",
  "app.title": "Shortcut settings",
  "app.subtitle": "Hold the activation keys and press a registered key. Activation keys + . opens this window.",

  "language.name": "Language",
  "language.ja": "日本語",
  "language.en": "English",

  "activation.title": "Activation keys",
  "activation.hint": "Hover here and press delete to clear, then press the combination you want.",
  "activation.recording": "Press a combination…",
  "activation.empty": "Not set",
  "activation.tooMany": "Up to three activation keys",
  "activation.modifiersOnly": "Activation keys must be modifier keys",

  "notice.title": "If the activation keys do nothing",
  "notice.accessibility": "Allow {app} in System Settings → Privacy & Security → Accessibility.",
  "notice.voiceover": "Left Control + Left Option collides with the VoiceOver modifier.",
  "notice.inputError": "Cannot start keyboard monitoring",

  "permission.title": "Accessibility permission required",
  "permission.lead": "{app} needs macOS Accessibility permission to receive key presses. Open the settings with the button below and follow these steps.",
  "permission.step1": "Press the “+” at the bottom left of the list and add {app} from Applications.",
  "permission.step2": "Turn on the switch next to {app}.",
  "permission.step3": "If an old entry is still listed, select it, remove it with “−”, then add the app again.",
  "permission.note": "Monitoring starts automatically within a few seconds. No restart needed.",
  "permission.button": "Open Accessibility settings",
  "permission.granted": "Keyboard monitoring started",
  "permission.detail": "Details",

  "layout.title": "Keyboard layout",
  "layout.keyboard-jis": "JIS",
  "layout.keyboard-us": "ANSI",

  "pane.keyboard.title": "Keyboard layout",
  "pane.keyboard.desc": "Pick a key to jump to its row in the list below.",
  "pane.list.title": "Registered items",
  "pane.list.desc": "Grouped by type. Duplicate keys are prevented automatically.",

  "section.window": "Window layouts",
  "section.window.desc": "Only actions with a key appear in the popup",
  "section.apps": "Applications",
  "section.folders": "Folders",
  "section.count": "{count}",

  "button.addApp": "＋ App",
  "button.addFolder": "＋ Folder",
  "button.icon": "Icon",
  "button.delete": "Remove",
  "badge.builtin": "Built-in",
  "badge.offLayout": "Not on this layout",

  "aria.keyAssign": "Assigned key",
  "aria.displayName": "Display name",

  "key.unassigned": "Unassigned",
  "key.reserved": "Opens settings (reserved)",
  "key.group.letters": "Letters",
  "key.group.digits": "Digits",
  "key.group.symbols": "Symbols",
  "key.group.specials": "Special keys",

  "item.windowHint": "Resize the frontmost window",

  "picker.title": "Assign the {key} key",
  "picker.lead": "This key is free. Choose what to assign to it.",
  "picker.window": "Window layout",
  "picker.windowDesc": "Resize the frontmost window",
  "picker.app": "Launch an app",
  "picker.appDesc": "Pick an application",
  "picker.folder": "Open a folder",
  "picker.folderDesc": "Pick a folder",
  "picker.chooseAction": "Window layout for the {key} key",
  "picker.back": "Back",
  "picker.cancel": "Cancel",
  "status.saved": "Saved",
  "status.unregistered": "{key} is unassigned. Assign it from the list below.",
  "footer.configPath": "Settings are stored in the OS user data directory",
  "popup.empty": "Press activation keys + . to register apps and window layouts",

  "action.left-half": "Left half",
  "action.right-half": "Right half",
  "action.top-half": "Top half",
  "action.bottom-half": "Bottom half",
  "action.left-third": "Left third",
  "action.center-third": "Center third",
  "action.right-third": "Right third",
  "action.left-two-thirds": "Left two thirds",
  "action.right-two-thirds": "Right two thirds",
  "action.maximize": "Maximize",

  "error.invalid-layout": "Invalid keyboard layout",
  "error.invalid-language": "Invalid language",
  "error.activation-empty": "Choose at least one activation key",
  "error.activation-too-many": "Up to three activation keys",
  "error.activation-duplicate": "Duplicate activation key {0}",
  "error.activation-unknown": "Unsupported activation key: {0}",
  "error.reserved-key": ". is reserved for opening the settings window",
  "error.unassignable-key": "This key cannot be assigned: {0}",
  "error.duplicate-key": "Key {0} is already used",
  "error.empty-name": "The display name cannot be empty",
  "error.missing-target": "Target not found: {0}",
  "error.accessibility-denied": "Accessibility permission has not been granted",
  "error.event-tap-failed": "Could not start keyboard monitoring: {0}",
  "error.open-settings-failed": "Could not open System Settings: {0}",
};

const DICTIONARIES: Record<Language, Record<MessageKey, string>> = { ja, en };

let current: Language = "ja";

export function setLanguage(language: Language): void {
  current = DICTIONARIES[language] ? language : "ja";
  document.documentElement.lang = current;
}

export function language(): Language {
  return current;
}

export function t(key: MessageKey, params?: Record<string, string | number>): string {
  const template = DICTIONARIES[current][key] ?? ja[key] ?? key;
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, name: string) => String(params[name] ?? match));
}

/** Rust側のエラーコードを翻訳する。未知の文字列はそのまま返す。 */
export function translateError(raw: unknown): string {
  const text = String(raw);
  const body = text.slice(text.indexOf(ERROR_PREFIX) + ERROR_PREFIX.length);
  if (!text.includes(ERROR_PREFIX)) return text;
  const separator = body.indexOf(":");
  const code = (separator === -1 ? body : body.slice(0, separator)).trim();
  const argument = separator === -1 ? "" : body.slice(separator + 1);
  const key = `error.${code}` as MessageKey;
  if (!(key in ja)) return text;
  return t(key, { 0: argument });
}
