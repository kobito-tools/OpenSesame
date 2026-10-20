import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { activationLabel, isModifierCode, sortModifiers } from "./activation";
import { APP_NAME, setLanguage, t, translateError, type Language } from "./i18n";
import { kobito } from "./kobito";
import {
  ASSIGNABLE_KEYS,
  buildKeyboard,
  displayKey,
  fallbackGlyph,
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
  SeriesApp,
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
    visual.textContent = fallbackGlyph(item.type, item.name);
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

  /** 表示中の盤面。真なら裏盤面（Shift）で、キーボード図も一覧もこちらに揃える。 */
  let back = false;
  const onLayer = (entry: { shift?: boolean }) => (entry.shift ?? false) === back;

  /** 起動キー録音の状態。ホバー中に delete を押すと開始する。 */
  const recorder = { recording: false, pending: new Set<string>() };

  const showStatus = (message: string, kind: "success" | "error" = "success", duration = 3600) => {
    const status = root.querySelector<HTMLElement>("[data-status]");
    if (!status) return;
    status.textContent = message;
    status.dataset.kind = kind;
    window.clearTimeout(statusTimer);
    statusTimer = window.setTimeout(() => {
      status.textContent = "";
    }, duration);
  };

  /** 保存できたかどうかを返す。失敗時は保存済みの設定へ戻す。 */
  const save = async (): Promise<boolean> => {
    try {
      config = await invoke<LauncherConfig>("save_config", { config });
      setLanguage(config.language);
      paint();
      showStatus(t("status.saved"));
      return true;
    } catch (error) {
      config = await invoke<LauncherConfig>("get_config");
      setLanguage(config.language);
      paint();
      showStatus(translateError(error), "error");
      return false;
    }
  };

  const addTarget = async (type: "application" | "folder", key?: string) => {
    try {
      const item = await invoke<LauncherItem | null>("select_target", {
        targetType: type,
        key: key ?? null,
        shift: back,
      });
      if (!item) return;
      config.items.push(item);
      await save();
    } catch (error) {
      showStatus(translateError(error), "error");
    }
  };

  /** 登録できたかどうかを返す。失敗時は入力欄を残して直せるようにする。 */
  const addWebsite = async (url: string, name: string, key?: string): Promise<boolean> => {
    try {
      const item = await invoke<LauncherItem>("create_website", {
        url,
        name: name || null,
        key: key ?? null,
        shift: back,
      });
      config.items.push(item);
      return await save();
    } catch (error) {
      showStatus(translateError(error), "error");
      return false;
    }
  };

  /** 登録できたかどうかを返す。失敗時は入力欄を残して直せるようにする。 */
  const addCommand = async (
    command: string,
    name: string,
    workingDir: string,
    key?: string,
  ): Promise<boolean> => {
    try {
      const item = await invoke<LauncherItem>("create_command", {
        command,
        name: name || null,
        workingDir: workingDir || null,
        key: key ?? null,
        shift: back,
      });
      config.items.push(item);
      return await save();
    } catch (error) {
      showStatus(translateError(error), "error");
      return false;
    }
  };

  /** 設定に覚えるのはアプリIDと操作の id だけ。URL は押されたときに Rust 側が起動情報から読む。 */
  const addSeriesAction = async (app: string, action: string, key?: string): Promise<boolean> => {
    try {
      const item = await invoke<LauncherItem>("create_series_action", {
        app,
        action,
        key: key ?? null,
        shift: back,
      });
      config.items.push(item);
      return await save();
    } catch (error) {
      showStatus(translateError(error), "error");
      return false;
    }
  };

  const occupiedByOther = (ownerId: string, key: string) => {
    const itemUses = config.items.some(
      (item) => item.id !== ownerId && onLayer(item) && item.key === key,
    );
    const actionUses = config.window_actions.some(
      (item) => `window-${item.action}` !== ownerId && onLayer(item) && item.key === key,
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
      binding.shift = Boolean(key) && back;
      void save();
    });
    const badge = document.createElement("span");
    badge.className = "built-in-badge";
    badge.textContent = t("badge.builtin");
    row.append(detail, select, offLayout(binding.key) ? offLayoutBadge() : badge);
    return row;
  };

  const hideToggle = (item: LauncherItem): HTMLElement => {
    const label = document.createElement("label");
    label.className = "hide-toggle";
    label.title = t("item.hideWhenFrontmostHint");
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = item.hide_when_frontmost ?? false;
    checkbox.addEventListener("change", () => {
      item.hide_when_frontmost = checkbox.checked;
      void save();
    });
    const text = document.createElement("span");
    text.textContent = t("item.hideWhenFrontmost");
    label.append(checkbox, text);
    return label;
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
    const dir = item.type === "command" ? item.target.working_dir : undefined;
    if (item.type === "kobito-action") {
      path.textContent = t("item.seriesAction", {
        app: item.target.app ?? "",
        action: item.target.action ?? "",
      });
    } else {
      path.textContent = dir
        ? t("item.commandIn", { command: item.target.value ?? "", dir })
        : (item.target.value ?? "");
    }
    path.title = path.textContent;
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
    const refetch = createButton(t("button.refetchIcon"), "button compact");
    refetch.addEventListener("click", async () => {
      refetch.disabled = true;
      showStatus(t("status.fetchingIcon"));
      try {
        const updated = await invoke<LauncherItem>("refetch_website_icon", { itemId: item.id });
        config.items = config.items.map((entry) => (entry.id === updated.id ? updated : entry));
        await save();
      } catch (error) {
        refetch.disabled = false;
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
    if (item.type === "application") rowActions.append(hideToggle(item));
    if (item.type === "website") rowActions.append(refetch);
    if (item.type === "command") {
      const edit = createButton(t("button.editCommand"), "button compact");
      edit.addEventListener("click", () => openPicker(undefined, item));
      rowActions.append(edit);
    }
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

  /**
   * 空いているキーを押したときの選択肢。選ぶとOSの選択UIか操作一覧、URLやコマンドの入力へ進む。
   * `code` を省くと `start` の入力欄から始め、空いているキーを自動で割り当てる。
   * `start` に登録済みのコマンドを渡すと、その編集になる。
   */
  const openPicker = (
    code?: string,
    start: "website" | "command" | "series" | LauncherItem = "website",
  ) => {
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
      title.textContent = t("picker.title", { key: displayKey(code ?? "") });
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
        ["picker.website", "picker.websiteDesc", paintWebsiteForm],
        ["picker.command", "picker.commandDesc", () => paintCommandForm()],
        ["picker.series", "picker.seriesDesc", () => void paintSeriesActions()],
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
      title.textContent = t("picker.chooseAction", { key: displayKey(code ?? "") });
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
          if (code) {
            binding.key = code;
            binding.shift = back;
          }
          closePicker();
          void save();
        });
        list.append(choice);
      }
      const backButton = createButton(t("picker.back"), "button compact");
      backButton.addEventListener("click", paintChoices);
      const footer = document.createElement("div");
      footer.className = "picker-footer";
      footer.append(backButton);
      card.append(title, list, footer);
    };

    /**
     * シリーズのアプリが起動情報に名乗った操作を、アプリごとに並べる。
     * アプリの名前は起動情報から読んだものだけを出す。
     */
    const paintSeriesActions = async () => {
      card.replaceChildren();
      const title = document.createElement("strong");
      title.textContent = code
        ? t("picker.seriesTitle", { key: displayKey(code) })
        : t("picker.series");
      const lead = document.createElement("p");
      lead.textContent = t("picker.seriesLoading");
      const list = document.createElement("div");
      list.className = "picker-series";
      const backButton = createButton(t(code ? "picker.back" : "picker.cancel"), "button compact");
      backButton.addEventListener("click", code ? paintChoices : closePicker);
      const footer = document.createElement("div");
      footer.className = "picker-footer";
      footer.append(backButton);
      card.append(title, lead, list, footer);
      let apps: SeriesApp[] = [];
      try {
        apps = await invoke<SeriesApp[]>("list_series_actions");
      } catch (error) {
        showStatus(translateError(error), "error");
      }
      const english = config.language === "en";
      lead.textContent = t(apps.length > 0 ? "picker.seriesLead" : "picker.seriesEmpty");
      for (const app of apps) {
        const group = document.createElement("section");
        group.className = "picker-series-app";
        const heading = document.createElement("span");
        heading.className = "picker-series-name";
        heading.textContent = english ? (app.name_en ?? app.name) : app.name;
        const actions = document.createElement("div");
        actions.className = "picker-actions";
        for (const action of app.actions) {
          const choice = document.createElement("button");
          choice.type = "button";
          choice.className = "picker-action";
          choice.textContent = english ? (action.label_en ?? action.label) : action.label;
          choice.addEventListener("click", async () => {
            choice.disabled = true;
            if (await addSeriesAction(app.app, action.id, code)) closePicker();
            else choice.disabled = false;
          });
          actions.append(choice);
        }
        group.append(heading, actions);
        list.append(group);
      }
    };

    const field = (label: string, placeholder: string): [HTMLLabelElement, HTMLInputElement] => {
      const wrapper = document.createElement("label");
      wrapper.className = "picker-field";
      const caption = document.createElement("span");
      caption.textContent = label;
      const input = document.createElement("input");
      input.type = "text";
      input.placeholder = placeholder;
      input.spellcheck = false;
      input.autocapitalize = "off";
      wrapper.append(caption, input);
      return [wrapper, input];
    };

    const paintWebsiteForm = () => {
      card.replaceChildren();
      const title = document.createElement("strong");
      title.textContent = code
        ? t("picker.websiteTitle", { key: displayKey(code) })
        : t("picker.website");
      const form = document.createElement("form");
      form.className = "picker-form";
      const [urlField, urlInput] = field(t("picker.websiteUrl"), "https://example.com");
      urlInput.required = true;
      const [nameField, nameInput] = field(t("picker.websiteName"), t("picker.websiteNameHint"));
      const footer = document.createElement("div");
      footer.className = "picker-footer";
      // キーから来たときは種類の選択へ戻れるようにし、ボタンから来たときは閉じるだけにする。
      const backButton = createButton(t(code ? "picker.back" : "picker.cancel"), "button compact");
      backButton.addEventListener("click", code ? paintChoices : closePicker);
      const submit = createButton(t("picker.websiteAdd"), "button compact primary");
      submit.type = "submit";
      footer.append(backButton, submit);
      form.append(urlField, nameField, footer);
      form.addEventListener("submit", async (event) => {
        event.preventDefault();
        const url = urlInput.value.trim();
        if (!url) return;
        // アイコンを取りに行く数秒のあいだ、押せたかどうか分かるようにする。
        submit.disabled = true;
        submit.textContent = t("status.fetchingIcon");
        const added = await addWebsite(url, nameInput.value.trim(), code);
        if (added) {
          closePicker();
        } else {
          submit.disabled = false;
          submit.textContent = t("picker.websiteAdd");
        }
      });
      card.append(title, form);
      urlInput.focus();
    };

    /** `editing` を渡すと、その登録済みコマンドを書き換える。 */
    const paintCommandForm = (editing?: LauncherItem) => {
      card.replaceChildren();
      const title = document.createElement("strong");
      title.textContent = editing
        ? t("picker.commandEdit")
        : code
          ? t("picker.commandTitle", { key: displayKey(code) })
          : t("picker.command");
      const form = document.createElement("form");
      form.className = "picker-form";
      const [commandField, commandInput] = field(t("picker.commandLine"), t("picker.commandLineHint"));
      commandInput.required = true;
      const [dirField, dirInput] = field(t("picker.commandDir"), t("picker.commandDirHint"));
      const pickDir = createButton(t("picker.commandDirPick"), "button compact");
      pickDir.addEventListener("click", async () => {
        const chosen = await invoke<string | null>("pick_folder");
        if (chosen) dirInput.value = chosen;
      });
      const dirRow = document.createElement("div");
      dirRow.className = "picker-field-row";
      dirRow.append(dirInput, pickDir);
      dirField.append(dirRow);
      const [nameField, nameInput] = field(t("picker.commandName"), t("picker.commandNameHint"));
      if (editing) {
        commandInput.value = editing.target.value ?? "";
        dirInput.value = editing.target.working_dir ?? "";
        nameInput.value = editing.name;
      }
      const footer = document.createElement("div");
      footer.className = "picker-footer";
      const backButton = createButton(t(code ? "picker.back" : "picker.cancel"), "button compact");
      backButton.addEventListener("click", code ? paintChoices : closePicker);
      const submitLabel = t(editing ? "picker.commandSave" : "picker.websiteAdd");
      const submit = createButton(submitLabel, "button compact primary");
      submit.type = "submit";
      footer.append(backButton, submit);
      form.append(commandField, dirField, nameField, footer);
      form.addEventListener("submit", async (event) => {
        event.preventDefault();
        const command = commandInput.value.trim();
        if (!command) return;
        const workingDir = dirInput.value.trim();
        submit.disabled = true;
        if (editing) {
          // 保存に失敗したときは save が保存済みの設定へ戻すので、ここで書き換えてよい。
          editing.target.value = command;
          editing.target.working_dir = workingDir || undefined;
          editing.name = nameInput.value.trim() || editing.name;
          if (await save()) closePicker();
          return;
        }
        const added = await addCommand(command, nameInput.value.trim(), workingDir, code);
        if (added) closePicker();
        else submit.disabled = false;
      });
      card.append(title, form);
      commandInput.focus();
    };

    if (code) paintChoices();
    else if (start === "website") paintWebsiteForm();
    else if (start === "series") void paintSeriesActions();
    else paintCommandForm(start === "command" ? undefined : start);
    root.append(backdrop);
    if (!code) backdrop.querySelector<HTMLInputElement>(".picker-field input")?.focus();
  };

  const onKeyboardSelect = (code: string) => {
    if (!revealRow(code)) openPicker(code);
  };

  /**
   * キーボード上のドラッグで割り当てを別のキーへ移す。
   * 移動先に既に割り当てがあれば、2つのキーの割り当てを入れ替える。
   */
  const moveKey = (from: string, to: string) => {
    const owners = (key: string) => [
      ...config.items.filter((item) => onLayer(item) && item.key === key),
      ...config.window_actions.filter((binding) => onLayer(binding) && binding.key === key),
    ];
    const moving = owners(from);
    if (moving.length === 0) return;
    const displaced = owners(to);
    for (const owner of moving) owner.key = to;
    for (const owner of displaced) owner.key = from;
    void save().then((saved) => {
      if (!saved) return;
      const params = { from: displayKey(from), to: displayKey(to) };
      showStatus(t(displaced.length > 0 ? "status.swapped" : "status.moved", params));
    });
  };

  const keyboardPreview = (): HTMLElement => {
    const slots = new Map<string, KeyboardSlot>();
    for (const binding of config.window_actions) {
      if (binding.key && onLayer(binding)) {
        slots.set(binding.key, { name: actionName(binding), node: () => windowIcon(binding.action) });
      }
    }
    for (const item of config.items.filter(onLayer)) {
      slots.set(item.key, { name: item.name, node: () => targetIcon(item) });
    }
    const keyboard = buildKeyboard(config.keyboard_layout, slots, {
      variant: "settings",
      onSelect: onKeyboardSelect,
      onMove: moveKey,
    });
    keyboard.classList.toggle("keyboard-back", back);
    return keyboard;
  };

  const paintItems = (container: HTMLElement) => {
    const order = (key: string | undefined) =>
      key ? ASSIGNABLE_KEYS.indexOf(key) : Number.MAX_SAFE_INTEGER;
    const sortedItems = config.items.filter(onLayer).sort((a, b) => order(a.key) - order(b.key));
    // 未割り当ての操作はどちらの盤面にも出し、反対の盤面で使っている操作だけを隠す。
    const sortedWindows = config.window_actions
      .filter((binding) => !binding.key || onLayer(binding))
      .sort((a, b) => order(a.key) - order(b.key));
    const apps = sortedItems.filter((item) => item.type === "application");
    const folders = sortedItems.filter((item) => item.type === "folder");
    const websites = sortedItems.filter((item) => item.type === "website");
    const commands = sortedItems.filter((item) => item.type === "command");
    const seriesActions = sortedItems.filter((item) => item.type === "kobito-action");
    container.append(sectionLabel(t("section.window"), t("section.window.desc")));
    for (const action of sortedWindows) container.append(windowRow(action));
    container.append(sectionLabel(t("section.apps"), t("section.count", { count: apps.length })));
    for (const item of apps) container.append(itemRow(item));
    container.append(
      sectionLabel(t("section.folders"), t("section.count", { count: folders.length })),
    );
    for (const item of folders) container.append(itemRow(item));
    container.append(
      sectionLabel(t("section.websites"), t("section.count", { count: websites.length })),
    );
    for (const item of websites) container.append(itemRow(item));
    container.append(
      sectionLabel(t("section.commands"), t("section.count", { count: commands.length })),
    );
    for (const item of commands) container.append(itemRow(item));
    container.append(
      sectionLabel(t("section.series"), t("section.count", { count: seriesActions.length })),
    );
    for (const item of seriesActions) container.append(itemRow(item));
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

  /** 起動キーに含めたShiftは裏盤面の切り替えに使えないので、代わりのキーを知らせる。 */
  const layerNote = (): HTMLElement | null => {
    const current = config.activation[platform];
    const left = current.includes("ShiftLeft");
    const right = current.includes("ShiftRight");
    if (!left && !right) return null;
    const note = document.createElement("p");
    note.className = "activation-note";
    note.textContent = t(
      left && right ? "layer.unavailable" : left ? "layer.useRightShift" : "layer.useLeftShift",
    );
    return note;
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
    // 設定画面はページ全体が縦に動くので、描き直しの前後で位置を保つ。
    const pageScroll = root.scrollTop;
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
            <div data-layer></div>
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
    const note = layerNote();
    if (note) shell.querySelector(".activation-card")?.append(note);
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

    shell.querySelector("[data-layer]")?.append(
      segmented<"front" | "back">(
        [
          ["front", t("layer.front")],
          ["back", t("layer.back")],
        ],
        back ? "back" : "front",
        (value) => {
          back = value === "back";
          closePicker();
          paint();
        },
      ),
    );

    const actions = shell.querySelector<HTMLElement>("[data-actions]");
    const addApp = createButton(t("button.addApp"), "button primary");
    const addFolder = createButton(t("button.addFolder"));
    addApp.addEventListener("click", () => void addTarget("application"));
    const addSite = createButton(t("button.addWebsite"));
    addFolder.addEventListener("click", () => void addTarget("folder"));
    addSite.addEventListener("click", () => openPicker());
    const addCommandButton = createButton(t("button.addCommand"));
    addCommandButton.addEventListener("click", () => openPicker(undefined, "command"));
    const addSeriesButton = createButton(t("button.addSeries"));
    addSeriesButton.addEventListener("click", () => openPicker(undefined, "series"));
    actions?.append(addApp, addFolder, addSite, addCommandButton, addSeriesButton);

    shell.querySelector<HTMLElement>("[data-keyboard]")?.append(keyboardPreview());
    const items = shell.querySelector<HTMLElement>("[data-items]");
    if (items) paintItems(items);
    root.append(shell);

    const keyboard = root.querySelector<HTMLElement>("[data-keyboard]");
    if (keyboard) keyboard.scrollLeft = keyboardScroll;
    root.scrollTop = pageScroll;
  };

  paint();
  // コマンドがすぐに失敗したときは、Rust側が設定画面を開いてから知らせてくる。
  await listen<string>("launch-failed", ({ payload }) => {
    showStatus(translateError(payload), "error", 15000);
  });
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
