#!/bin/zsh
# アプリに入れる壁紙（昼・夕方・夜）を作り直す：scripts/wallpapers/*.py（元）→
#   src-tauri/wallpapers/kobito-<名前>.jpg（3840×2160。アプリに入れる）
#   public/wallpapers/kobito-<名前>.jpg（480×270。設定画面の縮小画像）
# Python 3・Google Chrome・ImageMagick（magick）が必要。
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
cd "${0:A:h}/.."
chrome="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
[[ -x $chrome ]] || { echo "Google Chrome was not found." >&2; exit 1; }
(( $+commands[magick] )) || { echo "ImageMagick (magick) was not found. Install it with brew install imagemagick." >&2; exit 1; }
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p src-tauri/wallpapers public/wallpapers
for name in day evening night; do
  python3 scripts/wallpapers/$name.py "$work/$name.svg"
  # SVG は 2560×1440。1.5 倍で描いて 3840×2160 にする（水彩・鉛筆・紙のフィルターは Chrome で描く）。
  "$chrome" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1.5 --window-size=2560,1440 \
    --screenshot="$work/$name.png" "file://$work/$name.svg" >/dev/null 2>&1
  magick "$work/$name.png" -quality 90 -sampling-factor 4:2:0 -strip "src-tauri/wallpapers/kobito-$name.jpg"
  magick "$work/$name.png" -resize 480x270 -quality 82 -strip "public/wallpapers/kobito-$name.jpg"
  echo "src-tauri/wallpapers/kobito-$name.jpg"
done
