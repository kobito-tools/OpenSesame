import "./styles.css";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { renderPopup } from "./popup";
import { renderSettings } from "./settings";

const root = document.querySelector<HTMLDivElement>("#app");

if (!root) {
  throw new Error("#app was not found");
}

/**
 * 設定画面の描画に失敗すると、何も出ないまま原因も分からなくなる。
 * 読み込み中の表示と、失敗したときの内容を画面に残す。
 */
function showStartupMessage(target: HTMLElement, title: string, detail?: string) {
  target.replaceChildren();
  const box = document.createElement("div");
  box.className = "startup-message";
  const heading = document.createElement("strong");
  heading.textContent = title;
  box.append(heading);
  if (detail) {
    const pre = document.createElement("pre");
    pre.textContent = detail;
    box.append(pre);
  }
  target.append(box);
}

function describe(error: unknown): string {
  if (error instanceof Error) return error.stack ?? `${error.name}: ${error.message}`;
  return String(error);
}

if (getCurrentWindow().label === "popup") {
  document.body.classList.add("popup-body");
  void renderPopup(root);
} else {
  document.body.classList.add("settings-body");
  showStartupMessage(root, "読み込み中… / Loading…");
  renderSettings(root).catch((error: unknown) => {
    showStartupMessage(root, "設定画面を表示できません / Cannot show settings", describe(error));
  });
}
