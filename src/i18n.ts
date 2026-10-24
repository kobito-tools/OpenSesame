export type Language = "ja" | "en";

/** 画面に出すアプリ名。tauri.conf.json の productName と揃える。 */
export const APP_NAME = "OpenSesame!";

export const LANGUAGES: Language[] = ["ja", "en"];

/** Rust側が返すエラーはこの接頭辞＋コードで届くので、表示時に翻訳する。 */
const ERROR_PREFIX = "applauncher-error:";

const ja = {
  "app.eyebrow": "OPEN SESAME",
  "app.title": "ショートカット設定",
  "app.subtitle": "起動キーを押したまま、登録したキーを入力します。Shiftも押すと裏盤面に切り替わります。設定画面は 起動キー + . で開きます。",

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
  "notice.config.title": "動かさない項目があります",
  "notice.config.lead": "設定ファイルに、画面からは保存できない中身があります。直すか消すまで、キーを押しても動かしません。",

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
  "layer.front": "表盤面",
  "layer.back": "裏盤面（Shift）",
  "layer.backBadge": "⇧ 裏盤面",
  "layer.useRightShift": "起動キーに左Shiftが含まれているため、裏盤面には右Shiftで切り替えます。",
  "layer.useLeftShift": "起動キーに右Shiftが含まれているため、裏盤面には左Shiftで切り替えます。",
  "layer.unavailable": "起動キーに左右両方のShiftが含まれているため、裏盤面に切り替えられません。",
  "pane.keyboard.desc": "キーを選ぶと下の一覧の該当行へ移動します。登録済みのキーはドラッグで別のキーへ移せます。",
  "pane.list.title": "登録項目",
  "pane.list.desc": "種類別に表示しています。キーの重複は自動的に防止されます。",

  "section.window": "ウィンドウ整形",
  "section.window.desc": "キーを設定した操作だけがポップアップに表示されます",
  "section.window.unassigned": "キーを割り当てていない操作（{count}件）",
  "section.apps": "アプリ",
  "section.folders": "フォルダ",
  "section.websites": "Webサイト",
  "section.commands": "コマンド",
  "section.count": "{count}件",

  "button.addApp": "＋ アプリ",
  "button.addFolder": "＋ フォルダ",
  "button.addWebsite": "＋ Webサイト",
  "button.addCommand": "＋ コマンド",
  "button.icon": "アイコン",
  "button.refetchIcon": "アイコン再取得",
  "button.editCommand": "編集",
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
  "item.commandIn": "{command}（{dir} で実行）",
  "item.hideWhenFrontmost": "前面なら隠す",
  "item.hideWhenFrontmostHint": "このアプリが前面にあるときにもう一度押すと隠します（Windowsでは最小化）",

  "picker.title": "{key} キーに割り当てる",
  "picker.lead": "このキーはまだ空いています。割り当てる種類を選んでください。",
  "picker.window": "ウィンドウサイズ変更",
  "picker.windowDesc": "前面のウィンドウを整形します",
  "picker.app": "アプリの起動",
  "picker.appDesc": "アプリケーションを選びます",
  "picker.folder": "フォルダの選択",
  "picker.folderDesc": "フォルダを選びます",
  "picker.website": "Webサイトを開く",
  "picker.websiteDesc": "URLを入力して既定のブラウザで開きます",
  "picker.websiteTitle": "{key} キーに割り当てるWebサイト",
  "picker.websiteUrl": "URL",
  "picker.websiteName": "表示名",
  "picker.websiteNameHint": "省略するとドメイン名になります",
  "picker.websiteAdd": "登録",
  "picker.command": "コマンドを実行",
  "picker.commandDesc": "ターミナルで打つコマンドを1行で登録します（Python のアプリなど）",
  "picker.commandTitle": "{key} キーに割り当てるコマンド",
  "picker.commandEdit": "コマンドの編集",
  "picker.commandLine": "コマンド（1行）",
  "picker.commandLineHint": "例: uv run app.py",
  "picker.commandDir": "作業フォルダ（任意）",
  "picker.commandDirHint": "省略するとホームフォルダで動きます",
  "picker.commandDirPick": "選ぶ…",
  "picker.commandName": "表示名",
  "picker.commandNameHint": "省略するとコマンドの最初の語になります",
  "picker.commandSave": "保存",
  "picker.chooseAction": "{key} キーに割り当てるウィンドウ操作",
  "picker.back": "戻る",
  "picker.cancel": "キャンセル",
  "status.saved": "保存しました",
  "status.fetchingIcon": "アイコンを取得中…",
  "status.moved": "{from} の割り当てを {to} へ移動しました",
  "status.swapped": "{from} と {to} の割り当てを入れ替えました",
  "status.unregistered": "{key} は未登録です。下の一覧からキーを割り当てられます。",
  "footer.configPath": "設定はOSのユーザーデータ領域に保存されます",
  "popup.empty": "起動キー + . でアプリやWebサイト、コマンド、ウィンドウ操作を登録",

  "action.left-half": "画面左半分",
  "action.right-half": "画面右半分",
  "action.top-half": "画面上半分",
  "action.bottom-half": "画面下半分",
  "action.top-left-quarter": "画面左上4分の1",
  "action.top-right-quarter": "画面右上4分の1",
  "action.bottom-left-quarter": "画面左下4分の1",
  "action.bottom-right-quarter": "画面右下4分の1",
  "action.left-third": "画面左3分の1",
  "action.center-third": "画面中央3分の1",
  "action.right-third": "画面右3分の1",
  "action.left-two-thirds": "画面左3分の2",
  "action.right-two-thirds": "画面右3分の2",
  "action.center": "画面中央（大きさはそのまま）",
  "action.maximize": "全画面表示",

  "error.invalid-layout": "キーボード配列の指定が不正です",
  "error.invalid-language": "言語の指定が不正です",
  "error.activation-empty": "起動キーを1つ以上選んでください",
  "error.activation-too-many": "起動キーは3つまでです",
  "error.activation-shift-only": "Shiftだけの起動キーは使えません。大文字の入力と区別できないため、ほかの修飾キーと組み合わせてください",
  "error.activation-duplicate": "起動キー {0} が重複しています",
  "error.activation-unknown": "起動キーに使えないキーです: {0}",
  "error.reserved-key": ". は設定画面用の予約キーです",
  "error.unassignable-key": "割り当てできないキーです: {0}",
  "error.duplicate-key": "キー {0} が重複しています",
  "error.empty-name": "表示名は空にできません",
  "error.missing-target": "対象が見つかりません: {0}",
  "error.invalid-url": "http または https のURLを入力してください: {0}",
  "error.icon-fetch-failed": "このサイトのアイコンを取得できませんでした。「アイコン」から画像を設定できます",
  "error.accessibility-denied": "アクセシビリティ権限が付与されていません",
  "error.event-tap-failed": "キーボード監視を開始できませんでした: {0}",
  "error.open-settings-failed": "システム設定を開けませんでした: {0}",
  "error.invalid-command": "コマンドを1行で入力してください",
  "error.missing-working-dir": "作業フォルダが見つかりません: {0}",
  "error.command-spawn-failed": "コマンドを起動できませんでした: {0}",
  "error.command-failed": "コマンドがすぐに終わりました（終了コード {0}）",
  "error.invalid-path": "アプリとフォルダは絶対パスで指定してください: {0}",
  "error.item-blocked": "「{0}」は設定ファイルの中身が検査を通らないので、動かしませんでした",
} as const;

export type MessageKey = keyof typeof ja;

const en: Record<MessageKey, string> = {
  "app.eyebrow": "OPEN SESAME",
  "app.title": "Shortcut settings",
  "app.subtitle": "Hold the activation keys and press a registered key. Add Shift to switch to the Shift layer. Activation keys + . opens this window.",

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
  "notice.config.title": "Some items are turned off",
  "notice.config.lead": "The config file holds entries the settings window would not save. They do nothing when pressed until you fix or remove them.",

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
  "layer.front": "Main",
  "layer.back": "Shift layer",
  "layer.backBadge": "⇧ Shift layer",
  "layer.useRightShift": "Left Shift is part of the activation keys, so use Right Shift for the Shift layer.",
  "layer.useLeftShift": "Right Shift is part of the activation keys, so use Left Shift for the Shift layer.",
  "layer.unavailable": "Both Shift keys are activation keys, so the Shift layer cannot be opened.",
  "pane.keyboard.desc": "Pick a key to jump to its row in the list below. Drag an assigned key onto another key to move it.",
  "pane.list.title": "Registered items",
  "pane.list.desc": "Grouped by type. Duplicate keys are prevented automatically.",

  "section.window": "Window layouts",
  "section.window.desc": "Only actions with a key appear in the popup",
  "section.window.unassigned": "Actions without a key ({count})",
  "section.apps": "Applications",
  "section.folders": "Folders",
  "section.websites": "Websites",
  "section.commands": "Commands",
  "section.count": "{count}",

  "button.addApp": "＋ App",
  "button.addFolder": "＋ Folder",
  "button.addWebsite": "＋ Website",
  "button.addCommand": "＋ Command",
  "button.icon": "Icon",
  "button.refetchIcon": "Refetch icon",
  "button.editCommand": "Edit",
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
  "item.commandIn": "{command} (runs in {dir})",
  "item.hideWhenFrontmost": "Hide if in front",
  "item.hideWhenFrontmostHint": "Press again while this app is in front to hide it (minimize on Windows)",

  "picker.title": "Assign the {key} key",
  "picker.lead": "This key is free. Choose what to assign to it.",
  "picker.window": "Window layout",
  "picker.windowDesc": "Resize the frontmost window",
  "picker.app": "Launch an app",
  "picker.appDesc": "Pick an application",
  "picker.folder": "Open a folder",
  "picker.folderDesc": "Pick a folder",
  "picker.website": "Open a website",
  "picker.websiteDesc": "Enter a URL to open in your default browser",
  "picker.websiteTitle": "Website for the {key} key",
  "picker.websiteUrl": "URL",
  "picker.websiteName": "Display name",
  "picker.websiteNameHint": "Defaults to the domain name",
  "picker.websiteAdd": "Add",
  "picker.command": "Run a command",
  "picker.commandDesc": "Register a one-line terminal command, such as a Python app",
  "picker.commandTitle": "Command for the {key} key",
  "picker.commandEdit": "Edit command",
  "picker.commandLine": "Command (one line)",
  "picker.commandLineHint": "e.g. uv run app.py",
  "picker.commandDir": "Working folder (optional)",
  "picker.commandDirHint": "Runs in your home folder when left empty",
  "picker.commandDirPick": "Choose…",
  "picker.commandName": "Display name",
  "picker.commandNameHint": "Defaults to the first word of the command",
  "picker.commandSave": "Save",
  "picker.chooseAction": "Window layout for the {key} key",
  "picker.back": "Back",
  "picker.cancel": "Cancel",
  "status.saved": "Saved",
  "status.fetchingIcon": "Fetching icon…",
  "status.moved": "Moved {from} to {to}",
  "status.swapped": "Swapped {from} and {to}",
  "status.unregistered": "{key} is unassigned. Assign it from the list below.",
  "footer.configPath": "Settings are stored in the OS user data directory",
  "popup.empty": "Press activation keys + . to register apps, websites, commands and window layouts",

  "action.left-half": "Left half",
  "action.right-half": "Right half",
  "action.top-half": "Top half",
  "action.bottom-half": "Bottom half",
  "action.top-left-quarter": "Top left quarter",
  "action.top-right-quarter": "Top right quarter",
  "action.bottom-left-quarter": "Bottom left quarter",
  "action.bottom-right-quarter": "Bottom right quarter",
  "action.left-third": "Left third",
  "action.center-third": "Center third",
  "action.right-third": "Right third",
  "action.left-two-thirds": "Left two thirds",
  "action.right-two-thirds": "Right two thirds",
  "action.center": "Center (keep size)",
  "action.maximize": "Maximize",

  "error.invalid-layout": "Invalid keyboard layout",
  "error.invalid-language": "Invalid language",
  "error.activation-empty": "Choose at least one activation key",
  "error.activation-too-many": "Up to three activation keys",
  "error.activation-shift-only": "Shift alone cannot be the activation key because it is used for capital letters. Combine it with another modifier key",
  "error.activation-duplicate": "Duplicate activation key {0}",
  "error.activation-unknown": "Unsupported activation key: {0}",
  "error.reserved-key": ". is reserved for opening the settings window",
  "error.unassignable-key": "This key cannot be assigned: {0}",
  "error.duplicate-key": "Key {0} is already used",
  "error.empty-name": "The display name cannot be empty",
  "error.missing-target": "Target not found: {0}",
  "error.invalid-url": "Enter an http or https URL: {0}",
  "error.icon-fetch-failed": "Could not get this site's icon. You can set an image with Icon",
  "error.accessibility-denied": "Accessibility permission has not been granted",
  "error.event-tap-failed": "Could not start keyboard monitoring: {0}",
  "error.open-settings-failed": "Could not open System Settings: {0}",
  "error.invalid-command": "Enter the command on one line",
  "error.missing-working-dir": "Working folder not found: {0}",
  "error.command-spawn-failed": "Could not start the command: {0}",
  "error.command-failed": "The command stopped right away (exit code {0})",
  "error.invalid-path": "Apps and folders need an absolute path: {0}",
  "error.item-blocked": "\"{0}\" did not run because its entry in the config file did not pass the checks",
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
