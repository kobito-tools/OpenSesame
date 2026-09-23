import "./styles.css";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { renderPopup } from "./popup";
import { renderSettings } from "./settings";

const root = document.querySelector<HTMLDivElement>("#app");

if (!root) {
  throw new Error("#app was not found");
}

if (getCurrentWindow().label === "popup") {
  document.body.classList.add("popup-body");
  void renderPopup(root);
} else {
  document.body.classList.add("settings-body");
  void renderSettings(root);
}

