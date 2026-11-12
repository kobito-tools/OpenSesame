#!/bin/zsh
# シリーズの紹介画像（1280×640。作りはシリーズ共通）を、英語版と日本語版の2枚で書き出す。
#   assets/social-preview.png    英語版（GitHub の Social preview に使う）
#   assets/social-preview.ja.png 日本語版
# Google Chrome と Noto Sans JP のフォントファイルが必要。
# 使い方: scripts/social-preview.sh <NotoSansJP.ttf のパス>
set -euo pipefail
cd "${0:A:h}/.."
[[ $# -ge 1 ]] || { echo "使い方: scripts/social-preview.sh <NotoSansJP.ttf のパス>" >&2; exit 1; }
font="${1:A}"
[[ -f "$font" ]] || { echo "フォントが見つかりません: $font" >&2; exit 1; }
chrome="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p assets

# $1: 書き出し先  $2: html の lang  $3: 説明（大）  $4: 説明（小）
render() {
  cat > "$work/og.html" <<HTML
<html lang="$2"><head><meta charset="utf-8"><style>
@font-face{font-family:Noto;src:url(file://$font)}
html,body{margin:0;width:1280px;height:640px;overflow:hidden}
body{background:linear-gradient(180deg,#9A7234,#785623);font-family:Noto,sans-serif;color:#FBF6EA;position:relative}
img{position:absolute;left:80px;top:100px;width:440px;height:440px}
.text{position:absolute;left:560px;right:64px;top:0;bottom:0;display:flex;flex-direction:column;justify-content:center}
h1{margin:0;font-weight:700;font-size:84px;letter-spacing:-.01em;line-height:1}
.lead{margin-top:30px;font-size:30px;white-space:nowrap;font-weight:500;line-height:1.5}
.sub{margin-top:10px;font-size:24px;opacity:.8;white-space:nowrap}
.org{position:absolute;right:64px;bottom:48px;font-size:22px;letter-spacing:.06em;opacity:.7}
</style></head><body><img src="file://${PWD}/kobito/icon.svg"><div class="text"><h1>OpenSesame!</h1><div class="lead">$3</div><div class="sub">$4</div></div><div class="org">kobito-tools</div></body></html>
HTML
  "$chrome" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 --window-size=1280,640 \
    --screenshot="$work/og.png" "file://$work/og.html" >/dev/null 2>&1
  cp "$work/og.png" "$1"
  echo "$1"
}

render assets/social-preview.png en \
  "A keyboard-shaped launcher<br>that appears while you hold a key." \
  "Apps, folders, websites and window layouts, one key each."
render assets/social-preview.ja.png ja \
  "キーを押している間だけ現れる、<br>キーボード型のランチャー" \
  "アプリもフォルダも窓の配置も、キー1つで。"
