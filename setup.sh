#!/bin/zsh
# OpenSesame! を、この Mac で使える状態にする（macOS だけ。Windows はインストーラーを使う）。
# 決まりは docs/kobito-spec/SETUP.md。共通の手順は docs/kobito-spec/setup.zsh にある。
# 何度動かしても安全。登録した項目・設定には触れず、書くのは起動情報だけ。sudo は使わない。

source "${0:A:h}/docs/kobito-spec/setup.zsh" || exit 1

APP_NAME="OpenSesame!"
EXEC_NAME="OpenSesame"
RUSTUP_URL="https://sh.rustup.rs"

kobito_steps=(
  "同じアプリIDのリポジトリが、別の場所に記されていないかを確かめる（かぶっていれば何も変えずに止まる）"
  "Xcode Command Line Tools・Node.js 20 以降があるかを確かめる（無ければ入れ方を表示して止まる）"
  "Rust があるかを確かめる（無ければ、公式の手順 rustup で入れるかを尋ねる）"
  "$APP_NAME.app をビルドする（数分かかる）"
  "アプリケーションフォルダの kobito-tools フォルダへ移すかを尋ねる（--yes では移さない）"
  "起動情報を、置いた場所の .app で書き出す"
  "このリポジトリの場所を PC 固有領域の repos/ に記す（シンボリックリンク）"
  "許可（アクセシビリティ）の案内を表示する"
)
kobito_setup "$APP_NAME" "$@"
# ビルド・起動情報より前に確かめる（あとで止まると、起動情報だけがこのリポジトリに変わって repos/ と食い違う）。
kobito_check_repository
# Finder から開いたときにも、Homebrew で入れた Node.js が見つかるようにする（build.sh と同じ）。
path=($path /opt/homebrew/bin /usr/local/bin)

# 足りない道具は、--dry-run なら知らせて続ける（何をするかを最後まで見せるため）。
missing() {
  (( kobito_dry_run )) && { kobito_info "（--dry-run）$1。実際に動かすとここで止まります。"; return 0; }
  kobito_fail "$1"
}

kobito_step "必要な道具を確かめます"
if xcode-select -p >/dev/null 2>&1; then
  kobito_info "Xcode Command Line Tools: あります"
else
  missing "Xcode Command Line Tools が見つかりません。ターミナルで xcode-select --install を実行して入れてから、もう一度動かしてください"
fi
if (( $+commands[node] && $+commands[npm] )) && (( $(node -p 'process.versions.node.split(".")[0]' 2>/dev/null || print 0) >= 20 )); then
  kobito_info "Node.js: あります（$(node -v)）"
else
  missing "Node.js 20 以降が見つかりません。https://nodejs.org/ から入れるか、brew install node（古いときは brew upgrade node）で入れてから、もう一度動かしてください"
fi

kobito_step "Rust を確かめます"
if (( $+commands[cargo] )); then
  kobito_info "Rust: あります（$(cargo --version)）"
elif (( kobito_dry_run )); then
  kobito_info "Rust: ありません。実際に動かすと、公式の手順（curl --proto '=https' --tlsv1.2 -sSf $RUSTUP_URL | sh -s -- -y）で入れるかを尋ねます。"
elif kobito_ask "Rust がありません。公式の手順（rustup）で入れますか？" y; then
  curl --proto '=https' --tlsv1.2 -sSf "$RUSTUP_URL" | sh -s -- -y || kobito_fail "Rust を入れられませんでした。https://rustup.rs/ の手順で入れてから、もう一度動かしてください。"
  rehash
  (( $+commands[cargo] )) || kobito_fail "Rust を入れましたが、cargo が見つかりません。ターミナルを開き直してから、もう一度動かしてください。"
  kobito_info "Rust を入れました（$(cargo --version)）"
else
  kobito_fail "ビルドには Rust が要ります。https://rustup.rs/ の手順で入れてから、もう一度動かしてください。"
fi

# ソースが新しくなっていることがあるので、毎回作り直す。
kobito_step "$APP_NAME.app をビルドします（数分かかります）"
built="$kobito_here/$APP_NAME.app"
kobito_run zsh ./build.sh || kobito_fail "ビルドできませんでした。上の表示を確かめてください。"

# 起動情報は .app の場所を含むので、移すかどうかを決めてから書き出す。
# 置き先はアプリケーションフォルダの kobito-tools フォルダ。直下に残っている古い .app には触らない。
# 試すときは KOBITO_APPLICATIONS_DIR で別のフォルダを指せる（実際のアプリケーションフォルダに触れないため）。
kobito_step "$APP_NAME.app をアプリケーションフォルダへ移すかを決めます"
placed="$built"
if kobito_ask "$APP_NAME.app をアプリケーションフォルダの kobito-tools フォルダへ移しますか？" n; then
  kobito_applications_folder
  target="$kobito_applications/$APP_NAME.app"
  move=1
  if [[ -e $target ]]; then
    kobito_info "$target がすでにあります。"
    if pgrep -f "^$target/Contents/MacOS/" >/dev/null 2>&1; then
      kobito_info "$target が動いているので、置き換えません。終了してから、もう一度 ./setup.sh を動かしてください。"
      move=0
    elif ! kobito_ask "置き換えますか？（今あるものは消えます。登録した項目と設定は消えません）" n; then
      kobito_info "置き換えません。今あるものはそのままにします。"
      move=0
    fi
  fi
  if (( move )); then
    if { [[ ! -e $target ]] || kobito_run rm -rf "$target"; } && kobito_run mv "$built" "$target"; then
      placed="$target"
      (( kobito_dry_run )) || kobito_info "移しました: $placed"
      old="${KOBITO_APPLICATIONS_DIR:-/Applications}/$APP_NAME.app"
      [[ -e $old ]] && kobito_info "以前の置き場所の $old は、そのままにしています。使わなければ、ゴミ箱へ移してください（アクセシビリティの一覧からも「－」で消せます）。"
    else
      kobito_info "移せませんでした。$built はこのフォルダに残しています。"
    fi
  fi
else
  kobito_info "移しません。$built を使います。"
fi

# --write-launch-info は、書いてすぐ終わる（画面もキーの監視も始めず、動いている方も終わらせない）。
kobito_step "起動情報を書き出します"
kobito_run "$placed/Contents/MacOS/$EXEC_NAME" --write-launch-info || kobito_fail "起動情報を書き出せませんでした。アプリを一度起動すると、書き出されます。"

kobito_link_repository

kobito_step "許可について"
kobito_info "キー入力を検知するため、「アクセシビリティ」の許可が要ります（初めて起動すると案内が出ます）。"
kobito_info "  1. 案内の「アクセシビリティ設定を開く」をクリックします"
kobito_info "  2. 一覧の下の「＋」から $APP_NAME を追加し、スイッチをオンにします"
kobito_info "作り直したときは、作るたびに署名が変わるため、一覧から $APP_NAME を「－」で消して、追加し直してください。"
[[ $placed == $built ]] && kobito_info "アプリケーションフォルダへ移してから起動すると、許可が保たれやすくなります。"

kobito_finish "open \"$placed\""
