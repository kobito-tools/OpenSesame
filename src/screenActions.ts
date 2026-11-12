import { t } from "./i18n";
import type {
  DisplayInfo,
  DisplayModeRef,
  DisplaySetting,
  DisplayStep,
  LauncherItem,
  ModeOption,
  ScreenRef,
  WallpaperFit,
} from "./types";

/** 巡回に入れられる解像度の数。Rust 側の `MAX_DISPLAY_MODES` とそろえる。 */
export const MAX_DISPLAY_MODES = 8;

/** 1段階上げ下げの向き。一覧の先頭にこの順で出す。 */
export const DISPLAY_STEPS: DisplayStep[] = ["up", "down"];

/** 壁紙の表示の仕方。並びは Rust 側の `WALLPAPER_FITS` とそろえる。 */
export const WALLPAPER_FITS: WallpaperFit[] = ["fill", "fit", "stretch", "center"];

export function modeLabel(mode: DisplayModeRef): string {
  const size = `${mode.width} × ${mode.height}`;
  return mode.refresh > 0 ? `${size} · ${Math.round(mode.refresh)}Hz` : size;
}

export function sameMode(a: DisplayModeRef, b: DisplayModeRef): boolean {
  return (
    a.width === b.width &&
    a.height === b.height &&
    a.pixel_width === b.pixel_width &&
    a.pixel_height === b.pixel_height &&
    Math.round(a.refresh * 100) === Math.round(b.refresh * 100)
  );
}

/** 選んだ解像度の並び。キーの表示名や一覧の説明に使う。 */
export function modesSummary(modes: DisplayModeRef[]): string {
  return modes.map(modeLabel).join(" ⇄ ");
}

/** 選んだ解像度、または1段階上げ下げの説明。 */
export function settingSummary(setting: DisplaySetting): string {
  return setting.step ? t(`display.step.${setting.step}`) : modesSummary(setting.modes);
}

/** 表示名を省略したときの名前。ポップアップのキーに出るので短くする。 */
export function defaultDisplayName(setting: DisplaySetting): string {
  if (setting.step) return t(`display.stepName.${setting.step}`);
  const [first] = setting.modes;
  if (!first) return setting.screen.name;
  if (setting.modes.length === 1) return `${first.width}×${first.height}`;
  return t("display.cycleName", { count: setting.modes.length });
}

/**
 * 覚えている画面を、いまつながっている画面から探す。Rust 側の `display::find_screen` と同じ順で、
 * メーカー・型番・製造番号 → メーカー・型番（名前の合う方）→ 名前。
 */
export function findDisplay(displays: DisplayInfo[], screen: ScreenRef): number {
  const known = screen.vendor !== 0 || screen.model !== 0;
  if (known) {
    if (screen.serial !== 0) {
      const exact = displays.findIndex(
        (d) =>
          d.screen.vendor === screen.vendor &&
          d.screen.model === screen.model &&
          d.screen.serial === screen.serial,
      );
      if (exact !== -1) return exact;
    }
    const sameModel = displays
      .map((d, index) => ({ d, index }))
      .filter(({ d }) => d.screen.vendor === screen.vendor && d.screen.model === screen.model);
    return (sameModel.find(({ d }) => d.screen.name === screen.name) ?? sameModel[0])?.index ?? -1;
  }
  const named = displays.filter((d) => screen.name && d.screen.name === screen.name);
  return named.length === 1 ? displays.indexOf(named[0]) : -1;
}

/** 設定画面の一覧に出す、その項目の中身の説明。 */
export function screenItemSummary(item: LauncherItem): string {
  if (item.type === "display" && item.display) {
    return `${item.display.screen.name} · ${settingSummary(item.display)}`;
  }
  if (item.type === "wallpaper") {
    const setting = item.wallpaper;
    const where = setting?.screen ? setting.screen.name : t("wallpaper.allScreens");
    const fit = t(`wallpaper.fit.${setting?.fit ?? "fill"}` as never);
    return `${item.target.value} · ${where} · ${fit}`;
  }
  return item.target.value;
}

/** 解像度の形と大きさを示す小さな図。一覧で最も大きい解像度に対する比で描く。 */
export function modeIcon(mode: DisplayModeRef, largest: number): HTMLElement {
  const visual = document.createElement("div");
  visual.className = "table-icon window-table-icon";
  const diagram = document.createElement("span");
  diagram.className = "mode-diagram";
  const scale = Math.max(0.35, Math.sqrt((mode.width * mode.height) / Math.max(largest, 1)));
  const box = 30 * scale;
  const ratio = mode.width / mode.height;
  diagram.style.width = `${ratio >= 1 ? box : box * ratio}px`;
  diagram.style.height = `${ratio >= 1 ? box / ratio : box}px`;
  visual.append(diagram);
  return visual;
}

/** 壁紙の表示の仕方を示す図。形は styles.css の `data-fit` で決める。 */
export function fitIcon(fit: WallpaperFit): HTMLElement {
  const visual = document.createElement("div");
  visual.className = "table-icon window-table-icon";
  const diagram = document.createElement("span");
  diagram.className = "fit-diagram";
  diagram.dataset.fit = fit;
  visual.append(diagram);
  return visual;
}

function screenTabLabel(display: DisplayInfo): string {
  return display.main ? `${display.screen.name}（${t("display.main")}）` : display.screen.name;
}

/** 画面を選ぶタブ。`allLabel` を渡すと先頭に「すべての画面」を置き、その位置を -1 とする。 */
export function screenTabs(
  displays: DisplayInfo[],
  current: number,
  onChange: (index: number) => void,
  allLabel?: string,
): HTMLElement {
  const tabs = document.createElement("div");
  tabs.className = "window-tabs";
  tabs.setAttribute("role", "tablist");
  const entries: Array<[number, string]> = displays.map((display, index) => [
    index,
    screenTabLabel(display),
  ]);
  if (allLabel) entries.unshift([-1, allLabel]);
  for (const [index, label] of entries) {
    const tab = document.createElement("button");
    tab.type = "button";
    tab.className = "window-tab";
    tab.setAttribute("role", "tab");
    const selected = index === current;
    tab.classList.toggle("active", selected);
    tab.setAttribute("aria-selected", String(selected));
    const text = document.createElement("span");
    text.textContent = label;
    tab.append(text);
    tab.addEventListener("click", () => {
      if (index !== current) onChange(index);
    });
    tabs.append(tab);
  }
  return tabs;
}

/**
 * 表示中の画面・検索語・詳しい一覧かどうか・1段階上げ下げを選んだか。
 * 描き直しても保てるよう呼び出し側が持つ。`step` を選ぶと、決まった解像度の選択は外す。
 */
export interface DisplayBrowserState {
  screen: number;
  query: string;
  detailed: boolean;
  step: DisplayStep | null;
}

export interface DisplayBrowserOptions {
  displays: DisplayInfo[];
  state: DisplayBrowserState;
  /** 選んだモード（選んだ順）。この関数の中で書き換える。 */
  selected: DisplayModeRef[];
  /** 画面を切り替えたときや、選び方が変わったとき。 */
  onChange: () => void;
  /** 選べる数を超えたとき。 */
  onLimit: () => void;
  autofocus?: boolean;
}

function optionWords(option: ModeOption): string {
  return [
    `${option.width}x${option.height}`,
    modeLabel(option),
    option.hidpi ? "hidpi retina" : "",
    option.current ? t("display.current") : "",
    option.native ? t("display.native") : "",
  ]
    .join(" ")
    .toLowerCase();
}

/** 1段階上げ下げを示す図。矢印の向きで上げる・下げるを分ける。 */
function stepIcon(step: DisplayStep): HTMLElement {
  const visual = document.createElement("div");
  visual.className = "table-icon window-table-icon";
  const diagram = document.createElement("span");
  diagram.className = "step-diagram";
  diagram.dataset.step = step;
  diagram.textContent = step === "up" ? "↑" : "↓";
  visual.append(diagram);
  return visual;
}

/**
 * 画面のタブ、絞り込みの検索欄、解像度の格子。ウィンドウ整形の一覧と同じ見た目にそろえる。
 * 押した順に選び、2つ以上選ぶと押すたびに順に切り替える項目になる。
 * 先頭の「1段階上げる・下げる」は、どちらか1つだけを選ぶ（決まった解像度の選択とは両立しない）。
 */
export function displayModeBrowser(options: DisplayBrowserOptions): HTMLElement {
  const { displays, state, selected } = options;
  const browser = document.createElement("div");
  browser.className = "window-browser";

  const toolbar = document.createElement("div");
  toolbar.className = "window-browser-toolbar";
  const search = document.createElement("input");
  search.type = "search";
  search.className = "window-search";
  search.placeholder = t("display.search");
  search.setAttribute("aria-label", t("display.search"));
  search.value = state.query;
  const detailToggle = document.createElement("label");
  detailToggle.className = "hide-toggle";
  detailToggle.title = t("display.detailedHint");
  const detailBox = document.createElement("input");
  detailBox.type = "checkbox";
  detailBox.checked = state.detailed;
  const detailText = document.createElement("span");
  detailText.textContent = t("display.detailed");
  detailToggle.append(detailBox, detailText);
  toolbar.append(
    screenTabs(displays, state.screen, (index) => {
      state.screen = index;
      selected.length = 0;
      options.onChange();
    }),
    search,
    detailToggle,
  );

  const description = document.createElement("p");
  description.className = "window-browser-desc";
  const grid = document.createElement("div");
  grid.className = "window-grid";
  browser.append(toolbar, description, grid);

  const paintGrid = () => {
    const display = displays[state.screen];
    const list = display ? (state.detailed ? display.detailed : display.modes) : [];
    const words = state.query.toLowerCase().split(/\s+/).filter(Boolean);
    const shown = list.filter((option) => {
      const haystack = optionWords(option);
      return words.every((word) => haystack.includes(word));
    });
    const largest = Math.max(1, ...list.map((option) => option.width * option.height));
    description.textContent = state.step
      ? t(`display.stepSelected.${state.step}`)
      : selected.length > 0
        ? t("display.selected", { modes: modesSummary(selected) })
        : t("display.lead");
    const steps = DISPLAY_STEPS.filter((step) => {
      const haystack = `${t(`display.step.${step}`)} ${step} step`.toLowerCase();
      return words.every((word) => haystack.includes(word));
    });
    grid.replaceChildren(
      ...steps.map((step) => {
        const tile = document.createElement("button");
        tile.type = "button";
        tile.className = "window-tile picker-tile";
        const chosen = state.step === step;
        tile.classList.toggle("selected", chosen);
        tile.setAttribute("aria-pressed", String(chosen));
        tile.title = t(`display.stepHint.${step}`);
        const name = document.createElement("span");
        name.className = "window-tile-name";
        name.textContent = t(`display.step.${step}`);
        tile.append(stepIcon(step), name);
        tile.addEventListener("click", () => {
          state.step = chosen ? null : step;
          if (state.step) selected.length = 0;
          paintGrid();
          options.onChange();
        });
        return tile;
      }),
      ...shown.map((option) => {
        const tile = document.createElement("button");
        tile.type = "button";
        tile.className = "window-tile picker-tile";
        const order = selected.findIndex((mode) => sameMode(mode, option));
        tile.classList.toggle("selected", order !== -1);
        tile.setAttribute("aria-pressed", String(order !== -1));
        const name = document.createElement("span");
        name.className = "window-tile-name";
        name.textContent = modeLabel(option);
        const tags = document.createElement("span");
        tags.className = "mode-tags";
        const tagTexts = [
          option.current ? t("display.current") : "",
          option.native ? t("display.native") : "",
          state.detailed && option.hidpi ? "HiDPI" : "",
        ].filter(Boolean);
        tags.textContent = tagTexts.join(" · ");
        tile.title = `${modeLabel(option)} (${option.pixel_width} × ${option.pixel_height} px)`;
        tile.append(modeIcon(option, largest), name);
        if (tagTexts.length > 0) tile.append(tags);
        if (order !== -1) {
          const badge = document.createElement("kbd");
          badge.className = "window-tile-key";
          badge.textContent = String(order + 1);
          tile.append(badge);
        }
        tile.addEventListener("click", () => {
          state.step = null;
          if (order !== -1) {
            selected.splice(order, 1);
          } else if (selected.length >= MAX_DISPLAY_MODES) {
            options.onLimit();
            return;
          } else {
            selected.push({
              width: option.width,
              height: option.height,
              pixel_width: option.pixel_width,
              pixel_height: option.pixel_height,
              refresh: option.refresh,
            });
          }
          paintGrid();
          options.onChange();
        });
        return tile;
      }),
    );
    if (shown.length === 0 && steps.length === 0) {
      const empty = document.createElement("p");
      empty.className = "window-grid-empty";
      empty.textContent = t("display.empty");
      grid.append(empty);
    }
  };

  search.addEventListener("input", () => {
    state.query = search.value;
    paintGrid();
  });
  search.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && search.value) {
      search.value = "";
      state.query = "";
      paintGrid();
    }
  });
  detailBox.addEventListener("change", () => {
    state.detailed = detailBox.checked;
    paintGrid();
  });

  paintGrid();
  if (options.autofocus) requestAnimationFrame(() => search.focus());
  return browser;
}

/** 壁紙の表示の仕方を、図付きの格子から1つ選ぶ。 */
export function wallpaperFitGrid(
  current: WallpaperFit,
  onChange: (fit: WallpaperFit) => void,
): HTMLElement {
  const grid = document.createElement("div");
  grid.className = "window-grid";
  for (const fit of WALLPAPER_FITS) {
    const tile = document.createElement("button");
    tile.type = "button";
    tile.className = "window-tile picker-tile";
    tile.classList.toggle("selected", fit === current);
    tile.setAttribute("aria-pressed", String(fit === current));
    tile.title = t(`wallpaper.fitHint.${fit}` as never);
    const name = document.createElement("span");
    name.className = "window-tile-name";
    name.textContent = t(`wallpaper.fit.${fit}` as never);
    tile.append(fitIcon(fit), name);
    tile.addEventListener("click", () => onChange(fit));
    grid.append(tile);
  }
  return grid;
}
