//! kobito シリーズの約束（docs/kobito-spec/CONTRACTS.md）のうち、PC 固有領域に置くもの。
//!
//! ```text
//! ~/Library/Application Support/kobito-tools/   （Windows は %APPDATA%\kobito-tools\）
//!   apps/<アプリID>.json   ④ 起動情報（自分の分を起動のたびに書く）
//! ```
//!
//! テストでは実際の設定に触れないよう、環境変数 `KOBITO_CONFIG_DIR` で場所を変えられる
//! （シリーズ共通の決まり）。ほかのアプリの名前は書かない。

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

/// 起動情報に名乗るアイコンの元。リポジトリの `kobito/icon.svg` を、同じ形で `.app` の中
/// （macOS は `Contents/Resources/`、Windows は実行ファイルの隣）に入れておくので、`.app` を移しても見つかる
/// （`tauri.conf.json` の `bundle.resources`）。
pub const ICON_FILE: &str = "kobito/icon.svg";
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
        assert_eq!(marker["package"], 1);
        for key in ["tagline", "tagline_en", "description", "description_en"] {
            assert!(!marker[key].as_str().unwrap().trim().is_empty(), "{key}");
        }
        for key in ["tagline", "tagline_en"] {
            let text = marker[key].as_str().unwrap();
            assert!(text.chars().count() <= 80 && !text.contains('\n'), "{key}");
        }
        for name in ["kobito/icon.svg", "kobito/icon-1024.png", "Setup.command"] {
            assert!(root.join(name).is_file(), "{name}");
        }
        let config: Value = serde_json::from_str(
            &fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(marker["bundle_id"], config["identifier"]);
        // 版は tauri.conf.json・Cargo.toml・package.json とそろえる。
        let package: Value = serde_json::from_str(&fs::read_to_string(root.join("package.json")).unwrap()).unwrap();
        assert_eq!(marker["version"], config["version"]);
        assert_eq!(marker["version"], package["version"]);
        assert_eq!(marker["version"], env!("CARGO_PKG_VERSION"));
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
        fs::write(&icon, b"<svg/>").unwrap();
        let record = launch_record(&executable);
        assert_eq!(record["icon"]["file"], icon.to_string_lossy().as_ref());
        assert!(Path::new(record["icon"]["file"].as_str().unwrap()).is_absolute());
        // 約束の上限（256 KB）を超える画像は名乗らない。
        fs::write(&icon, vec![0u8; 256 * 1024 + 1]).unwrap();
        assert!(launch_record(&executable)["icon"].get("file").is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    /// `.app` に入れる画像が、リポジトリの `kobito/icon.svg` で、約束の形（`viewBox` が正方形の .svg、256 KB 以下）に合うこと。
    #[test]
    fn the_bundled_icon_fits_the_contract() {
        let config: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let resources = config["bundle"]["resources"].as_object().unwrap();
        let (source, _) = resources
            .iter()
            .find(|(_, target)| target.as_str() == Some(ICON_FILE))
            .expect("the icon is bundled");
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(source);
        assert_eq!(path, Path::new(env!("CARGO_MANIFEST_DIR")).join("../kobito/icon.svg"));
        assert!(fs::metadata(&path).unwrap().len() <= ICON_MAX_BYTES);
        let svg = fs::read_to_string(&path).unwrap();
        let view_box = svg.split("viewBox=\"").nth(1).and_then(|rest| rest.split('"').next()).expect("viewBox");
        let numbers: Vec<f64> = view_box.split_whitespace().map(|n| n.parse().unwrap()).collect();
        assert_eq!(numbers.len(), 4);
        assert_eq!(numbers[2], numbers[3]);
        assert!(!svg.contains("<script"));
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
}
