#!/bin/zsh
# OpenSesame!.app をこのフォルダ直下に作る（macOS）。Xcode Command Line Tools・Node.js・Rust が必要。
# 手元で作ったアプリには、ダウンロードしたときに付く隔離の印が無いので、Gatekeeper の警告が出ない。
set -euo pipefail
cd "${0:A:h}"

APP_NAME="OpenSesame!"
BUNDLE="src-tauri/target/release/bundle/macos/$APP_NAME.app"

# Finder から開いたときにも、rustup・Homebrew で入れた道具が見つかるようにする。
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

missing=()
xcode-select -p >/dev/null 2>&1 || missing+=("Xcode Command Line Tools … ターミナルで xcode-select --install")
command -v npm >/dev/null 2>&1 || missing+=("Node.js 20 以降 … https://nodejs.org/ から入れるか、brew install node")
command -v cargo >/dev/null 2>&1 || missing+=("Rust … https://rustup.rs/ の手順で rustup を入れる")
if (( ${#missing[@]} )); then
  echo "ビルドに必要なものが見つかりません。次を入れてから、もう一度実行してください。" >&2
  for item in "${missing[@]}"; do echo "  - $item" >&2; done
  exit 1
fi

npm ci
# 手元で使うだけなので、.app だけを作る（dmg は作らない）。署名は tauri.conf.json の「-」（その場の署名）。
npm run tauri -- build --bundles app

rm -rf "$APP_NAME.app"
mv "$BUNDLE" "$APP_NAME.app"
echo "$PWD/$APP_NAME.app"
