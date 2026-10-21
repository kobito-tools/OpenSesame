#!/bin/zsh
# アイコンを作り直す：kobito/icon.svg（元）→ kobito/icon-1024.png と、Tauri が使う src-tauri/icons/ の画像。
# Node.js が必要（npm ci のあとで動かす）。tauri icon は Android・iOS 向けの画像も作るが、使う物だけを写す。
set -euo pipefail
cd "${0:A:h}/.."
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
# 作った画像の一覧は長いので、失敗したときだけ出す。
icon() { npx tauri icon kobito/icon.svg "$@" > "$work/log" 2>&1 || { cat "$work/log" >&2; exit 1; } }
icon --output "$work/all"
icon --output "$work/large" --png 1024
cp "$work/large/1024x1024.png" kobito/icon-1024.png
# tauri.conf.json の bundle.icon に載せている物だけ。.icns と .ico は、いくつもの大きさを持つので必ず入れる。
for name in 32x32.png 128x128.png 128x128@2x.png icon.icns icon.ico; do
  cp "$work/all/$name" "src-tauri/icons/$name"
done
echo kobito/icon-1024.png src-tauri/icons/
