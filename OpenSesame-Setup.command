#!/bin/zsh
# Finder からダブルクリックで setup.sh を動かす入口。
cd "${0:A:h}"
./setup.sh
status=$?
echo ""
read "reply?Enterキーで閉じます..."
exit $status
