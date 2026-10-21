#!/bin/zsh
# Finder からダブルクリックで、OpenSesame! の初期設定（setup.sh）を動かす入口。
cd "${0:A:h}" || exit 1
zsh ./setup.sh "$@"
result=$?
print
read -rs "?Enter キーを押すと閉じます。"
print
exit $result
