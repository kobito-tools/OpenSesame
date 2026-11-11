#!/bin/zsh
# OpenSesame! を、この Mac で使える状態にする（macOS だけ）。ターミナルで ./setup.sh として動かす。
# 決まりは docs/kobito-spec/SETUP.md。共通の手順は docs/kobito-spec/setup.zsh にある。
# 何度動かしても安全。登録した項目・設定には触れず、書くのは起動情報だけ。sudo は使わない。

source "${0:A:h}/docs/kobito-spec/setup.zsh" || exit 1

APP_NAME="OpenSesame!"
EXEC_NAME="OpenSesame"
RUSTUP_URL="https://sh.rustup.rs"

kobito_steps=(
  "Check that no repository with the same app ID is recorded in another location (if one is, stop without changing anything)"
  "Check that the Xcode Command Line Tools and Node.js 20 or later are installed (if not, show how to install them and stop)"
  "Check that Rust is installed (if not, ask whether to install it with the official rustup method)"
  "Build $APP_NAME.app (takes a few minutes)"
  "Ask whether to move it to the kobito-tools folder in the Applications folder (--yes does not move it)"
  "Write the launch info using the .app where it was placed"
  "Record this repository's location in repos/ in the per-Mac folder (a symbolic link)"
  "Show how to grant the Accessibility permission"
)
kobito_setup "$APP_NAME" "$@"
# ビルド・起動情報より前に確かめる（あとで止まると、起動情報だけがこのリポジトリに変わって repos/ と食い違う）。
kobito_check_repository
# PATH に入っていなくても、Homebrew の既定の場所に入れた Node.js が見つかるようにする（build.sh と同じ）。
path=($path /opt/homebrew/bin /usr/local/bin)

# 足りない道具は、--dry-run なら知らせて続ける（何をするかを最後まで見せるため）。
missing() {
  (( kobito_dry_run )) && { kobito_info "(--dry-run) $1. A real run would stop here."; return 0; }
  kobito_fail "$1"
}

kobito_step "Checking the required tools"
if xcode-select -p >/dev/null 2>&1; then
  kobito_info "Xcode Command Line Tools: found"
else
  missing "The Xcode Command Line Tools were not found. Run xcode-select --install in Terminal to install them, then run this again"
fi
if (( $+commands[node] && $+commands[npm] )) && (( $(node -p 'process.versions.node.split(".")[0]' 2>/dev/null || print 0) >= 20 )); then
  kobito_info "Node.js: found ($(node -v))"
else
  missing "Node.js 20 or later was not found. Install it from https://nodejs.org/ or with brew install node (brew upgrade node if it is too old), then run this again"
fi

kobito_step "Checking Rust"
if (( $+commands[cargo] )); then
  kobito_info "Rust: found ($(cargo --version))"
elif (( kobito_dry_run )); then
  kobito_info "Rust: not found. A real run would ask whether to install it with the official method (curl --proto '=https' --tlsv1.2 -sSf $RUSTUP_URL | sh -s -- -y)."
elif kobito_ask "Rust is not installed. Install it with the official method (rustup)?" y; then
  curl --proto '=https' --tlsv1.2 -sSf "$RUSTUP_URL" | sh -s -- -y || kobito_fail "Could not install Rust. Install it by following https://rustup.rs/, then run this again."
  rehash
  (( $+commands[cargo] )) || kobito_fail "Rust was installed but cargo cannot be found. Open a new terminal window and run this again."
  kobito_info "Installed Rust ($(cargo --version))"
else
  kobito_fail "Rust is needed to build the app. Install it by following https://rustup.rs/, then run this again."
fi

# ソースが新しくなっていることがあるので、毎回作り直す。
kobito_step "Building $APP_NAME.app (takes a few minutes)"
built="$kobito_here/$APP_NAME.app"
kobito_run zsh ./build.sh || kobito_fail "Could not build the app. Check the output above."

# 起動情報は .app の場所を含むので、移すかどうかを決めてから書き出す。
# 置き先はアプリケーションフォルダの kobito-tools フォルダ。直下に残っている古い .app には触らない。
# 試すときは KOBITO_APPLICATIONS_DIR で別のフォルダを指せる（実際のアプリケーションフォルダに触れないため）。
kobito_step "Deciding whether to move $APP_NAME.app to the Applications folder"
placed="$built"
if kobito_ask "Move $APP_NAME.app to the kobito-tools folder in the Applications folder?" n; then
  kobito_applications_folder
  target="$kobito_applications/$APP_NAME.app"
  move=1
  if [[ -e $target ]]; then
    kobito_info "$target already exists."
    if pgrep -f "^$target/Contents/MacOS/" >/dev/null 2>&1; then
      kobito_info "$target is running, so it is not replaced. Quit it, then run ./setup.sh again."
      move=0
    elif ! kobito_ask "Replace it? (The existing app is deleted. Your registered items and settings are kept.)" n; then
      kobito_info "Not replacing it. The existing app is left as it is."
      move=0
    fi
  fi
  if (( move )); then
    if { [[ ! -e $target ]] || kobito_run rm -rf "$target"; } && kobito_run mv "$built" "$target"; then
      placed="$target"
      (( kobito_dry_run )) || kobito_info "Moved: $placed"
      old="${KOBITO_APPLICATIONS_DIR:-/Applications}/$APP_NAME.app"
      [[ -e $old ]] && kobito_info "The copy in the old location, $old, is left as it is. If you no longer use it, move it to the Trash (you can also remove it from the Accessibility list with \"-\")."
    else
      kobito_info "Could not move it. $built is left in this folder."
    fi
  fi
else
  kobito_info "Not moving it. Using $built."
fi

# --write-launch-info は、書いてすぐ終わる（画面もキーの監視も始めず、動いている方も終わらせない）。
kobito_step "Writing the launch info"
kobito_run "$placed/Contents/MacOS/$EXEC_NAME" --write-launch-info || kobito_fail "Could not write the launch info. It is written when you start the app once."

kobito_link_repository

kobito_step "About permissions"
kobito_info "$APP_NAME needs the Accessibility permission to detect key presses (it shows a guide the first time you start it)."
kobito_info "  1. Click \"Open Accessibility settings\" in the guide"
kobito_info "  2. Add $APP_NAME with \"+\" below the list and turn its switch on"
kobito_info "Each build changes the signature, so after rebuilding, remove $APP_NAME from the list with \"-\" and add it again."
[[ $placed == $built ]] && kobito_info "Moving the app to the Applications folder before starting it makes the permission more likely to stick."

kobito_finish "open \"$placed\""
