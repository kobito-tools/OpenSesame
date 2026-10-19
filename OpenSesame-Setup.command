#!/bin/zsh
cd "${0:A:h}"
./build.sh
status=$?
if [[ $status -eq 0 ]]; then
  echo ""
  echo "OpenSesame!.appを作成しました。アプリケーションフォルダへ移動して起動してください。"
  echo "キー入力を検知するため「アクセシビリティ」の許可が必要です（初めて起動すると案内が出ます）。"
  echo "作り直したときは、アクセシビリティの一覧から OpenSesame! を「－」で消して、追加し直してください。"
fi
echo ""
read "reply?Enterキーで閉じます..."
exit $status
