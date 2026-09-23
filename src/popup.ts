import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { setLanguage, t } from "./i18n";
import { buildKeyboard, type KeyboardSlot } from "./keys";
import type { LauncherConfig, WindowActionBinding } from "./types";

type DisplayEntry = {
  kind: "application" | "folder" | "window";
  name: string;
  key: string;
  iconPath?: string;
  action?: string;
};

function entries(config: LauncherConfig): DisplayEntry[] {
  const targets = config.items.map(
    (item) =>
      ({
        kind: item.type,
        name: item.name,
        key: item.key,
        iconPath: item.icon.value,
      }) satisfies DisplayEntry,
  );
  const windows = config.window_actions
    .filter((item): item is WindowActionBinding & { key: string } => Boolean(item.key))
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
    icon.textContent = entry.kind === "folder" ? "📁" : entry.name.slice(0, 1).toUpperCase();
  }
  return icon;
}

function paint(root: HTMLElement, config: LauncherConfig): void {
  setLanguage(config.language);
  root.replaceChildren();
  const panel = document.createElement("main");
  panel.className = "launcher-panel";
  const allEntries = entries(config);
  if (allEntries.length === 0) {
    const empty = document.createElement("div");
    empty.className = "popup-empty";
    empty.textContent = t("popup.empty");
    panel.append(empty);
  } else {
    const slots = new Map<string, KeyboardSlot>(
      allEntries.map((entry) => [
        entry.key,
        { name: entry.name, node: () => visual(entry) },
      ]),
    );
    panel.append(buildKeyboard(config.keyboard_layout, slots, { variant: "popup" }));
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

export async function renderPopup(root: HTMLElement): Promise<void> {
  const config = await invoke<LauncherConfig>("get_config");
  paint(root, config);
  replayEntrance(root);
  await listen<LauncherConfig>("config-changed", ({ payload }) => {
    paint(root, payload);
    replayEntrance(root);
  });
  await listen("popup-shown", () => replayEntrance(root));
}
