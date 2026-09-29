/**
 * アプリアイコンに描かれている小人。source.svg の同じ図形を切り出したもので、
 * 見た目を変えるときはアイコン側と揃えること。
 */
const KOBITO_SVG = `
<svg viewBox="6 3 41 42" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" stroke-linecap="round" stroke-linejoin="round">
  <g fill="none" stroke="#3B4441" stroke-width="1.8"><path d="M19 33 l-2 5 l-3 5"/><path d="M29 33 l2 5 l3 5"/></g>
  <path d="M10 23C10 11 16 5 25 5c8 0 13 6 13 16 0 11-6 16-15 16-8 0-13-5-13-14Z" fill="#E6E3D9" stroke="#3B4441" stroke-width="1.7"/>
  <path d="M15 13c5-4 13-4 18 0v12c-5 4-13 4-18 0Z" fill="#F6F7F2" stroke="#3B4441" stroke-width="1.4"/>
  <circle cx="21" cy="18" r="1.4" fill="#3B4441"/><circle cx="28" cy="18" r="1.4" fill="#3B4441"/>
  <path d="M22 22c2 2 4 2 6 0" fill="none" stroke="#3B4441" stroke-width="1.2"/>
  <g fill="none" stroke="#3B4441" stroke-width="1.8"><path d="M37 24 Q41 22 44.5 17"/><path d="M11.5 26 Q8.5 29 8 32.5"/></g>
</svg>`;

/** 飾りなので読み上げやクリックの対象にはしない。 */
export function kobito(className: string): HTMLElement {
  const figure = document.createElement("div");
  figure.className = `kobito ${className}`;
  figure.innerHTML = KOBITO_SVG;
  return figure;
}
