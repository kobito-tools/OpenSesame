import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { activationLabel, isModifierCode, sortModifiers } from "./activation";
import { APP_NAME, setLanguage, t, translateError, type Language } from "./i18n";
import { kobito } from "./kobito";
import {
  ASSIGNABLE_KEYS,
  buildKeyboard,
  displayKey,
  KEY_GROUPS,
  KEYBOARD_LAYOUTS_ORDER,
  layoutKeys,
  type KeyboardSlot,
} from "./keys";
import type {
  KeyboardLayout,
  LauncherConfig,
  LauncherItem,
  Platform,
  RuntimeInfo,
  WindowActionBinding,
} from "./types";

/** 削除キーで録音を始める。macOSの delete は Backspace として届く。 */
const CLEAR_KEYS = new Set(["Backspace", "Delete"]);

function createButton(label: string, className = "button"): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = className;
  button.textContent = label;
  return button;
}

/** 登録済みのアイコンはFinderのアイコンをそのまま出す。未取得のときだけ代替表示。 */
function targetIcon(item: LauncherItem): HTMLElement {
  const visual = document.createElement("div");
  visual.className = "table-icon";
  if (item.icon.value) {
    visual.classList.add("bare-icon");
    const image = document.createElement("img");
    image.src = convertFileSrc(item.icon.value);
    image.alt = "";
    visual.append(image);
  } else {
    visual.textContent = item.type === "folder" ? "📁" : item.name.slice(0, 1).toUpperCase();
  }
  return visual;
}

function windowIcon(action: string): HTMLElement {
  const visual = document.createElement("div");
  visual.className = "table-icon window-table-icon";
  const diagram = document.createElement("span");
  diagram.className = "window-diagram";
  diagram.dataset.action = action;
  visual.append(diagram);
  return visual;
}

function sectionLabel(title: string, description?: string): HTMLElement {
  const heading = document.createElement("div");
  heading.className = "table-section-label";
  const strong = document.createElement("strong");
  strong.textContent = title;
  heading.append(strong);
  if (description) {
    const span = document.createElement("span");
    span.textContent = description;
    heading.append(span);
  }
  return heading;
}

/** 選択肢が少ない設定はセグメント式のトグルで見せる。 */
function segmented<T extends string>(
  options: Array<[T, string]>,
  current: T,
  onChange: (value: T) => void,
): HTMLElement {
  const group = document.createElement("div");
  group.className = "segmented";
  group.setAttribute("role", "radiogroup");
  for (const [value, label] of options) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "segment";
    button.setAttribute("role", "radio");
    button.setAttribute("aria-checked", String(value === current));
    button.textContent = label;
    if (value === current) button.classList.add("active");
    button.addEventListener("click", () => {
      if (value !== current) onChange(value);
    });
    group.append(button);
  }
  return group;
}

export async function renderSettings(root: HTMLElement): Promise<void> {
  let config = await invoke<LauncherConfig>("get_config");
  const runtime = await invoke<RuntimeInfo>("runtime_info");
  const platform: Platform = runtime.platform;
  let statusTimer: number | undefined;
  /** キーボード監視の状態。Rust側から随時更新される。 */
  let inputError: string | null = runtime.input_error ?? null;
  setLanguage(config.language);

  /** 起動キー録音の状態。ホバー中に delete を押すと開始する。 */
  const recorder = { recording: false, pending: new Set<string>() };

  const showStatus = (message: string, kind: "success" | "error" = "success") => {
    const status = root.querySelector<HTMLElement>("[data-status]");
    if (!status) return;
    status.textContent = message;
    status.dataset.kind = kind;
    window.clearTimeout(statusTimer);
    statusTimer = window.setTimeout(() => {
      status.textContent = "";
    }, 3600);
  };

  const save = async () => {
    try {
      config = await invoke<LauncherConfig>("save_config", { config });
      setLanguage(config.language);
      paint();
      showStatus(t("status.saved"));
    } catch (error) {
      config = await invoke<LauncherConfig>("get_config");
      setLanguage(config.language);
      paint();
      showStatus(translateError(error), "error");
    }
  };

  const addTarget = async (type: "application" | "folder", key?: string) => {
    try {
      const item = await invoke<LauncherItem | null>("select_target", {
        targetType: type,
        key: key ?? null,
      });
      if (!item) return;
      config.items.push(item);
      await save();
    } catch (error) {
      showStatus(translateError(error), "error");
    }
  };

  const occupiedByOther = (ownerId: string, key: string) => {
    const itemUses = config.items.some((item) => item.id !== ownerId && item.key === key);
    const actionUses = config.window_actions.some(
      (item) => `window-${item.action}` !== ownerId && item.key === key,
    );
    return itemUses || actionUses;
  };

  const keySelect = (
    ownerId: string,
    current: string | undefined,
    allowEmpty: boolean,
    onChange: (key: string | undefined) => void,
  ): HTMLSelectElement => {
    const select = document.createElement("select");
    select.className = "key-select";
    select.setAttribute("aria-label", t("aria.keyAssign"));
    if (allowEmpty) {
      const empty = document.createElement("option");
      empty.value = "";
      empty.textContent = "—";
      empty.selected = !current;
      select.append(empty);
    }
    for (const group of KEY_GROUPS) {
      const optgroup = document.createElement("optgroup");
      optgroup.label = t(group.title);
      for (const key of group.codes) {
        const option = document.createElement("option");
        option.value = key;
        option.textContent = displayKey(key);
        option.selected = current === key;
        option.disabled = occupiedByOther(ownerId, key);
        optgroup.append(option);
      }
      select.append(optgroup);
    }
    select.addEventListener("change", () => onChange(select.value || undefined));
    return select;
  };

  /** ウィンドウ整形の名前は設定ファイルの値ではなく、現在の言語から引く。 */
  const actionName = (binding: WindowActionBinding): string => {
    const key = `action.${binding.action}` as const;
    const translated = t(key as never);
    return translated === key ? binding.name : translated;
  };

  /** 現在の配列に存在しないキーは、配列図に出ないので一覧側で知らせる。 */
  const offLayout = (key: string | undefined) =>
    Boolean(key) && !layoutKeys(config.keyboard_layout).has(key as string);

  const offLayoutBadge = (): HTMLElement => {
    const badge = document.createElement("span");
    badge.className = "off-layout-badge";
    badge.textContent = t("badge.offLayout");
    return badge;
  };

  const windowRow = (binding: WindowActionBinding): HTMLElement => {
    const row = document.createElement("article");
    row.className = "item-row window-action-row";
    if (binding.key) row.dataset.key = binding.key;
    if (offLayout(binding.key)) row.classList.add("off-layout");
    row.append(windowIcon(binding.action));
    const detail = document.createElement("div");
    detail.className = "item-detail";
    const name = document.createElement("strong");
    name.textContent = actionName(binding);
    const hint = document.createElement("span");
    hint.className = "target-path";
    hint.textContent = t("item.windowHint");
    detail.append(name, hint);
    const select = keySelect(`window-${binding.action}`, binding.key, true, (key) => {
      binding.key = key;
      void save();
    });
    const badge = document.createElement("span");
    badge.className = "built-in-badge";
    badge.textContent = t("badge.builtin");
    row.append(detail, select, offLayout(binding.key) ? offLayoutBadge() : badge);
    return row;
  };

  const itemRow = (item: LauncherItem): HTMLElement => {
    const row = document.createElement("article");
    row.className = "item-row";
    row.dataset.key = item.key;
    if (offLayout(item.key)) row.classList.add("off-layout");
    row.append(targetIcon(item));
    const detail = document.createElement("div");
    detail.className = "item-detail";
    const name = document.createElement("input");
    name.className = "name-input";
    name.value = item.name;
    name.setAttribute("aria-label", t("aria.displayName"));
    name.addEventListener("change", () => {
      item.name = name.value.trim() || item.name;
      void save();
    });
    const path = document.createElement("span");
    path.className = "target-path";
    path.textContent = item.target.value;
    path.title = item.target.value;
    detail.append(name, path);
    const select = keySelect(item.id, item.key, false, (key) => {
      if (key) item.key = key;
      void save();
    });
    const iconButton = createButton(t("button.icon"), "button compact");
    iconButton.addEventListener("click", async () => {
      try {
        const updated = await invoke<LauncherItem | null>("select_custom_icon", { itemId: item.id });
        if (!updated) return;
        config.items = config.items.map((entry) => (entry.id === updated.id ? updated : entry));
        await save();
      } catch (error) {
        showStatus(translateError(error), "error");
      }
    });
    const remove = createButton(t("button.delete"), "button compact danger");
    remove.addEventListener("click", () => {
      config.items = config.items.filter((entry) => entry.id !== item.id);
      void save();
    });
    const rowActions = document.createElement("div");
    rowActions.className = "row-actions";
    if (offLayout(item.key)) rowActions.append(offLayoutBadge());
    rowActions.append(iconButton, remove);
    row.append(detail, select, rowActions);
    return row;
  };

  /** 割り当て済みのキーは、下半分の該当行へ移動して一時的に強調する。 */
  const revealRow = (code: string) => {
    const row = root.querySelector<HTMLElement>(`.item-row[data-key="${CSS.escape(code)}"]`);
    if (!row) return false;
    row.scrollIntoView({ behavior: "smooth", block: "center" });
    row.classList.add("highlight");
    window.setTimeout(() => row.classList.remove("highlight"), 1400);
    return true;
  };

  const closePicker = () => root.querySelector(".picker-backdrop")?.remove();

  /** 空いているキーを押したときの3択。選ぶとOSの選択UIか操作一覧へ進む。 */
  const openPicker = (code: string) => {
    closePicker();
    const backdrop = document.createElement("div");
    backdrop.className = "picker-backdrop";
    backdrop.addEventListener("click", (event) => {
      if (event.target === backdrop) closePicker();
    });
    const card = document.createElement("div");
    card.className = "picker";
    card.setAttribute("role", "dialog");
    card.setAttribute("aria-modal", "true");
    backdrop.append(card);

    const paintChoices = () => {
      card.replaceChildren();
      const title = document.createElement("strong");
      title.textContent = t("picker.title", { key: displayKey(code) });
      const lead = document.createElement("p");
      lead.textContent = t("picker.lead");
      const list = document.createElement("div");
      list.className = "picker-choices";
      const choices: Array<[string, string, () => void]> = [
        ["picker.window", "picker.windowDesc", paintWindowActions],
        [
          "picker.app",
          "picker.appDesc",
          () => {
            closePicker();
            void addTarget("application", code);
          },
        ],
        [
          "picker.folder",
          "picker.folderDesc",
          () => {
            closePicker();
            void addTarget("folder", code);
          },
        ],
      ];
      for (const [label, description, run] of choices) {
        const choice = document.createElement("button");
        choice.type = "button";
        choice.className = "picker-choice";
        const strong = document.createElement("strong");
        strong.textContent = t(label as never);
        const small = document.createElement("small");
        small.textContent = t(description as never);
        choice.append(strong, small);
        choice.addEventListener("click", run);
        list.append(choice);
      }
      const cancel = createButton(t("picker.cancel"), "button compact");
      cancel.addEventListener("click", closePicker);
      const footer = document.createElement("div");
      footer.className = "picker-footer";
      footer.append(cancel);
      card.append(title, lead, list, footer);
    };

    const paintWindowActions = () => {
      card.replaceChildren();
      const title = document.createElement("strong");
      title.textContent = t("picker.chooseAction", { key: displayKey(code) });
      const list = document.createElement("div");
      list.className = "picker-actions";
      for (const binding of config.window_actions) {
        const choice = document.createElement("button");
        choice.type = "button";
        choice.className = "picker-action";
        const name = document.createElement("span");
        name.textContent = actionName(binding);
        choice.append(windowIcon(binding.action), name);
        choice.addEventListener("click", () => {
          // 同じ操作が別のキーに付いていたら、こちらへ付け替える。
          binding.key = code;
          closePicker();
          void save();
        });
        list.append(choice);
      }
      const back = createButton(t("picker.back"), "button compact");
      back.addEventListener("click", paintChoices);
      const footer = document.createElement("div");
      footer.className = "picker-footer";
      footer.append(back);
      card.append(title, list, footer);
    };

    paintChoices();
    root.append(backdrop);
  };

  const onKeyboardSelect = (code: string) => {
    if (!revealRow(code)) openPicker(code);
  };

  const keyboardPreview = (): HTMLElement => {
    const slots = new Map<string, KeyboardSlot>();
    for (const binding of config.window_actions) {
      if (binding.key) {
        slots.set(binding.key, { name: actionName(binding), node: () => windowIcon(binding.action) });
      }
    }
    for (const item of config.items) {
      slots.set(item.key, { name: item.name, node: () => targetIcon(item) });
    }
    return buildKeyboard(config.keyboard_layout, slots, {
      variant: "settings",
      onSelect: onKeyboardSelect,
    });
  };

  const paintItems = (container: HTMLElement) => {
    const order = (key: string | undefined) =>
      key ? ASSIGNABLE_KEYS.indexOf(key) : Number.MAX_SAFE_INTEGER;
    const sortedItems = [...config.items].sort((a, b) => order(a.key) - order(b.key));
    const sortedWindows = [...config.window_actions].sort((a, b) => order(a.key) - order(b.key));
    const apps = sortedItems.filter((item) => item.type === "application");
    const folders = sortedItems.filter((item) => item.type === "folder");
    container.append(sectionLabel(t("section.window"), t("section.window.desc")));
    for (const action of sortedWindows) container.append(windowRow(action));
    container.append(sectionLabel(t("section.apps"), t("section.count", { count: apps.length })));
    for (const item of apps) container.append(itemRow(item));
    container.append(
      sectionLabel(t("section.folders"), t("section.count", { count: folders.length })),
    );
    for (const item of folders) container.append(itemRow(item));
  };

  /**
   * 起動キーは現在の組み合わせだけを出し、ホバー中の delete で録音を始める。
   * 押されている修飾キーを集め、最初のキーを離した時点で確定する。
   */
  const commitRecording = () => {
    const captured = sortModifiers([...recorder.pending]);
    recorder.recording = false;
    recorder.pending.clear();
    if (captured.length === 0) {
      paint();
      return;
    }
    if (captured.length > 3) {
      paint();
      showStatus(t("activation.tooMany"), "error");
      return;
    }
    config.activation[platform] = captured;
    void save();
  };

  const cancelRecording = () => {
    recorder.recording = false;
    recorder.pending.clear();
    paint();
  };

  const updateRecorderDisplay = () => {
    const value = root.querySelector<HTMLElement>("[data-activation-value]");
    if (!value) return;
    const captured = sortModifiers([...recorder.pending]);
    value.textContent =
      captured.length > 0
        ? activationLabel(platform, config.language, captured)
        : t("activation.recording");
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (!recorder.recording && event.key === "Escape" && root.querySelector(".picker-backdrop")) {
      event.preventDefault();
      closePicker();
      return;
    }
    if (recorder.recording) {
      event.preventDefault();
      if (event.key === "Escape") {
        cancelRecording();
        return;
      }
      if (isModifierCode(event.code)) {
        recorder.pending.add(event.code);
        updateRecorderDisplay();
      } else {
        showStatus(t("activation.modifiersOnly"), "error");
      }
      return;
    }
    // 再描画でホバー状態が失われないよう、その場でDOMに問い合わせる。
    const area = root.querySelector<HTMLElement>("[data-activation]");
    const aimed = area && (area.matches(":hover") || area === document.activeElement);
    if (!aimed || !CLEAR_KEYS.has(event.key)) return;
    event.preventDefault();
    recorder.recording = true;
    recorder.pending.clear();
    area.classList.add("recording");
    updateRecorderDisplay();
  };

  const onKeyUp = (event: KeyboardEvent) => {
    if (!recorder.recording) return;
    event.preventDefault();
    if (recorder.pending.size > 0) commitRecording();
  };

  window.addEventListener("keydown", onKeyDown, true);
  window.addEventListener("keyup", onKeyUp, true);

  const activationRecorder = (): HTMLElement => {
    const area = document.createElement("div");
    area.className = "activation-recorder";
    area.tabIndex = 0;
    area.dataset.activation = "";
    area.setAttribute("role", "button");
    area.setAttribute("aria-label", t("activation.title"));
    const value = document.createElement("strong");
    value.dataset.activationValue = "";
    const current = sortModifiers(config.activation[platform]);
    value.textContent =
      current.length > 0
        ? activationLabel(platform, config.language, current)
        : t("activation.empty");
    const hint = document.createElement("small");
    hint.textContent = t("activation.hint");
    area.append(value, hint);
    return area;
  };

  /** アクセシビリティ権限は場所が分かりにくいので、開くボタンと手順を出す。 */
  const permissionCard = (): HTMLElement => {
    const card = document.createElement("aside");
    card.className = "permission-card";
    const title = document.createElement("strong");
    title.textContent = t("permission.title");
    const lead = document.createElement("p");
    lead.textContent = t("permission.lead", { app: APP_NAME });
    const steps = document.createElement("ol");
    for (const key of ["permission.step1", "permission.step2", "permission.step3"] as const) {
      const step = document.createElement("li");
      step.textContent = t(key, { app: APP_NAME });
      steps.append(step);
    }
    const note = document.createElement("p");
    note.className = "permission-note-line";
    note.textContent = t("permission.note");
    const open = createButton(t("permission.button"), "button primary");
    open.addEventListener("click", async () => {
      try {
        await invoke("open_accessibility_settings");
      } catch (error) {
        showStatus(translateError(error), "error");
      }
    });
    card.append(title, lead, steps, note, open);
    if (inputError) {
      const detail = document.createElement("details");
      const summary = document.createElement("summary");
      summary.textContent = t("permission.detail");
      const text = document.createElement("span");
      text.textContent = translateError(inputError);
      detail.append(summary, text);
      card.append(detail);
    }
    return card;
  };

  /** 起動キーが効かない原因になりやすい事柄を1行にまとめる。 */
  const notices = (): string[] => {
    if (platform !== "macos") return [];
    const messages: string[] = [];
    const current = sortModifiers(config.activation[platform]);
    if (current.includes("ControlLeft") && current.includes("AltLeft")) {
      messages.push(t("notice.voiceover"));
    }
    return messages;
  };

  const paint = () => {
    const keyboardScroll = root.querySelector<HTMLElement>("[data-keyboard]")?.scrollLeft ?? 0;
    const listScroll = root.querySelector<HTMLElement>("[data-items]")?.scrollTop ?? 0;
    root.replaceChildren();
    const shell = document.createElement("main");
    shell.className = "settings-shell";
    const noticeMessages = notices();
    shell.innerHTML = `
      <header class="settings-header">
        <div class="header-title">
          <p class="eyebrow">${t("app.eyebrow")}</p>
          <h1>${t("app.title")}</h1>
          <p class="subtitle">${t("app.subtitle")}</p>
        </div>
        <div class="header-controls">
          <div class="header-field">
            <span class="field-label">${t("language.name")}</span>
            <div data-language></div>
          </div>
          <div class="header-field">
            <span class="field-label">${t("layout.title")}</span>
            <div data-keyboard-layout></div>
          </div>
        </div>
      </header>
      <section class="activation-card">
        <span class="field-label">${t("activation.title")}</span>
        <div data-activation-slot></div>
      </section>
      <div data-permission></div>
      ${noticeMessages.length > 0 ? `<aside class="permission-note"><strong>${t("notice.title")}</strong><span>${noticeMessages.join(" ")}</span></aside>` : ""}
      <section class="settings-split">
        <div class="split-pane keyboard-pane">
          <div class="pane-heading">
            <div><h2>${t("pane.keyboard.title")}</h2><p>${t("pane.keyboard.desc")}</p></div>
          </div>
          <div class="keyboard-scroll" data-keyboard></div>
        </div>
        <div class="split-pane list-pane">
          <div class="pane-heading">
            <div><h2>${t("pane.list.title")}</h2><p>${t("pane.list.desc")}</p></div>
            <div class="button-row" data-actions></div>
          </div>
          <div class="items-table" data-items></div>
        </div>
      </section>
      <footer class="settings-footer"><span data-status aria-live="polite"></span><span class="config-path" data-config-path>${t("footer.configPath")}</span></footer>
    `;

    // パスに引用符などが含まれてもHTMLが壊れないよう、文字列には埋め込まずDOMで設定する。
    const configPath = shell.querySelector<HTMLElement>("[data-config-path]");
    if (configPath) configPath.title = runtime.config_path;
    if (inputError) shell.querySelector("[data-permission]")?.append(permissionCard());
    shell.querySelector(".settings-header")?.prepend(kobito("kobito-settings"));
    shell.querySelector("[data-activation-slot]")?.append(activationRecorder());
    shell.querySelector("[data-language]")?.append(
      segmented<Language>(
        [
          ["ja", t("language.ja")],
          ["en", t("language.en")],
        ],
        config.language,
        (value) => {
          config.language = value;
          void save();
        },
      ),
    );
    shell.querySelector("[data-keyboard-layout]")?.append(
      segmented<KeyboardLayout>(
        KEYBOARD_LAYOUTS_ORDER.map((mode) => [mode, t(`layout.${mode}`)] as [KeyboardLayout, string]),
        config.keyboard_layout,
        (value) => {
          config.keyboard_layout = value;
          void save();
        },
      ),
    );

    const actions = shell.querySelector<HTMLElement>("[data-actions]");
    const addApp = createButton(t("button.addApp"), "button primary");
    const addFolder = createButton(t("button.addFolder"));
    addApp.addEventListener("click", () => void addTarget("application"));
    addFolder.addEventListener("click", () => void addTarget("folder"));
    actions?.append(addApp, addFolder);

    shell.querySelector<HTMLElement>("[data-keyboard]")?.append(keyboardPreview());
    const items = shell.querySelector<HTMLElement>("[data-items]");
    if (items) paintItems(items);
    root.append(shell);

    const keyboard = root.querySelector<HTMLElement>("[data-keyboard]");
    if (keyboard) keyboard.scrollLeft = keyboardScroll;
    const list = root.querySelector<HTMLElement>("[data-items]");
    if (list) list.scrollTop = listScroll;
  };

  paint();
  // 権限が付与されるとRust側が自動で監視を開始するので、案内の出し入れも追従させる。
  await listen<string | null>("input-monitor-status", ({ payload }) => {
    const next = payload ?? null;
    if (next === inputError) return;
    const recovered = Boolean(inputError) && !next;
    inputError = next;
    paint();
    if (recovered) showStatus(t("permission.granted"));
  });
}
