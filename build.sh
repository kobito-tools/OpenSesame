#!/bin/zsh
# OpenSesame!.app をこのフォルダ直下に作る（macOS）。Xcode Command Line Tools・Node.js・Rust が必要。
# 手元で作ったアプリには、ダウンロードしたときに付く隔離の印が無いので、Gatekeeper の警告が出ない。
set -euo pipefail
cd "${0:A:h}"

APP_NAME="OpenSesame!"
BUNDLE="src-tauri/target/release/bundle/macos/$APP_NAME.app"
STAGES=4
zmodload zsh/datetime

# Finder から開いたときにも、rustup・Homebrew で入れた道具が見つかるようにする。
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

missing=()
xcode-select -p >/dev/null 2>&1 || missing+=("Xcode Command Line Tools: run xcode-select --install in Terminal")
command -v npm >/dev/null 2>&1 || missing+=("Node.js 20 or later: install it from https://nodejs.org/ or with brew install node")
command -v cargo >/dev/null 2>&1 || missing+=("Rust: install rustup by following https://rustup.rs/")
if (( ${#missing[@]} )); then
  print -ru2 -- "The tools needed for the build were not found. Install the following, then run this again."
  for item in "${missing[@]}"; do print -ru2 -- "  - $item"; done
  exit 1
fi

# ---- 進み具合の表示 ----
# setup.sh の見出しの下に並ぶよう、2 文字下げて出す。ターミナルに出す文言は英語。
stage() { print -r -- "  [$1/$STAGES] $2" }

# 経過を出す行。ターミナルなら、終わるまで同じ行で経過秒数を書き換える。
ticker_pid=""
stop_ticker() {
  [[ -n $ticker_pid ]] || return 0
  kill "$ticker_pid" 2>/dev/null || true
  wait "$ticker_pid" 2>/dev/null || true
  ticker_pid=""
  print -n -- $'\r\e[K'
}
# 途中で止めたときも、経過の行と一時ファイルを片付ける。
build_log=""
cleanup() { stop_ticker; [[ -z $build_log ]] || rm -f "$build_log" }
trap cleanup EXIT
trap 'cleanup; exit 130' INT TERM

# run_long <番号> <見出し> <コマンド…>：時間のかかるコマンドを動かし、終わったら所要秒数を出す。
# コマンドは前で動かし（Ctrl-C がそのまま届く）、経過の表示だけを裏で回す。
# npm と cargo の出力はとても長いので一時ファイルに受け、うまくいったら捨てる。
# 失敗したときは、終わりの部分を見せ、全部は残したファイルで読めるようにする。
run_long() {
  local number=$1 label=$2 started=$EPOCHSECONDS rc=0
  shift 2
  build_log=$(mktemp "${${TMPDIR:-/tmp}%/}/opensesame-build.XXXXXX")
  local log=$build_log
  if [[ -t 1 ]]; then
    (
      local frames='|/-\' i=0
      while true; do
        print -rn -- $'\r'"  [$number/$STAGES] $label ${frames[i % 4 + 1]} $(( EPOCHSECONDS - started ))s "
        i=$(( i + 1 ))
        sleep 0.25
      done
    ) &
    ticker_pid=$!
  else
    stage "$number" "$label ..."
  fi
  "$@" >"$log" 2>&1 || rc=$?
  stop_ticker
  if (( rc )); then
    print -ru2 -- "  [$number/$STAGES] $label: failed (exit code $rc)"
    tail -n 60 "$log" | sed 's/^/      /' >&2
    print -ru2 -- "      The full output is in $log"
    # 失敗したときの記録は、読めるように消さずに残す。
    build_log=""
    return $rc
  fi
  stage "$number" "$label: done in $(( EPOCHSECONDS - started ))s"
  rm -f "$log"
  build_log=""
  return 0
}

run_long 1 "Installing the JavaScript packages" npm ci
run_long 2 "Building the settings and panel screens" npm run build
# 手元で使うだけなので、.app だけを作る（dmg は作らない）。署名は tauri.conf.json の「-」（その場の署名）。
# 画面は 2 で作ったので、tauri build の前処理（beforeBuildCommand）は止める。
# 初めてのときは Rust の部品を全部コンパイルするので数分かかる。2 回目からは速い。
run_long 3 "Compiling the app with Rust (the first build takes a few minutes)" \
  npm run tauri -- build --bundles app --config '{"build":{"beforeBuildCommand":null}}'

stage 4 "Placing the app in this folder"
rm -rf "$APP_NAME.app"
mv "$BUNDLE" "$APP_NAME.app"
print -r -- "  Built: $PWD/$APP_NAME.app"
