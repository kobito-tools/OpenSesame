//! kobito シリーズの約束（docs/kobito-spec/CONTRACTS.md）のうち、PC 固有領域に置くもの。
//!
//! ```text
//! ~/Library/Application Support/kobito-tools/   （Windows は %APPDATA%\kobito-tools\）
//!   apps/<アプリID>.json   ④ 起動情報（自分の分を起動のたびに書き、ほかの分は「シリーズの操作」に読む）
//! ```
//!
//! テストでは実際の設定に触れないよう、環境変数 `KOBITO_CONFIG_DIR` で場所を変えられる
//! （シリーズ共通の決まり）。ほかのアプリの名前は書かない。

use serde::Serialize;
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub const APP_ID: &str = "opensesame";
pub const APP_NAME: &str = "OpenSesame!";

/// この引数で起動したら、起動情報だけを書いて終わる（`setup.sh` から呼ぶ）。
pub const WRITE_LAUNCH_INFO_FLAG: &str = "--write-launch-info";
/// この引数で起動したら、名乗り（起動情報と同じ JSON）を標準出力に出すだけで終わる。
/// 何も書かず、何も起動しない（シリーズの決まり：docs/kobito-spec/APP_PACKAGE.md）。
pub const METADATA_FLAG: &str = "--metadata";

/// PC 固有領域の `kobito-tools` フォルダ。
pub fn config_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("KOBITO_CONFIG_DIR").filter(|dir| !dir.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support"));
    base.map(|base| base.join("kobito-tools"))
}

/// 起動情報（④）を置くフォルダ。
pub fn apps_dir() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("apps"))
}

/// 起動情報に名乗るアイコンの画像。`.app` の中（macOS は `Contents/Resources/`、
/// Windows は実行ファイルの隣）に入れておくので、`.app` を移しても見つかる。
/// 元は `src-tauri/icons/128x128@2x.png`（`tauri.conf.json` の `bundle.resources`）。
pub const ICON_FILE: &str = "kobito-icon.png";
/// 約束④の `icon.file` の大きさの上限。
const ICON_MAX_BYTES: u64 = 256 * 1024;

/// 実行ファイルから見た、`.app` の中のアイコンの画像。無い・大きすぎるときは `None`。
pub fn bundled_icon(executable: &Path) -> Option<PathBuf> {
    let dir = executable.parent()?;
    #[cfg(target_os = "macos")]
    let path = dir.parent()?.join("Resources").join(ICON_FILE);
    #[cfg(not(target_os = "macos"))]
    let path = dir.join(ICON_FILE);
    let metadata = fs::metadata(&path).ok()?;
    (metadata.is_file() && metadata.len() <= ICON_MAX_BYTES).then_some(path)
}

/// 自分の起動情報。起動のコマンドは、今動いている実行ファイルそのもの。
/// `embed` は書かない（ユーティリティアプリはほかのアプリの中に表示しない）。
/// 見せ方（英語名・色・アイコン）も名乗る。外から呼べる操作（`actions`）は持たない。
pub fn launch_record(executable: &Path) -> Value {
    let mut icon = json!({ "glyph": "🔑" });
    if let Some(file) = bundled_icon(executable) {
        icon["file"] = json!(file.to_string_lossy());
    }
    json!({
        "app": APP_ID,
        "name": APP_NAME,
        "command": [executable.to_string_lossy()],
        "name_en": APP_NAME,
        "color": "gray",
        "icon": icon,
    })
}

/// 自分の起動情報の JSON（`--write-launch-info` が書くものと、`--metadata` が出すものは同じ）。
fn launch_record_json() -> Result<Vec<u8>, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("実行ファイルの場所が分かりません: {error}"))?;
    let mut bytes = serde_json::to_vec_pretty(&launch_record(&executable))
        .map_err(|error| format!("起動情報を変換できません: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// 自分の起動情報を書く。起動のたびに呼ぶ（`.app` を移しても追いつくように）。
pub fn write_launch_info() -> Result<PathBuf, String> {
    let bytes = launch_record_json()?;
    let dir = apps_dir().ok_or_else(|| "起動情報の置き場所が分かりません".to_string())?;
    let path = dir.join(format!("{APP_ID}.json"));
    write_atomic(&path, &bytes)?;
    Ok(path)
}

/// 一時ファイルに書いてから置き換える（途中で止まっても壊れたファイルを残さない）。
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} の置き場所が分かりません", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("{} を作れません: {error}", parent.display()))?;
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temporary = parent.join(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()));
    let written = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .and_then(|mut file| file.write_all(bytes).and_then(|_| file.sync_all()));
    if let Err(error) = written {
        let _ = fs::remove_file(&temporary);
        return Err(format!("{} を書き込めません: {error}", path.display()));
    }
    #[cfg(windows)]
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&temporary, path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("{} を置き換えられません: {error}", path.display())
    })
}

/// `--write-launch-info` で起動したときは、起動情報を書いてすぐ終わる。
/// 二重起動を防ぐ処理やキーの監視より先に呼ぶこと（動いている方を終わらせないため）。
pub fn exit_if_only_writing_launch_info() {
    if !std::env::args()
        .skip(1)
        .any(|arg| arg == WRITE_LAUNCH_INFO_FLAG)
    {
        return;
    }
    match write_launch_info() {
        Ok(path) => {
            println!("{}", path.display());
            std::process::exit(0);
        }
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}

/// `--metadata` で起動したときは、名乗りを標準出力に出してすぐ終わる。何も書かない。
/// `exit_if_only_writing_launch_info` と同じく、二重起動を防ぐ処理より先に呼ぶこと。
pub fn exit_if_only_printing_metadata() {
    if !std::env::args().skip(1).any(|arg| arg == METADATA_FLAG) {
        return;
    }
    match launch_record_json() {
        Ok(bytes) => {
            let _ = std::io::stdout().write_all(&bytes);
            std::process::exit(0);
        }
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}

// ---- ほかのアプリの起動情報を読む（シリーズの操作） ----

/// 起動情報の、操作（`actions`）の1件。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SeriesAction {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_en: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// `{today}` を置き換える前の URL。画面には渡さない。
    #[serde(skip)]
    pub url: String,
}

/// 起動情報1件のうち、ランチャーが使う項目。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SeriesApp {
    pub app: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_en: Option<String>,
    pub actions: Vec<SeriesAction>,
    /// 起動コマンド。画面には渡さない。
    #[serde(skip)]
    pub command: Vec<String>,
    /// アイコンの画像（`icon.file`）。拡張子が合うものだけ。画面には渡さない。
    #[serde(skip)]
    pub icon_file: Option<PathBuf>,
}

/// アプリIDの形（`^[a-z][a-z0-9-]{0,39}$`）。
pub fn is_app_id(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some('a'..='z'))
        && text.len() <= 40
        && chars.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '-'))
}

fn text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
}

/// 自分のアプリID宛ての ② の URL で、置き換えは `{today}` だけか。
fn is_own_url(url: &str, app: &str) -> bool {
    let Some(rest) = url.strip_prefix("kobito://") else {
        return false;
    };
    if url.chars().any(char::is_whitespace) || rest.replace("{today}", "").contains(['{', '}']) {
        return false;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    authority == app
}

/// 読んだ起動情報を確かめる。必須の項目が欠けていれば `None`。
/// 形の違う操作は、その1件だけを捨てる（`id` が重なれば、後のものを捨てる）。
pub fn normalize(record: &Value) -> Option<SeriesApp> {
    let app = record.get("app")?.as_str().filter(|app| is_app_id(app))?;
    let name = record
        .get("name")?
        .as_str()
        .filter(|name| !name.is_empty())?;
    let command: Vec<String> = record
        .get("command")?
        .as_array()?
        .iter()
        .map(|part| {
            part.as_str()
                .filter(|part| !part.is_empty())
                .map(str::to_string)
        })
        .collect::<Option<_>>()?;
    if command.is_empty() {
        return None;
    }
    let mut actions: Vec<SeriesAction> = Vec::new();
    for item in record
        .get("actions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let (Some(id), Some(label), Some(url)) = (
            text(item.get("id")),
            text(item.get("label")),
            item.get("url").and_then(Value::as_str),
        ) else {
            continue;
        };
        if !is_own_url(url, app) || actions.iter().any(|action| action.id == id) {
            continue;
        }
        actions.push(SeriesAction {
            id,
            label,
            label_en: text(item.get("label_en")),
            role: text(item.get("role")),
            url: url.to_string(),
        });
    }
    let icon_file = record
        .get("icon")
        .and_then(|icon| icon.get("file"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .filter(|file| {
            let extension = file.extension().map(|ext| ext.to_ascii_lowercase());
            file.is_absolute()
                && matches!(
                    extension.as_deref().and_then(|ext| ext.to_str()),
                    Some("png" | "svg")
                )
        });
    Some(SeriesApp {
        app: app.to_string(),
        name: name.to_string(),
        name_en: text(record.get("name_en")),
        actions,
        command,
        icon_file,
    })
}

fn read_file(path: &Path) -> Option<SeriesApp> {
    let bytes = fs::read(path).ok()?;
    let record: Value = serde_json::from_slice(&bytes).ok()?;
    let app = normalize(&record)?;
    // ファイル名とアプリIDが合わないものは捨てる。
    (path.file_stem()?.to_str()? == app.app).then_some(app)
}

/// そのアプリの起動情報。無い・読めない・形が違えば `None`。
pub fn read_app(app: &str) -> Option<SeriesApp> {
    if !is_app_id(app) {
        return None;
    }
    read_file(&apps_dir()?.join(format!("{app}.json")))
}

/// この PC のすべての起動情報を、表示名の順に返す。自分の分は除く。
pub fn read_all() -> Vec<SeriesApp> {
    let Some(entries) = apps_dir().and_then(|dir| fs::read_dir(dir).ok()) else {
        return Vec::new();
    };
    let mut apps: Vec<SeriesApp> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter_map(|path| read_file(&path))
        .filter(|app| app.app != APP_ID)
        .collect();
    apps.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.app.cmp(&b.app)));
    apps
}

/// この PC の今日の日付（`YYYY-MM-DD`）。
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// 押されたときに、その時点の起動情報を読み直して、起動のコマンドと開く URL を得る。
/// 見つからなければ、画面で翻訳できるエラーを返す。
pub fn resolve_action(
    app: &str,
    action: &str,
    today: &str,
) -> Result<(Vec<String>, String), String> {
    let record = read_app(app).ok_or_else(|| crate::error("series-app-missing", app))?;
    let found = record
        .actions
        .iter()
        .find(|candidate| candidate.id == action)
        .ok_or_else(|| {
            crate::error(
                "series-action-missing",
                &format!("{} › {action}", record.name),
            )
        })?;
    Ok((record.command.clone(), found.url.replace("{today}", today)))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;

    /// 環境変数はプロセス全体で共有するので、`KOBITO_CONFIG_DIR` を変えるテストは順に動かす。
    pub(crate) static ENV_LOCK: Mutex<()> = Mutex::new(());

    pub(crate) fn with_config_dir<T>(run: impl FnOnce(&Path) -> T) -> T {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = std::env::temp_dir().join(format!("opensesame-series-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        std::env::set_var("KOBITO_CONFIG_DIR", &dir);
        let result = run(&dir);
        std::env::remove_var("KOBITO_CONFIG_DIR");
        fs::remove_dir_all(&dir).unwrap();
        result
    }

    #[test]
    fn the_config_dir_follows_the_environment_variable() {
        with_config_dir(|dir| {
            assert_eq!(config_dir().unwrap(), dir);
            assert_eq!(apps_dir().unwrap(), dir.join("apps"));
        });
    }

    #[test]
    fn the_record_has_the_required_fields_and_no_embed() {
        let record = launch_record(Path::new(
            "/Applications/OpenSesame!.app/Contents/MacOS/OpenSesame",
        ));
        assert_eq!(record["app"], "opensesame");
        assert_eq!(record["name"], "OpenSesame!");
        assert_eq!(
            record["command"],
            json!(["/Applications/OpenSesame!.app/Contents/MacOS/OpenSesame"])
        );
        assert!(record.get("embed").is_none());
    }

    /// 目印（リポジトリの直下の kobito-app.json。シリーズの決まり：APP_PACKAGE.md）の形と、名乗りとのそろい。
    #[test]
    fn the_marker_names_this_app_as_a_native_series_app() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let path = root.join("kobito-app.json");
        assert!(fs::metadata(&path).unwrap().len() <= 64 * 1024);
        let marker: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(marker["format"], "kobito-app");
        assert_eq!(marker["v"], 1);
        assert_eq!(marker["contracts"], 1);
        assert_eq!(marker["app"], APP_ID);
        assert_eq!(marker["name"], APP_NAME);
        assert_eq!(marker["name_en"], APP_NAME);
        // .app で配るので launch.py を持たない。登録は setup.sh と --write-launch-info。
        assert_eq!(marker["kind"], "native");
        assert!(marker.get("entry").is_none());
        assert_eq!(marker["setup"], "setup.sh");
        assert!(root.join("setup.sh").is_file());
        assert_eq!(marker["repository"], "kobito-tools/OpenSesame");
        assert_eq!(marker["platforms"], json!(["macos", "windows"]));
        for key in ["description", "description_en"] {
            assert!(!marker[key].as_str().unwrap().trim().is_empty(), "{key}");
        }
        let config: Value = serde_json::from_str(
            &fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(marker["bundle_id"], config["identifier"]);
    }

    #[test]
    fn names_its_look_with_the_icon_inside_the_app() {
        let root = std::env::temp_dir().join(format!("opensesame-bundle-{}", uuid::Uuid::new_v4()));
        #[cfg(target_os = "macos")]
        let (executable, icon) = (
            root.join("OpenSesame!.app/Contents/MacOS/OpenSesame"),
            root.join("OpenSesame!.app/Contents/Resources")
                .join(ICON_FILE),
        );
        #[cfg(not(target_os = "macos"))]
        let (executable, icon) = (root.join("OpenSesame.exe"), root.join(ICON_FILE));
        // 画像が無ければ、代わりの1文字だけを名乗る。
        let record = launch_record(&executable);
        assert_eq!(record["name_en"], "OpenSesame!");
        assert_eq!(record["color"], "gray");
        assert_eq!(record["icon"], json!({ "glyph": "🔑" }));
        fs::create_dir_all(icon.parent().unwrap()).unwrap();
        fs::write(&icon, b"png").unwrap();
        let record = launch_record(&executable);
        assert_eq!(record["icon"]["file"], icon.to_string_lossy().as_ref());
        assert!(Path::new(record["icon"]["file"].as_str().unwrap()).is_absolute());
        // 約束の上限（256 KB）を超える画像は名乗らない。
        fs::write(&icon, vec![0u8; 256 * 1024 + 1]).unwrap();
        assert!(launch_record(&executable)["icon"].get("file").is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    /// `.app` に入れる画像が、約束の形（正方形の .png、256 KB 以下）に合うこと。
    #[test]
    fn the_bundled_icon_fits_the_contract() {
        let config: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let resources = config["bundle"]["resources"].as_object().unwrap();
        let (source, _) = resources
            .iter()
            .find(|(_, target)| target.as_str() == Some(ICON_FILE))
            .expect("the icon is bundled");
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(source);
        assert_eq!(path.extension().unwrap(), "png");
        assert!(fs::metadata(&path).unwrap().len() <= ICON_MAX_BYTES);
        let image = image::open(&path).unwrap();
        assert_eq!(image.width(), image.height());
    }

    #[test]
    fn writes_the_launch_info_into_the_apps_folder() {
        with_config_dir(|dir| {
            let path = write_launch_info().unwrap();
            assert_eq!(path, dir.join("apps/opensesame.json"));
            let written: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert_eq!(written["app"], "opensesame");
            let executable = std::env::current_exe().unwrap();
            assert_eq!(written["command"][0], executable.to_string_lossy().as_ref());
            // 2回目は置き換わり、一時ファイルは残らない。
            write_launch_info().unwrap();
            let names: Vec<String> = fs::read_dir(dir.join("apps"))
                .unwrap()
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect();
            assert_eq!(names, ["opensesame.json"]);
        });
    }
    fn write_app(dir: &Path, file: &str, record: Value) {
        let apps = dir.join("apps");
        fs::create_dir_all(&apps).unwrap();
        fs::write(apps.join(file), serde_json::to_vec(&record).unwrap()).unwrap();
    }

    fn sample() -> Value {
        json!({
            "app": "memo",
            "name": "Memo",
            "name_en": "Memo",
            "command": ["/usr/bin/memo", "--flag"],
            "icon": { "file": "/icons/memo.png", "glyph": "✎" },
            "actions": [
                { "id": "new", "label": "新しいメモ", "label_en": "New memo", "url": "kobito://memo/new", "role": "new" },
                { "id": "today", "label": "今日の日記", "url": "kobito://memo/diary/{today}" },
                { "id": "new", "label": "重なり", "url": "kobito://memo/other" },
                { "id": "other-app", "label": "よそ宛て", "url": "kobito://elsewhere/new" },
                { "id": "file", "label": "置き換えが違う", "url": "kobito://memo/import?file={file}" },
                { "id": "broken" }
            ]
        })
    }

    #[test]
    fn keeps_only_well_formed_actions_for_the_app_itself() {
        let app = normalize(&sample()).unwrap();
        let ids: Vec<&str> = app
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect();
        assert_eq!(ids, ["new", "today"]);
        assert_eq!(app.actions[0].label_en.as_deref(), Some("New memo"));
        assert_eq!(app.actions[0].role.as_deref(), Some("new"));
        assert_eq!(app.command, ["/usr/bin/memo", "--flag"]);
        assert_eq!(app.icon_file.as_deref(), Some(Path::new("/icons/memo.png")));
    }

    #[test]
    fn records_without_the_required_fields_are_dropped() {
        for broken in [
            json!({ "name": "x", "command": ["/x"] }),
            json!({ "app": "Bad ID", "name": "x", "command": ["/x"] }),
            json!({ "app": "x", "name": "", "command": ["/x"] }),
            json!({ "app": "x", "name": "x", "command": [] }),
            json!({ "app": "x", "name": "x", "command": ["/x", ""] }),
        ] {
            assert!(normalize(&broken).is_none(), "{broken}");
        }
        // 操作を名乗らないアプリも読める（操作が無いだけ）。
        let plain = normalize(&json!({ "app": "x", "name": "X", "command": ["/x"] })).unwrap();
        assert!(plain.actions.is_empty());
    }

    #[test]
    fn resolves_an_action_from_the_launch_info_at_press_time() {
        with_config_dir(|dir| {
            write_app(dir, "memo.json", sample());
            let (command, url) = resolve_action("memo", "today", "2026-10-20").unwrap();
            assert_eq!(command, ["/usr/bin/memo", "--flag"]);
            assert_eq!(url, "kobito://memo/diary/2026-10-20");
            // 起動情報が変われば、次に押したときに新しい URL になる。
            let mut changed = sample();
            changed["actions"][0]["url"] = json!("kobito://memo/create");
            write_app(dir, "memo.json", changed);
            assert_eq!(
                resolve_action("memo", "new", "2026-10-20").unwrap().1,
                "kobito://memo/create"
            );
            assert_eq!(
                resolve_action("memo", "gone", "2026-10-20").unwrap_err(),
                "applauncher-error:series-action-missing:Memo › gone"
            );
            assert_eq!(
                resolve_action("nothing", "new", "2026-10-20").unwrap_err(),
                "applauncher-error:series-app-missing:nothing"
            );
            assert!(resolve_action("../memo", "new", "2026-10-20").is_err());
        });
    }

    #[test]
    fn lists_other_apps_by_name_and_skips_mismatched_files() {
        with_config_dir(|dir| {
            write_app(dir, "memo.json", sample());
            write_app(
                dir,
                "alpha.json",
                json!({ "app": "alpha", "name": "Alpha", "command": ["/a"] }),
            );
            write_app(
                dir,
                "renamed.json",
                json!({ "app": "beta", "name": "Beta", "command": ["/b"] }),
            );
            fs::write(dir.join("apps/broken.json"), b"{ not json").unwrap();
            write_launch_info().unwrap();
            let names: Vec<String> = read_all().into_iter().map(|app| app.name).collect();
            assert_eq!(names, ["Alpha", "Memo"]);
        });
    }

    #[test]
    fn today_is_a_plain_date() {
        let day = today();
        assert_eq!(day.len(), 10);
        assert!(day.chars().enumerate().all(|(i, c)| if i == 4 || i == 7 {
            c == '-'
        } else {
            c.is_ascii_digit()
        }));
    }
}
