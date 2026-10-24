# 開発者向けドキュメント

OpenSesame! の中身の構造、ビルド方法、配布手順をまとめています。
アプリを使うだけなら [README](../README.md) を参照してください。

## 構成

| 層 | 技術 | 役割 |
|---|---|---|
| バックエンド | Rust / Tauri 2 | 設定保存、グローバルキーフック、対象の起動、コマンドの実行、kobito シリーズの起動情報、ウィンドウ整形、アイコンキャッシュ、トレイ |
| フロントエンド | Vanilla TypeScript / Vite | 設定画面と、フォーカスを奪わないポップアップ |

```
.
├── kobito-app.json            # kobito シリーズの目印（下の「自分の起動情報を書く」）
├── kobito/                    # アイコンの元 icon.svg と、その 1024px の書き出し icon-1024.png（シリーズの決まった場所）
├── setup.sh                   # 初期設定（macOS）。共通の手順 docs/kobito-spec/setup.zsh を読み込む
├── Setup.command              # setup.sh を Finder から動かす入口
├── build.sh                   # 手元で使う OpenSesame!.app を作る（macOS）
├── scripts/make-icons.sh      # kobito/icon.svg から、icon-1024.png と src-tauri/icons/ を作り直す
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
│   │   ├── command.rs         # 登録したコマンドの実行と、すぐ失敗したときの知らせ
│   │   ├── series.rs          # kobito シリーズの起動情報（自分の分の書き出し）
│   │   ├── instance.rs        # 二重起動の防止（新しい版を残し、旧版を終了させる）
│   │   ├── tap_macos.rs       # macOSのキーフック（CGEventTap）
│   │   ├── keys.rs            # 物理キーの正規名と対応表
│   │   ├── window_layout.rs   # 前面ウィンドウの整形
│   │   ├── storage.rs         # 設定の読み書きと移行
│   │   ├── icons.rs           # アイコンのキャッシュ
│   │   └── model.rs           # 設定ファイルの構造
│   ├── icons/                 # Tauri が使うアプリアイコン（scripts/make-icons.sh が kobito/icon.svg から作る）
│   ├── capabilities/          # フロントエンドに許可するTauri API
│   ├── Info.plist             # macOS用の追加項目（Apple Eventsの説明文）
│   └── tauri.conf.json
├── .github/workflows/         # CI と リリースビルド
└── docs/                      # この文書と、シリーズの規格の写し kobito-spec/（手で書き換えない）
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

手元で使う `.app` だけを作る場合（macOS）は、`./build.sh` を実行します。`npm ci` と `tauri build --bundles app` を行い、このフォルダ直下に `OpenSesame!.app` を置きます。ソースから使う人向けの入口は `setup.sh` です（道具の確かめ・ビルド・アプリケーションフォルダへ移すかの確認・起動情報の書き出し・許可と起動の案内。`--yes`・`--dry-run`・`--help` を受け付けます）。引数の読み方・表示・尋ね方は、シリーズ共通の `docs/kobito-spec/setup.zsh` を読み込んで使います。`Setup.command` は `setup.sh` を Finder からダブルクリックで動かすための入口です。試すときは `KOBITO_APPLICATIONS_DIR` に一時フォルダを指すと、実際のアプリケーションフォルダに触れません。

テスト:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npx tsc --noEmit
```

起動情報が約束どおりかは、作った `.app` で書き出して、`docs/kobito-spec/CONTRACTS.md` の「④ 起動情報」と見比べます。`KOBITO_CONFIG_DIR` を一時フォルダにすれば、本物の起動情報には触れません。

```bash
export KOBITO_CONFIG_DIR="$(mktemp -d)"
"./OpenSesame!.app/Contents/MacOS/OpenSesame" --write-launch-info
cat "$KOBITO_CONFIG_DIR/apps/opensesame.json"
```

`npm run build` はフロントエンド（`dist/`）を生成するだけです。アプリ本体はRustバイナリが `dist/` を取り込んでコンパイルされるため、**変更を反映するには `tauri dev` か `tauri build` が必要**です。

## リリース手順

GitHub Actions（`.github/workflows/release.yml`）が macOS（ユニバーサル版 `.dmg`）と Windows（`-setup.exe` / `.msi`）のインストーラーを作ります。

1. `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`、`kobito-app.json` の `version` を揃えて上げる（`cargo test` が揃っているかを確かめます）
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

元データは `kobito/icon.svg` です（シリーズの決まった場所）。書き換えたら、`npm ci` のあとで次を実行します。

```bash
./scripts/make-icons.sh
```

`tauri icon` で、`kobito/icon-1024.png`（1024×1024）と、`tauri.conf.json` の `bundle.icon` に載せている `src-tauri/icons/` の画像（`32x32.png`・`128x128.png`・`128x128@2x.png`・`icon.icns`・`icon.ico`）を作り直します。Android / iOS / Windowsストア向けの画像は写しません。`bundle.icon` には `.icns` と `.ico` を必ず含めてください。PNGだけを指定すると1サイズのみの `.icns` が生成され、Finderがアイコンを描けません。

## データ保存先

設定、アイコンキャッシュ、カスタムアイコンは Tauri の `appLocalDataDir` に保存されます。

- macOS: `~/Library/Application Support/io.github.kobito-tools.opensesame/`
- Windows: `%LOCALAPPDATA%\io.github.kobito-tools.opensesame\`

設定は一時ファイルへ書き込んで同期してから置き換えます。起動時に設定ファイルが読めない（壊れている）場合は `config.json.broken-<時刻>` に退避し、初期設定で起動します。

設定ファイルは起動時にバージョン4へ移行します。

- v3: 旧形式の起動キー名（`LeftControl` / `LeftOption` など）を `ControlLeft` / `AltLeft` へ正規化
- v4: `settings_layout` と `popup.layout` を `keyboard_layout` へ統合し、`language` を追加

項目を足しただけで、古い形のまま読めるときは版を上げません。読み込みのたびに補う方式（ウィンドウ整形の追加）か、`serde` の既定値で読みます。

- コマンドの種類（`"type": "command"`）と `target.working_dir` は、版を上げずに足しました。`working_dir` は無ければ「指定なし」として読み、指定なしの項目には書き出しません。そのため、これより前の設定ファイルはそのまま読めます（`storage.rs` のテストで確かめています）。
- 逆に、コマンドを登録した設定ファイルを、コマンドに対応していない古い版で開くと、読めずに `config.json.broken-<時刻>` へ退避されます。
- 以前の版にあった「シリーズの操作」（`"type": "kobito-action"`）は、種類から外しました。その項目が残った設定ファイルは、読み込むときにその項目だけを捨て、ほかの項目はそのまま読んで、捨てたあとの形で書き戻します（退避はしません。`storage.rs` のテストで確かめています）。

### 登録項目の形

| 種類（`type`） | `target.kind` | `target.value` | ほかの項目 |
|---|---|---|---|
| `application` | `path` | アプリのパス | `hide_when_frontmost` |
| `folder` | `path` | フォルダのパス | |
| `website` | `url` | http(s) の URL | |
| `command` | `command` | 1行のコマンド | `target.working_dir`（任意） |

## コマンドの実行

`command.rs` が受け持ちます。

| OS | 動かし方 | ねらい |
|---|---|---|
| macOS | `/bin/zsh -lc <コマンド>` | GUIのアプリは PATH が短いので、ログインシェルで `~/.zprofile` などを読ませ、ターミナルと同じコマンドが見つかるようにする |
| Windows | `cmd /C <コマンド>`（`raw_arg` でそのまま渡す、`CREATE_NO_WINDOW`） | cmd 独自の引用符の規則を崩さず、黒い窓も出さない |

- 終わるのを待たずに起動し、後始末（`wait`）は別スレッドで行います。標準入力と標準出力は捨て、エラー出力は詰まらないよう最後まで読み捨てます（先頭の300バイトだけ残す）。
- 起動できなかったとき、または3秒以内に失敗で終わったときは、`applauncher-error:command-failed:<終了コード> — <エラー出力の最後の行>` などを `launch-failed` イベントで送り、設定画面を開きます。設定画面はこれを翻訳して知らせ欄に出します。
- 作業フォルダが無いときは `missing-working-dir` で知らせます。保存のときも、新しく指定した作業フォルダだけは存在を確かめます（パスの項目と同じく、登録済みのものが後から消えても編集や削除はできます）。
- コマンドの中身は確かめません。保存のときに確かめるのは、空でないことと1行であることだけです。

## kobito シリーズの起動情報

`series.rs` が受け持ちます。形は `docs/kobito-spec/CONTRACTS.md` の「④ 起動情報」、考え方は `docs/kobito-spec/APP_INTERFACE.md` に従います。ほかのアプリの名前はコードに書きません。

置き場所は PC 固有領域の `kobito-tools/apps/<アプリID>.json` です（macOS は `~/Library/Application Support/`、Windows は `%APPDATA%\`）。環境変数 `KOBITO_CONFIG_DIR` があれば、その中の `apps/` を使います（テスト用）。

### 自分の起動情報を書く

```json
{
  "app": "opensesame",
  "name": "OpenSesame!",
  "command": ["/Applications/OpenSesame!.app/Contents/MacOS/OpenSesame"],
  "name_en": "OpenSesame!",
  "color": "gray",
  "icon": { "file": "/Applications/OpenSesame!.app/Contents/Resources/kobito/icon.svg", "glyph": "🔑" }
}
```

- 起動のたびに（`setup` の中で）書き直します。書けなくても起動は止めません。
- `command` は今動いている実行ファイル（`std::env::current_exe()`）です。`embed` は書きません（ユーティリティアプリはほかのアプリの中に表示しない）。`actions` も持ちません。
- アイコンは、シリーズの決まりどおり `kobito/icon.svg` を指します。`tauri.conf.json` の `bundle.resources` でリポジトリの `kobito/icon.svg` を `.app` の中に同じ `kobito/icon.svg` として入れ、実行ファイルの場所から求めます（macOS は `Contents/Resources/`、Windows は実行ファイルの隣）。リポジトリの画像を指さないのは、`.app` をアプリケーションフォルダへ移したあと、アプリがリポジトリの場所を知る手段が無いためです。画像が無い・256 KB を超えるときは `file` を書かず、`glyph` だけを名乗ります。
- 書くときは同じフォルダの一時ファイルに書いてから置き換えます。
- `--write-launch-info` を付けて起動すると、書いてすぐ終わります（書いたパスを標準出力に出す。失敗したら終了コード 1）。`run()` の最初、二重起動の処理（`instance::take_over_or_exit`）より前で調べるので、動いている OpenSesame! を終わらせず、画面もキーの監視も始めません。`setup.sh` は、`.app` をアプリケーションフォルダへ移すかを決めたあとに、置いた場所の `.app` でこれを動かします。
- `--metadata` を付けて起動すると、起動情報と同じ JSON を標準出力に出してすぐ終わります（何も書きません）。`--write-launch-info` と同じく、二重起動の処理より前で調べます（`series::exit_if_only_printing_metadata`）。
- リポジトリの直下の `kobito-app.json` は、シリーズの決まり（`docs/kobito-spec/APP_PACKAGE.md`）の目印です。アプリの形で配るので `"kind": "native"` と書き、`launch.py` は持ちません（登録は `setup.sh` と `--write-launch-info`）。アプリID・表示名は `series.rs`、`bundle_id` は `tauri.conf.json` の `identifier`、`version` は `tauri.conf.json`・`Cargo.toml`・`package.json` とそろえます（`series::tests::the_marker_names_this_app_as_a_native_series_app` が、`package`・`tagline` と `kobito/` のアイコンも含めて確かめます）。リポジトリの形が決まりどおりかは、kobito シリーズの確かめの道具（`docs/kobito-spec/APP_PACKAGE.md` §12）で調べられます。

## セキュリティ上の設計

- ネットワーク通信は行いません。キー入力はメモリ上で判定するだけで、記録も送信もしません。
- 設定画面には CSP（`tauri.conf.json` の `app.security.csp`）を設定し、外部スクリプトの読み込みを禁止しています。
- フロントエンドに公開している Tauri API は `capabilities/default.json` の既定権限のみです。ファイルの読み取りは `assetProtocol` でアプリのデータ領域（`$APPLOCALDATA`）に限っています。
- 登録したコマンドは、利用者が入れた文字列をそのままシェルに渡します。画面から渡せるのは保存済みの設定だけで、キーを押したときに Rust 側が設定から読んで動かします（フロントエンドから任意のコマンドを動かす入口はありません）。
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

- アプリ / フォルダ / Webサイト / コマンドの登録
- kobito シリーズの起動情報（④）の書き出し（起動のたびと `--write-launch-info`）
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
