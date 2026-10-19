#!/bin/zsh
# OpenSesame! を、この PC で使える状態にする（macOS）。
# 決まり（引数・終了コード・する順番）は kobito-workspace の docs/SETUP.md に従う。
set -uo pipefail
cd "${0:A:h}"

APP_NAME="OpenSesame!"
APP_DIR="/Applications"
RUSTUP_URL="https://sh.rustup.rs"

usage() {
  cat <<EOF
使い方: ./setup.sh [--yes] [--dry-run] [--help]

$APP_NAME をソースからビルドして、この PC で使える状態にします（macOS だけ）。

  --yes      尋ねる場面では、すべて既定の答えを選ぶ
             （Rust が無ければ入れる。アプリケーションフォルダへは移さない）
  --dry-run  何をするかを表示するだけで、何も変えない
  --help     この使い方を表示する

終了コード: 0 成功 / 1 失敗 / 2 引数の誤り
EOF
}

yes=0
dry_run=0
for arg in "$@"; do
  case "$arg" in
    --yes) yes=1 ;;
    --dry-run) dry_run=1 ;;
    --help|-h) usage; exit 0 ;;
    *) echo "知らない引数です: $arg" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ "$(uname -s)" != Darwin ]]; then
  echo "このスクリプトは macOS だけに対応しています。" >&2
  exit 1
fi

# Finder から開いたときにも、rustup・Homebrew で入れた道具が見つかるようにする（build.sh と同じ）。
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

step() { echo ""; echo "▶ $*"; }
info() { echo "  $*"; }
would() { echo "  （dry-run）$*"; }

# 尋ねる。$1 が問い、$2 が既定の答え（yes / no）。はいなら 0 を返す。
ask() {
  local question=$1 default=$2 reply hint
  if (( yes )); then
    info "$question → $([[ $default == yes ]] && echo はい || echo いいえ)（--yes の既定）"
    [[ $default == yes ]]
    return
  fi
  if [[ ! -t 0 ]]; then
    info "$question → いいえ（入力を受け取れないため）"
    return 1
  fi
  [[ $default == yes ]] && hint="[Y/n]" || hint="[y/N]"
  read "reply?  $question $hint "
  [[ -z $reply ]] && reply=$default
  [[ $reply == [yY]* ]]
}

# 1. 必要な道具を確かめる
step "1. 必要な道具を確かめます"
stop=0
if xcode-select -p >/dev/null 2>&1; then
  info "Xcode Command Line Tools … あり"
else
  info "Xcode Command Line Tools … ありません"
  info "  → ターミナルで xcode-select --install を実行して入れてください"
  stop=1
fi
if command -v node >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then
  node_major=$(node -p 'process.versions.node.split(".")[0]' 2>/dev/null || echo 0)
  if (( node_major >= 20 )); then
    info "Node.js … あり（$(node -v)）"
  else
    info "Node.js … 古いです（$(node -v)）。20 以降が必要です"
    info "  → https://nodejs.org/ から入れ直すか、brew upgrade node"
    stop=1
  fi
else
  info "Node.js … ありません"
  info "  → https://nodejs.org/ から入れるか、brew install node"
  stop=1
fi
has_rust=0
if command -v cargo >/dev/null 2>&1; then
  has_rust=1
  info "Rust … あり（$(cargo --version)）"
else
  info "Rust … ありません（次の手順で、入れるかを尋ねます）"
fi
if (( stop )); then
  if (( dry_run )); then
    would "足りない道具があるため、実際に動かすとここで止まります"
  else
    echo "" >&2
    echo "足りない道具を入れてから、もう一度 ./setup.sh を実行してください。" >&2
    exit 1
  fi
fi

# 2. Rust を公式の手順（rustup）で入れる
step "2. Rust を確かめます"
if (( has_rust )); then
  info "入っているので飛ばします"
elif ask "Rust を公式の手順（rustup）で入れますか？" yes; then
  if (( dry_run )); then
    would "curl --proto '=https' --tlsv1.2 -sSf $RUSTUP_URL | sh -s -- -y を実行します"
  else
    info "rustup で Rust を入れます…"
    if ! curl --proto '=https' --tlsv1.2 -sSf "$RUSTUP_URL" | sh -s -- -y; then
      echo "Rust を入れられませんでした。https://rustup.rs/ の手順で入れてから、もう一度実行してください。" >&2
      exit 1
    fi
    if ! command -v cargo >/dev/null 2>&1; then
      echo "Rust を入れましたが、cargo が見つかりません。ターミナルを開き直してから、もう一度実行してください。" >&2
      exit 1
    fi
    info "Rust を入れました（$(cargo --version)）"
  fi
else
  if (( dry_run )); then
    would "Rust が無いため、実際に動かすとここで止まります"
  else
    echo "" >&2
    echo "ビルドには Rust が必要です。https://rustup.rs/ の手順で入れてから、もう一度実行してください。" >&2
    exit 1
  fi
fi

# 3. ビルドする
step "3. $APP_NAME.app をビルドします（数分かかります）"
if (( dry_run )); then
  would "./build.sh を実行して、このフォルダに $APP_NAME.app を作ります"
else
  if ! ./build.sh; then
    echo "" >&2
    echo "ビルドに失敗しました。上の表示を確かめてください。" >&2
    exit 1
  fi
  info "$PWD/$APP_NAME.app を作りました"
fi

# 4. 起動情報を書き出す
step "4. 起動情報を書き出します"
info "飛ばします。$APP_NAME は、起動情報の書き出しにまだ対応していません。"

# 5. 任意の設定：アプリケーションフォルダへ移す
step "5. 任意の設定"
app_path="$PWD/$APP_NAME.app"
target="$APP_DIR/$APP_NAME.app"
if ask "$APP_NAME.app をアプリケーションフォルダへ移しますか？" no; then
  move=1
  if [[ -e $target ]]; then
    if pgrep -f "^$target/Contents/MacOS/" >/dev/null 2>&1; then
      info "$target が動いています。置き換えずにおきます（終了してから、もう一度実行してください）"
      move=0
    elif ! ask "$target がすでにあります。置き換えてよいですか？" no; then
      info "置き換えずにおきます"
      move=0
    fi
  fi
  if (( move )); then
    if (( dry_run )); then
      [[ -e $target ]] && would "$target を消します"
      would "$app_path を $APP_DIR へ移します"
    else
      [[ -e $target ]] && rm -rf "$target"
      mv "$app_path" "$target"
      app_path=$target
      info "$target へ移しました"
    fi
  fi
else
  info "移さずにおきます"
fi

# 6. 起動のしかた
step "6. 起動のしかた"
info "$app_path をダブルクリックして起動します。"
[[ $app_path != $APP_DIR/* ]] && info "アプリケーションフォルダへ移してから起動すると、次の許可が保たれやすくなります。"
echo ""
info "キー入力を検知するため、「アクセシビリティ」の許可が必要です（初めて起動すると案内が出ます）。"
info "  1. 案内の「アクセシビリティ設定を開く」をクリックします"
info "  2. 一覧の下の「＋」から $APP_NAME を追加し、スイッチをオンにします"
info "作り直したときは、作るたびに署名が変わるため、一覧から $APP_NAME を「－」で消して、追加し直してください。"
echo ""
(( dry_run )) && echo "dry-run のため、何も変えていません。" || echo "初期設定が終わりました。"
exit 0
