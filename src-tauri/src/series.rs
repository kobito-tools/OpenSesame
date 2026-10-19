//! kobito シリーズの約束（kobito-workspace の docs/CONTRACTS.md）のうち、PC 固有領域に置くもの。
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

/// 自分の起動情報。起動のコマンドは、今動いている実行ファイルそのもの。
/// `embed` は書かない（ユーティリティアプリは workspace の中に表示しない）。
pub fn launch_record(executable: &Path) -> Value {
    json!({
        "app": APP_ID,
        "name": APP_NAME,
        "command": [executable.to_string_lossy()],
    })
}

/// 自分の起動情報を書く。起動のたびに呼ぶ（`.app` を移しても追いつくように）。
pub fn write_launch_info() -> Result<PathBuf, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("実行ファイルの場所が分かりません: {error}"))?;
    let dir = apps_dir().ok_or_else(|| "起動情報の置き場所が分かりません".to_string())?;
    let path = dir.join(format!("{APP_ID}.json"));
    let mut bytes = serde_json::to_vec_pretty(&launch_record(&executable))
        .map_err(|error| format!("起動情報を変換できません: {error}"))?;
    bytes.push(b'\n');
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
    if !std::env::args().skip(1).any(|arg| arg == WRITE_LAUNCH_INFO_FLAG) {
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;

    /// 環境変数はプロセス全体で共有するので、`KOBITO_CONFIG_DIR` を変えるテストは順に動かす。
    pub(crate) static ENV_LOCK: Mutex<()> = Mutex::new(());

    pub(crate) fn with_config_dir<T>(run: impl FnOnce(&Path) -> T) -> T {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
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
        let record = launch_record(Path::new("/Applications/OpenSesame!.app/Contents/MacOS/OpenSesame"));
        assert_eq!(record["app"], "opensesame");
        assert_eq!(record["name"], "OpenSesame!");
        assert_eq!(
            record["command"],
            json!(["/Applications/OpenSesame!.app/Contents/MacOS/OpenSesame"])
        );
        assert!(record.get("embed").is_none());
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
