import { t, type MessageKey } from "./i18n";
import type { KeyboardLayout } from "./types";

/** 1キャップ分の定義。code を省略した要素はレイアウト調整用の空白。 */
export interface KeyCap {
  code?: string;
  label?: string;
  units?: number;
  /** JIS配列のEnterのように縦に複数行を占めるキー。 */
  rowSpan?: number;
  reserved?: boolean;
}

/** すべての行は 16 ユニット幅に揃える。CSS grid の 160 列へ 10 倍して割り当てる。 */
export const ROW_UNITS = 16;

const gap = (units: number): KeyCap => ({ units });

const KEYBOARD_US: KeyCap[][] = [
  [
    { code: "Escape", label: "esc", units: 1.4 },
    { code: "Backquote", label: "`" },
    { code: "Digit1", label: "1" },
    { code: "Digit2", label: "2" },
    { code: "Digit3", label: "3" },
    { code: "Digit4", label: "4" },
    { code: "Digit5", label: "5" },
    { code: "Digit6", label: "6" },
    { code: "Digit7", label: "7" },
    { code: "Digit8", label: "8" },
    { code: "Digit9", label: "9" },
    { code: "Digit0", label: "0" },
    { code: "Minus", label: "-" },
    { code: "Equal", label: "=" },
    { code: "Backspace", label: "⌫", units: 1.6 },
  ],
  [
    { code: "Tab", label: "⇥", units: 1.5 },
    { code: "KeyQ", label: "Q" },
    { code: "KeyW", label: "W" },
    { code: "KeyE", label: "E" },
    { code: "KeyR", label: "R" },
    { code: "KeyT", label: "T" },
    { code: "KeyY", label: "Y" },
    { code: "KeyU", label: "U" },
    { code: "KeyI", label: "I" },
    { code: "KeyO", label: "O" },
    { code: "KeyP", label: "P" },
    { code: "BracketLeft", label: "[" },
    { code: "BracketRight", label: "]" },
    { code: "Backslash", label: "\\", units: 1.5 },
    gap(1),
  ],
  [
    gap(1.8),
    { code: "KeyA", label: "A" },
    { code: "KeyS", label: "S" },
    { code: "KeyD", label: "D" },
    { code: "KeyF", label: "F" },
    { code: "KeyG", label: "G" },
    { code: "KeyH", label: "H" },
    { code: "KeyJ", label: "J" },
    { code: "KeyK", label: "K" },
    { code: "KeyL", label: "L" },
    { code: "Semicolon", label: ";" },
    { code: "Quote", label: "'" },
    { code: "Enter", label: "⏎", units: 2.2 },
    gap(1),
  ],
  [
    gap(2.3),
    { code: "KeyZ", label: "Z" },
    { code: "KeyX", label: "X" },
    { code: "KeyC", label: "C" },
    { code: "KeyV", label: "V" },
    { code: "KeyB", label: "B" },
    { code: "KeyN", label: "N" },
    { code: "KeyM", label: "M" },
    { code: "Comma", label: "," },
    { code: "Period", label: ".", reserved: true },
    { code: "Slash", label: "/" },
    gap(0.7),
    { code: "ArrowUp", label: "↑" },
    gap(2),
  ],
  [
    gap(4.5),
    { code: "Space", label: "space", units: 5.5 },
    gap(2),
    { code: "ArrowLeft", label: "←" },
    { code: "ArrowDown", label: "↓" },
    { code: "ArrowRight", label: "→" },
    gap(1),
  ],
];

const KEYBOARD_JIS: KeyCap[][] = [
  [
    { code: "Escape", label: "esc", units: 1.3 },
    { code: "Digit1", label: "1" },
    { code: "Digit2", label: "2" },
    { code: "Digit3", label: "3" },
    { code: "Digit4", label: "4" },
    { code: "Digit5", label: "5" },
    { code: "Digit6", label: "6" },
    { code: "Digit7", label: "7" },
    { code: "Digit8", label: "8" },
    { code: "Digit9", label: "9" },
    { code: "Digit0", label: "0" },
    { code: "Minus", label: "-" },
    { code: "Equal", label: "^" },
    { code: "IntlYen", label: "¥" },
    { code: "Backspace", label: "⌫", units: 1.7 },
  ],
  [
    { code: "Tab", label: "⇥", units: 1.6 },
    { code: "KeyQ", label: "Q" },
    { code: "KeyW", label: "W" },
    { code: "KeyE", label: "E" },
    { code: "KeyR", label: "R" },
    { code: "KeyT", label: "T" },
    { code: "KeyY", label: "Y" },
    { code: "KeyU", label: "U" },
    { code: "KeyI", label: "I" },
    { code: "KeyO", label: "O" },
    { code: "KeyP", label: "P" },
    { code: "BracketLeft", label: "@" },
    { code: "BracketRight", label: "[" },
    gap(0.3),
    { code: "Enter", label: "⏎", units: 2.1, rowSpan: 2 },
  ],
  [
    gap(1.9),
    { code: "KeyA", label: "A" },
    { code: "KeyS", label: "S" },
    { code: "KeyD", label: "D" },
    { code: "KeyF", label: "F" },
    { code: "KeyG", label: "G" },
    { code: "KeyH", label: "H" },
    { code: "KeyJ", label: "J" },
    { code: "KeyK", label: "K" },
    { code: "KeyL", label: "L" },
    { code: "Semicolon", label: ";" },
    { code: "Quote", label: ":" },
    { code: "Backslash", label: "]" },
    gap(2.1),
  ],
  [
    gap(2.4),
    { code: "KeyZ", label: "Z" },
    { code: "KeyX", label: "X" },
    { code: "KeyC", label: "C" },
    { code: "KeyV", label: "V" },
    { code: "KeyB", label: "B" },
    { code: "KeyN", label: "N" },
    { code: "KeyM", label: "M" },
    { code: "Comma", label: "," },
    { code: "Period", label: ".", reserved: true },
    { code: "Slash", label: "/" },
    { code: "IntlRo", label: "\\" },
    { code: "ArrowUp", label: "↑" },
    gap(1.6),
  ],
  [
    gap(4),
    { code: "Space", label: "space", units: 5 },
    gap(3.4),
    { code: "ArrowLeft", label: "←" },
    { code: "ArrowDown", label: "↓" },
    { code: "ArrowRight", label: "→" },
    gap(0.6),
  ],
];

export const KEYBOARD_LAYOUTS: Record<KeyboardLayout, KeyCap[][]> = {
  "keyboard-us": KEYBOARD_US,
  "keyboard-jis": KEYBOARD_JIS,
};

export const KEYBOARD_LAYOUTS_ORDER: KeyboardLayout[] = ["keyboard-jis", "keyboard-us"];

/** `.` は設定画面を開く予約キーなので割り当て対象から外す。 */
export const RESERVED_KEY = "Period";

export interface KeyGroup {
  title: MessageKey;
  codes: string[];
}

const LETTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".split("").map((letter) => `Key${letter}`);
const DIGITS = "1234567890".split("").map((digit) => `Digit${digit}`);
const SYMBOLS = [
  "Backquote",
  "Minus",
  "Equal",
  "BracketLeft",
  "BracketRight",
  "Backslash",
  "Semicolon",
  "Quote",
  "Comma",
  "Slash",
  "IntlYen",
  "IntlRo",
  "IntlBackslash",
];
const SPECIALS = [
  "Escape",
  "Tab",
  "Enter",
  "Space",
  "Backspace",
  "ArrowLeft",
  "ArrowUp",
  "ArrowDown",
  "ArrowRight",
];

export const KEY_GROUPS: KeyGroup[] = [
  { title: "key.group.letters", codes: LETTERS },
  { title: "key.group.digits", codes: DIGITS },
  { title: "key.group.symbols", codes: SYMBOLS },
  { title: "key.group.specials", codes: SPECIALS },
];

export const ASSIGNABLE_KEYS: string[] = KEY_GROUPS.flatMap((group) => group.codes);

/** その配列に実在するキーの集合。無いキーへの割り当ては配列上に出せない。 */
export function layoutKeys(layout: KeyboardLayout): Set<string> {
  return new Set(
    KEYBOARD_LAYOUTS[layout].flatMap((row) =>
      row.map((cap) => cap.code).filter((code): code is string => Boolean(code)),
    ),
  );
}

const EXTRA_LABELS: Record<string, string> = {
  Backquote: "`",
  Minus: "-",
  Equal: "= / ^",
  BracketLeft: "[ / @",
  BracketRight: "] / [",
  Backslash: "\\ / ]",
  Semicolon: ";",
  Quote: "' / :",
  Comma: ",",
  Period: ".",
  Slash: "/",
  IntlYen: "¥",
  IntlRo: "＼(ろ)",
  IntlBackslash: "§",
  Escape: "esc",
  Tab: "⇥",
  Enter: "⏎",
  Space: "space",
  Backspace: "⌫",
  ArrowLeft: "←",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowRight: "→",
};

/** 一覧やポップアップのバッジに使う短いキー名。 */
export function displayKey(code: string): string {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  return EXTRA_LABELS[code] ?? code;
}

export interface KeyboardSlot {
  name: string;
  node: () => HTMLElement;
}

export interface KeyboardOptions {
  variant: "settings" | "popup";
  onSelect?: (code: string) => void;
  /** 割り当て済みのキーを別のキーへドラッグしたときに呼ぶ。 */
  onMove?: (from: string, to: string) => void;
}

/** これ以上動いたらクリックではなくドラッグとみなす距離(px)。 */
const DRAG_THRESHOLD = 5;

/**
 * 割り当て済みのキーをつかんで別のキーへ運べるようにする。
 * TauriのWebViewではHTML5のドラッグ&ドロップがファイルドロップ処理と競合するので、
 * ポインタイベントで実装する。
 */
function enableKeyDrag(
  keyboard: HTMLElement,
  key: HTMLElement,
  code: string,
  onMove: (from: string, to: string) => void,
): void {
  key.addEventListener("pointerdown", (down) => {
    if (down.button !== 0) return;
    const startX = down.clientX;
    const startY = down.clientY;
    let ghost: HTMLElement | null = null;
    let target: HTMLElement | null = null;

    const dropTargetAt = (x: number, y: number): HTMLElement | null => {
      const hit = document.elementFromPoint(x, y)?.closest<HTMLElement>(".keyboard-key");
      if (!hit || !keyboard.contains(hit) || hit === key) return null;
      if (hit.classList.contains("reserved") || !hit.dataset.code) return null;
      return hit;
    };

    const onPointerMove = (move: PointerEvent) => {
      if (!ghost) {
        if (Math.hypot(move.clientX - startX, move.clientY - startY) < DRAG_THRESHOLD) return;
        const rect = key.getBoundingClientRect();
        ghost = key.cloneNode(true) as HTMLElement;
        ghost.classList.add("drag-ghost");
        ghost.style.width = `${rect.width}px`;
        ghost.style.height = `${rect.height}px`;
        document.body.append(ghost);
        key.classList.add("drag-source");
        keyboard.classList.add("dragging");
      }
      ghost.style.transform = `translate(${move.clientX - ghost.offsetWidth / 2}px, ${move.clientY - ghost.offsetHeight / 2}px)`;
      const next = dropTargetAt(move.clientX, move.clientY);
      if (next !== target) {
        target?.classList.remove("drop-target");
        next?.classList.add("drop-target");
        target = next;
      }
    };

    const finish = (commit: boolean) => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("pointercancel", onPointerCancel);
      if (!ghost) return;
      ghost.remove();
      key.classList.remove("drag-source");
      keyboard.classList.remove("dragging");
      target?.classList.remove("drop-target");
      // ドラッグ直後に同じキー上で発生するクリックで一覧へ移動しないよう、少しの間だけ印を付ける。
      key.dataset.dragged = "";
      window.setTimeout(() => delete key.dataset.dragged, 0);
      const to = target?.dataset.code;
      if (commit && to && to !== code) onMove(code, to);
    };
    const onPointerUp = () => finish(true);
    const onPointerCancel = () => finish(false);

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onPointerCancel);
  });
}

/** 設定画面とポップアップで共有するキーボード配列の描画。 */
export function buildKeyboard(
  layout: KeyboardLayout,
  slots: Map<string, KeyboardSlot>,
  options: KeyboardOptions,
): HTMLElement {
  const rows = KEYBOARD_LAYOUTS[layout];
  const keyboard = document.createElement("div");
  keyboard.className = `keyboard-layout ${layout} keyboard-${options.variant}`;
  // 列数と行数だけを渡し、実寸はCSSの --key-size に委ねて1x1を正方形に保つ。
  keyboard.style.setProperty("--columns", String(ROW_UNITS * 10));
  keyboard.style.setProperty("--rows", String(rows.length));
  rows.forEach((capRow, rowIndex) => {
    let offset = 0;
    for (const cap of capRow) {
      const units = cap.units ?? 1;
      const column = Math.round(offset * 10) + 1;
      offset += units;
      if (!cap.code) continue;
      const slot = slots.get(cap.code);
      const key = document.createElement(options.onSelect && !cap.reserved ? "button" : "div");
      if (key instanceof HTMLButtonElement) key.type = "button";
      key.className = "keyboard-key";
      key.style.gridColumn = `${column} / span ${Math.round(units * 10)}`;
      key.style.gridRow = `${rowIndex + 1} / span ${cap.rowSpan ?? 1}`;
      // 下の行ほど早く立ち上がるよう、逆順の番号をアニメーション遅延に渡す。
      key.style.setProperty("--row", String(rows.length - 1 - rowIndex));
      key.dataset.code = cap.code;
      if (slot) key.classList.add("assigned");
      if (cap.reserved) key.classList.add("reserved");
      // 名前はキー上に常時出るので、補足はネイティブのツールチップで足りる。
      // CSS製のものはスクロール枠に切られるため使わない。
      key.title = cap.reserved ? t("key.reserved") : (slot?.name ?? t("key.unassigned"));
      // アイコンはキーの枠いっぱいに敷き、キー名と表示名はその上に重ねる。
      if (slot) {
        const visual = document.createElement("span");
        visual.className = "key-visual";
        visual.append(slot.node());
        key.append(visual);
      }
      const label = document.createElement("kbd");
      label.textContent = cap.label ?? displayKey(cap.code);
      key.append(label);
      if (slot) {
        const name = document.createElement("span");
        name.className = "key-name";
        name.textContent = slot.name;
        key.append(name);
      }
      if (options.onSelect && !cap.reserved) {
        const code = cap.code;
        key.addEventListener("click", () => {
          if (key.dataset.dragged !== undefined) return;
          options.onSelect?.(code);
        });
      }
      if (slot && options.onMove && !cap.reserved) {
        enableKeyDrag(keyboard, key, cap.code, options.onMove);
      }
      keyboard.append(key);
    }
  });
  return keyboard;
}
