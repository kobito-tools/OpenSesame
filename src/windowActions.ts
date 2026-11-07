import { t } from "./i18n";
import type { WindowActionBinding } from "./types";

/** 設定画面で整形を見せる分類。並びは Rust 側の `default_window_actions` とそろえる。 */
export const WINDOW_ACTION_GROUPS = [
  { id: "halves", actions: ["left-half", "right-half", "top-half", "bottom-half"] },
  {
    id: "quarters",
    actions: ["top-left-quarter", "top-right-quarter", "bottom-left-quarter", "bottom-right-quarter"],
  },
  {
    id: "thirds",
    actions: ["left-third", "center-third", "right-third", "top-third", "middle-third", "bottom-third"],
  },
  {
    id: "twoThirds",
    actions: [
      "left-two-thirds",
      "center-two-thirds",
      "right-two-thirds",
      "top-two-thirds",
      "middle-two-thirds",
      "bottom-two-thirds",
    ],
  },
  {
    id: "edges",
    actions: [
      "expand-left",
      "shrink-left",
      "expand-right",
      "shrink-right",
      "expand-top",
      "shrink-top",
      "expand-bottom",
      "shrink-bottom",
    ],
  },
  { id: "other", actions: ["center", "almost-maximize", "maximize", "next-display", "restore"] },
] as const;

export type WindowActionGroupId = (typeof WINDOW_ACTION_GROUPS)[number]["id"];

/** 分類の説明。補足が要る分類だけに付ける。 */
const GROUP_DESCRIPTIONS: Partial<Record<WindowActionGroupId, string>> = {
  edges: "windowGroup.edges.desc",
  other: "windowGroup.other.desc",
};

/** どの分類にも無い操作（古い版の設定に残った操作など）は「移動・その他」に入れる。 */
export function groupOf(action: string): WindowActionGroupId {
  const group = WINDOW_ACTION_GROUPS.find((entry) =>
    (entry.actions as readonly string[]).includes(action),
  );
  return group?.id ?? "other";
}

/** ウィンドウ整形の名前は設定ファイルの値ではなく、現在の言語から引く。 */
export function actionName(binding: WindowActionBinding): string {
  const key = `action.${binding.action}`;
  const translated = t(key as never);
  return translated === key ? binding.name : translated;
}

/** 分類の中で見せる短い名前。分類が分かる場所でだけ使い、無ければ通常の名前にする。 */
export function actionShortName(binding: WindowActionBinding): string {
  const key = `actionShort.${binding.action}`;
  const translated = t(key as never);
  return translated === key ? actionName(binding) : translated;
}

/** 画面のどこを占めるかを示す小さな図。形は styles.css の `data-action` で決める。 */
export function windowDiagram(action: string): HTMLElement {
  const diagram = document.createElement("span");
  diagram.className = "window-diagram";
  diagram.dataset.action = action;
  return diagram;
}

/** 表示中の分類と検索語。描き直しても保てるよう、呼び出し側が持つ。 */
export interface WindowBrowserState {
  group: WindowActionGroupId;
  query: string;
}

export interface WindowBrowserOptions {
  bindings: WindowActionBinding[];
  state: WindowBrowserState;
  /** 1つの整形の項目。`label` は分類の中なら短い名前、検索結果なら通常の名前。 */
  tile: (binding: WindowActionBinding, label: string) => HTMLElement;
  /** 分類に出す項目が無いときの文言。 */
  emptyGroupText?: string;
  /** タブに、その分類で出す項目の数を添える。 */
  showCounts?: boolean;
  autofocus?: boolean;
}

function matches(binding: WindowActionBinding, query: string): boolean {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  const haystack = [
    actionName(binding),
    actionShortName(binding),
    binding.name,
    binding.action.replace(/-/g, " "),
    t(`windowGroup.${groupOf(binding.action)}` as never),
  ]
    .join(" ")
    .toLowerCase();
  return words.every((word) => haystack.includes(word));
}

/**
 * 分類のタブ、絞り込みの検索欄、図付きの格子。検索語があるときは分類をまたいで探す。
 * 入力中に検索欄を作り直さないよう、格子だけを描き直す。
 */
export function windowActionBrowser(options: WindowBrowserOptions): HTMLElement {
  const { bindings, state } = options;
  const browser = document.createElement("div");
  browser.className = "window-browser";

  const toolbar = document.createElement("div");
  toolbar.className = "window-browser-toolbar";
  const tabs = document.createElement("div");
  tabs.className = "window-tabs";
  tabs.setAttribute("role", "tablist");
  const search = document.createElement("input");
  search.type = "search";
  search.className = "window-search";
  search.placeholder = t("windowBrowser.search");
  search.setAttribute("aria-label", t("windowBrowser.search"));
  search.value = state.query;
  toolbar.append(tabs, search);

  const description = document.createElement("p");
  description.className = "window-browser-desc";
  const grid = document.createElement("div");
  grid.className = "window-grid";
  browser.append(toolbar, description, grid);

  const counts = new Map<WindowActionGroupId, number>();
  for (const binding of bindings) {
    const group = groupOf(binding.action);
    counts.set(group, (counts.get(group) ?? 0) + 1);
  }

  const paintGrid = () => {
    const query = state.query.trim();
    tabs.classList.toggle("searching", Boolean(query));
    for (const tab of tabs.querySelectorAll<HTMLButtonElement>(".window-tab")) {
      const selected = !query && tab.dataset.group === state.group;
      tab.classList.toggle("active", selected);
      tab.setAttribute("aria-selected", String(selected));
    }
    const shown = query
      ? bindings.filter((binding) => matches(binding, query))
      : bindings.filter((binding) => groupOf(binding.action) === state.group);
    const describe = query ? undefined : GROUP_DESCRIPTIONS[state.group];
    description.textContent = describe ? t(describe as never) : "";
    description.hidden = !describe;
    grid.replaceChildren(
      ...shown.map((binding) =>
        options.tile(binding, query ? actionName(binding) : actionShortName(binding)),
      ),
    );
    if (shown.length === 0) {
      const empty = document.createElement("p");
      empty.className = "window-grid-empty";
      empty.textContent = query
        ? t("windowBrowser.empty")
        : (options.emptyGroupText ?? t("windowBrowser.empty"));
      grid.append(empty);
    }
  };

  for (const group of WINDOW_ACTION_GROUPS) {
    const tab = document.createElement("button");
    tab.type = "button";
    tab.className = "window-tab";
    tab.dataset.group = group.id;
    tab.setAttribute("role", "tab");
    const label = document.createElement("span");
    label.textContent = t(`windowGroup.${group.id}` as never);
    tab.append(label);
    const count = counts.get(group.id) ?? 0;
    if (options.showCounts) {
      const badge = document.createElement("small");
      badge.textContent = String(count);
      tab.append(badge);
    }
    tab.addEventListener("click", () => {
      state.group = group.id;
      state.query = "";
      search.value = "";
      paintGrid();
    });
    tabs.append(tab);
  }

  search.addEventListener("input", () => {
    state.query = search.value;
    paintGrid();
  });
  search.addEventListener("keydown", (event) => {
    // 検索語があるときの Escape は検索語だけを消す（設定画面側はこのときピッカーを閉じない）。
    if (event.key === "Escape" && search.value) {
      search.value = "";
      state.query = "";
      paintGrid();
    }
  });

  paintGrid();
  if (options.autofocus) requestAnimationFrame(() => search.focus());
  return browser;
}
