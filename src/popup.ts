import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { setLanguage, t } from "./i18n";
import { buildKeyboard, fallbackGlyph, type KeyboardSlot } from "./keys";
import { kobito } from "./kobito";
import type { LauncherConfig, LauncherItem, WindowActionBinding } from "./types";

type DisplayEntry = {
  kind: LauncherItem["type"] | "window";
  name: string;
  key: string;
  iconPath?: string;
  action?: string;
};

/** 表盤面なら偽、裏盤面（Shift）なら真の項目だけを出す。 */
function entries(config: LauncherConfig, back: boolean): DisplayEntry[] {
  const targets = config.items.filter((item) => (item.shift ?? false) === back).map(
    (item) =>
      ({
        kind: item.type,
        name: item.name,
        key: item.key,
        iconPath: item.icon.value,
      }) satisfies DisplayEntry,
  );
  const windows = config.window_actions
    .filter(
      (item): item is WindowActionBinding & { key: string } =>
        Boolean(item.key) && (item.shift ?? false) === back,
    )
    .map((item) => {
      const key = `action.${item.action}` as const;
      const translated = t(key as never);
      return {
        kind: "window" as const,
        name: translated === key ? item.name : translated,
        key: item.key,
        action: item.action,
      };
    });
  return [...windows, ...targets];
}

/** Finderのアイコンをそのまま出す。大きさはキーの枠に合わせるのでCSSに委ねる。 */
function visual(entry: DisplayEntry): HTMLElement {
  const icon = document.createElement("div");
  icon.className = "launcher-icon";
  if (entry.iconPath) {
    icon.classList.add("bare-icon");
    const image = document.createElement("img");
    image.src = convertFileSrc(entry.iconPath);
    image.alt = "";
    image.draggable = false;
    icon.append(image);
  } else if (entry.kind === "window") {
    icon.classList.add("window-action-icon");
    const diagram = document.createElement("span");
    diagram.className = "window-diagram";
    diagram.dataset.action = entry.action;
    icon.append(diagram);
  } else {
    icon.classList.add("fallback-icon");
    icon.textContent = fallbackGlyph(entry.kind, entry.name);
  }
  return icon;
}

function paint(root: HTMLElement, config: LauncherConfig, back: boolean): void {
  setLanguage(config.language);
  root.replaceChildren();
  const panel = document.createElement("main");
  panel.className = "launcher-panel";
  panel.classList.toggle("back-side", back);
  const allEntries = entries(config, back);
  // 裏盤面は空でも盤面を出し、Shiftで切り替わったことが分かるようにする。
  if (allEntries.length === 0 && !back) {
    const empty = document.createElement("div");
    empty.className = "popup-empty";
    const message = document.createElement("span");
    message.textContent = t("popup.empty");
    empty.append(kobito("kobito-empty"), message);
    panel.append(empty);
  } else {
    const slots = new Map<string, KeyboardSlot>(
      allEntries.map((entry) => [
        entry.key,
        { name: entry.name, node: () => visual(entry) },
      ]),
    );
    panel.append(buildKeyboard(config.keyboard_layout, slots, { variant: "popup" }));
    if (back) {
      const badge = document.createElement("span");
      badge.className = "layer-badge";
      badge.textContent = t("layer.backBadge");
      panel.append(badge);
    }
    // 最下段の左はどの配列でも空いているので、そこに小人を立たせる。
    panel.append(kobito("kobito-popup"));
  }
  root.append(panel);
}

/** 表示のたびにキーが浮き上がるよう、アニメーションを再生し直す。 */
function replayEntrance(root: HTMLElement): void {
  const panel = root.querySelector<HTMLElement>(".launcher-panel");
  if (!panel) return;
  panel.classList.remove("rise");
  void panel.offsetWidth;
  panel.classList.add("rise");
}

/** Shiftで盤面を裏返したときは、立ち上がりより短い反転の演出にする。 */
function replayFlip(root: HTMLElement): void {
  const panel = root.querySelector<HTMLElement>(".launcher-panel");
  if (!panel) return;
  panel.classList.remove("flip");
  void panel.offsetWidth;
  panel.classList.add("flip");
}

export async function renderPopup(root: HTMLElement): Promise<void> {
  let config = await invoke<LauncherConfig>("get_config");
  let back = false;
  paint(root, config, back);
  replayEntrance(root);
  await listen<LauncherConfig>("config-changed", ({ payload }) => {
    config = payload;
    paint(root, config, back);
    replayEntrance(root);
  });
  await listen<boolean>("popup-layer", ({ payload }) => {
    if (payload === back) return;
    back = payload;
    paint(root, config, back);
    replayFlip(root);
  });
  await listen("popup-shown", () => replayEntrance(root));
}
