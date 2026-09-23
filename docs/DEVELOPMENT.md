# 開発者向けドキュメント

OpenSesame! の中身の構造、ビルド方法、配布手順をまとめています。
アプリを使うだけなら [README](../README.md) を参照してください。

## 構成

| 層 | 技術 | 役割 |
|---|---|---|
| バックエンド | Rust / Tauri 2 | 設定保存、グローバルキーフック、対象の起動、ウィンドウ整形、アイコンキャッシュ、トレイ |
| フロントエンド | Vanilla TypeScript / Vite | 設定画面と、フォーカスを奪わないポップアップ |

```
.
├── index.html                 # 設定画面・ポップアップ共通のエントリ
├── src/                       # フロントエンド
│   ├── main.ts                # ウィンドウのラベルで設定画面かポップアップかを振り分け
│   ├── settings.ts            # 設定画面
│   ├── popup.ts               # ポップアップ
│   ├── keys.ts                # キーボード配列（JIS / 英字）の描画
│   ├── activation.ts          # 起動キー（修飾キー）の正規化と表示名
│   ├── i18n.ts                # 日本語 / 英語の文言
│   ├── types.ts               # 設定ファイルの型
│   └── styles.css
├── src-tauri/                 # バックエンド
│   ├── src/
│   │   ├── lib.rs             # コマンド定義、起動処理、トレイ
│   │   ├── input.rs           # キー入力の状態遷移（OS非依存）と、Windowsのフック
│   │   ├── tap_macos.rs       # macOSのキーフック（CGEventTap）
│   │   ├── keys.rs            # 物理キーの正規名と対応表
│   │   ├── window_layout.rs   # 前面ウィンドウの整形
│   │   ├── storage.rs         # 設定の読み書きと移行
│   │   ├── icons.rs           # アイコンのキャッシュ
│   │   └── model.rs           # 設定ファイルの構造
│   ├── icons/                 # アプリアイコン（source.svg が元データ）
│   ├── capabilities/          # フロントエンドに許可するTauri API
│   ├── Info.plist             # macOS用の追加項目（Apple Eventsの説明文）
│   └── tauri.conf.json
├── .github/workflows/         # CI と リリースビルド
└── docs/
```

## 開発環境

必要なもの:

- Node.js 20以降
- Rust（stable）
- [Tauri 2 の OS 別 Prerequisites](https://v2.tauri.app/start/prerequisites/)

```bash
npm install
npm run tauri dev
```

配布用のアプリを作る場合:

```bash
npm run tauri build
```

テスト:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

`npm run build` はフロントエンド（`dist/`）を生成するだけです。アプリ本体はRustバイナリが `dist/` を取り込んでコンパイルされるため、**変更を反映するには `tauri dev` か `tauri build` が必要**です。

## リリース手順

GitHub Actions（`.github/workflows/release.yml`）が macOS（ユニバーサル版 `.dmg`）と Windows（`-setup.exe` / `.msi`）のインストーラーを作ります。

1. `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` の `version` を揃えて上げる
2. コミットして `main` に push する
3. バージョンのタグを付けて push する

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. Actions の完了後、GitHub の Releases に下書きができるので、内容を確認して「Publish release」を押す

`main` への push と Pull Request では `.github/workflows/ci.yml` が両OSでテストとビルドを確認します。

### コード署名について

macOS版は `bundle.macOS.signingIdentity` に `-` を指定し、ad-hoc 署名しています。Apple Developer Program の証明書と公証（notarization）が無いため、利用者は初回起動時に Gatekeeper の確認を通す必要があります（手順は README に記載）。Windows版も未署名のため、SmartScreen の確認が出ます。

証明書を用意できた場合は、[Tauri の署名ドキュメント](https://v2.tauri.app/distribute/sign/macos/) に従って Actions のシークレットを設定すれば、この手順は不要になります。

## アイコン

元データは `src-tauri/icons/source.svg` です。PNG（1024×1024、透過）に書き出して `src-tauri/icons/source.png` を置き換え、次を実行します。

```bash
npm run tauri icon src-tauri/icons/source.png
```

`tauri icon` は Android / iOS / Windowsストア向けの画像も生成しますが、本アプリはデスクトップのみなので、`tauri.conf.json` の `bundle.icon` に載っていないファイルは削除して構いません。`bundle.icon` には `.icns` と `.ico` を必ず含めてください。PNGだけを指定すると1サイズのみの `.icns` が生成され、Finderがアイコンを描けません。

## データ保存先

設定、アイコンキャッシュ、カスタムアイコンは Tauri の `appLocalDataDir` に保存されます。

- macOS: `~/Library/Application Support/io.github.kobito-tools.opensesame/`
- Windows: `%LOCALAPPDATA%\io.github.kobito-tools.opensesame\`

設定は一時ファイルへ書き込んで同期してから置き換えます。起動時に設定ファイルが読めない（壊れている）場合は `config.json.broken-<時刻>` に退避し、初期設定で起動します。

設定ファイルは起動時にバージョン4へ移行します。

- v3: 旧形式の起動キー名（`LeftControl` / `LeftOption` など）を `ControlLeft` / `AltLeft` へ正規化
- v4: `settings_layout` と `popup.layout` を `keyboard_layout` へ統合し、`language` を追加

## セキュリティ上の設計

- ネットワーク通信は行いません。キー入力はメモリ上で判定するだけで、記録も送信もしません。
- 設定画面には CSP（`tauri.conf.json` の `app.security.csp`）を設定し、外部スクリプトの読み込みを禁止しています。
- フロントエンドに公開している Tauri API は `capabilities/default.json` の既定権限のみです。ファイルの読み取りは `assetProtocol` でアプリのデータ領域（`$APPLOCALDATA`）に限っています。
- 登録名やパスなど利用者由来の文字列は `textContent` / 属性で描画し、HTML文字列には埋め込みません。

## macOSの権限

グローバルキー監視には「アクセシビリティ」の許可が必要です。権限が無い間は設定画面に手順とボタンを表示し、付与されると数秒で自動的に監視が始まります（`input.rs` が2秒間隔で再試行）。

アクセシビリティの許可はアプリのコード署名識別子とパスに紐づきます。ad-hoc 署名により識別子は `io.github.kobito-tools.opensesame` で固定されます。未署名だとビルドのたびに識別子が変わり、許可が定着しません。ビルドし直して差し替えた際に反応しなくなった場合は、一覧から項目を削除して追加し直します。

ウィンドウ整形は `osascript` で System Events を操作するため、初回に「オートメーション」の許可ダイアログが出ます。説明文は `src-tauri/Info.plist` の `NSAppleEventsUsageDescription` です。

## macOSのキーフックについて

macOSでは `src-tauri/src/tap_macos.rs` で CGEventTap を直接張っています。rdev は使いません。

rdev の `grab` はキー押下のたびに Text Services Manager でキーの文字表記を引きますが、このAPIはメインスレッドでの呼び出しを強制します。`grab` は専用スレッドで動くため、最初のキー押下で必ず `dispatch_assert_queue` に引っかかり SIGTRAP で落ちます。文字表記は本アプリでは使いません。

自前のタップはアプリのメインランループに繋いであり、コールバックもメインスレッドで動きます。そのため AppKit に触れても安全です。OSにタップを切られた場合（`kCGEventTapDisabledByTimeout` など）はコールバック内で張り直します。修飾キーの押下と解放は、flagsChanged イベントのデバイス固有ビットで判別しています。

AppKit に触れる処理（ウィンドウ表示、ディスプレイの照会）は、`run_on_main_thread` でメインスレッドへ回してください。重い処理をタップのコールバックで直接行うと、OSにタップを切られます。

Windows では rdev の `grab`（低レベルキーボードフック）を使っています。

## 半透明の作り方

透過ウィンドウでは、CSSの `backdrop-filter` はデスクトップに届きません。ぼかせるのはページ内のコンテンツだけなので、背景を持たないページではただの薄い面になり、壁紙が派手だと同化します。

そのため macOS では `window-vibrancy` で NSVisualEffectView を適用し、OS側でぼかしています（ポップアップは `HudWindow`、設定画面は `WindowBackground`）。ポップアップの角丸は `POPUP_CORNER_RADIUS` と CSS の `#app` で揃えてください。

マテリアルはシステムの外観モードに追従するため、ダークモードでは暗い面になります。本アプリはライトテーマなので、ぼかしの上に明るい面を一枚重ねて可読性を確保しています。この重ね方を外すと、ダークモードで文字が読めなくなります。

## 対応範囲

- アプリ / フォルダの登録
- 前面ウィンドウの10種類のレイアウト整形
- 物理キー A–Z、0–9、記号、Enter / Space / Tab / Backspace / Esc、矢印キーへの割り当て
- 起動キー（修飾キー1〜3個）のキー入力による編集
- JIS / 英字キーボード配列の表示
- 日本語 / 英語の表示切り替え
- システムアイコン（Finder / エクスプローラー表示のもの）の登録時キャッシュ
- カスタムPNGのアプリデータ領域へのコピー
- キーリピートによる多重起動防止
- `.` の設定画面用予約
- 登録キーの元アプリへの入力抑止
- トレイ（メニューバー）からの設定表示と終了
