use std::{
    io::Read,
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// この時間内に失敗で終わったら「起動できなかった」とみなして知らせる。
/// それより後の終了は、使い終わって閉じたものとして扱う。
const EARLY_EXIT: Duration = Duration::from_secs(3);

/// 知らせに載せるエラー出力の長さ。長い出力で知らせが埋まらないようにする。
const STDERR_LIMIT: usize = 300;

/// 設定に保存する前に、前後の空白を落として1行であることを確かめる。
pub fn normalize(command: &str) -> Result<String, String> {
    let trimmed = command.trim();
    if trimmed.is_empty() || trimmed.contains(['\n', '\r']) {
        return Err(crate::error("invalid-command", ""));
    }
    Ok(trimmed.to_string())
}

/// 空の作業フォルダは「指定なし」として扱う。
pub fn normalize_working_dir(working_dir: Option<&str>) -> Option<String> {
    working_dir
        .map(str::trim)
        .filter(|dir| !dir.is_empty())
        .map(str::to_string)
}

/// GUIのアプリはターミナルと違う PATH で動くので、macOSではログインシェルを通して
/// `uv` などターミナルで使えるコマンドがそのまま見つかるようにする。
fn shell(command: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        /// コマンドプロンプトの黒い窓を出さない。
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut process = Command::new("cmd");
        // cmd は独自の引用符の規則を持つので、Rust側で引用符を付けずにそのまま渡す。
        process.arg("/C").raw_arg(command);
        process.creation_flags(CREATE_NO_WINDOW);
        process
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut process = Command::new("/bin/zsh");
        process.arg("-lc").arg(command);
        process
    }
}

fn spawn(command: &str, working_dir: Option<&str>) -> Result<Child, String> {
    let mut process = shell(command);
    match working_dir {
        Some(dir) => {
            if !Path::new(dir).is_dir() {
                return Err(crate::error("missing-working-dir", dir));
            }
            process.current_dir(dir);
        }
        None => {
            if let Some(home) = home_dir() {
                process.current_dir(home);
            }
        }
    }
    process
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|failure| crate::error("command-spawn-failed", &failure.to_string()))
}

fn home_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    let variable = "USERPROFILE";
    #[cfg(not(target_os = "windows"))]
    let variable = "HOME";
    std::env::var_os(variable).map(Into::into)
}

/// 終わるのを待たずに動かす。起動できないときや、すぐに失敗で終わったときだけ
/// `on_failure` に翻訳できるエラーを渡す。プロセスの後始末は別スレッドで行う。
pub fn launch(
    command: &str,
    working_dir: Option<&str>,
    on_failure: impl FnOnce(String) + Send + 'static,
) {
    let mut child = match spawn(command, working_dir) {
        Ok(child) => child,
        Err(message) => {
            on_failure(message);
            return;
        }
    };
    let started = Instant::now();
    // 動き続けるコマンドがエラー出力で詰まらないよう、最後まで読み捨てる。
    let (sender, receiver) = mpsc::channel::<String>();
    if let Some(mut stderr) = child.stderr.take() {
        thread::spawn(move || {
            let mut kept = Vec::new();
            let mut buffer = [0u8; 4096];
            while let Ok(read) = stderr.read(&mut buffer) {
                if read == 0 {
                    break;
                }
                if kept.len() < STDERR_LIMIT {
                    kept.extend_from_slice(&buffer[..read]);
                }
            }
            let _ = sender.send(String::from_utf8_lossy(&kept).into_owned());
        });
    }
    thread::spawn(move || {
        let Ok(status) = child.wait() else {
            return;
        };
        if status.success() || started.elapsed() > EARLY_EXIT {
            return;
        }
        // 子が出力を開いたまま裏で動いていることもあるので、長くは待たない。
        let stderr = receiver
            .recv_timeout(Duration::from_millis(500))
            .unwrap_or_default();
        let code = status
            .code()
            .map_or_else(|| "?".to_string(), |code| code.to_string());
        on_failure(crate::error("command-failed", &failure_detail(&code, &stderr)));
    });
}

/// 終了コードに、エラー出力の最後の行を添える。原因はたいていそこに出る。
fn failure_detail(code: &str, stderr: &str) -> String {
    let last_line = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .last()
        .map(|line| line.chars().take(STDERR_LIMIT).collect::<String>());
    match last_line {
        Some(line) => format!("{code} — {line}"),
        None => code.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(command: &str, working_dir: Option<&str>) -> Option<String> {
        let (sender, receiver) = mpsc::channel();
        launch(command, working_dir, move |message| {
            let _ = sender.send(message);
        });
        receiver.recv_timeout(Duration::from_secs(5)).ok()
    }

    #[test]
    fn commands_must_be_one_line() {
        assert_eq!(normalize("  uv run app.py \n").unwrap(), "uv run app.py");
        assert!(normalize("   ").is_err());
        assert!(normalize("echo a\nrm -rf /").is_err());
    }

    #[test]
    fn blank_working_dir_means_none() {
        assert_eq!(normalize_working_dir(Some("  ")), None);
        assert_eq!(normalize_working_dir(Some(" /tmp ")).as_deref(), Some("/tmp"));
    }

    #[test]
    fn a_quick_failure_is_reported_with_its_exit_code() {
        let message = run("echo broken >&2; exit 3", None).expect("should report");
        assert_eq!(message, "applauncher-error:command-failed:3 — broken");
    }

    #[test]
    fn a_success_is_not_reported() {
        assert_eq!(run("true", None), None);
    }

    #[test]
    fn a_missing_working_dir_is_reported() {
        let message = run("true", Some("/nonexistent/opensesame")).expect("should report");
        assert_eq!(
            message,
            "applauncher-error:missing-working-dir:/nonexistent/opensesame"
        );
    }

    #[test]
    fn runs_in_the_working_dir() {
        let dir = std::env::temp_dir().join(format!("opensesame-cmd-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let marker = dir.join("ran.txt");
        let dir_text = dir.to_string_lossy().into_owned();
        assert_eq!(run("pwd > ran.txt", Some(&dir_text)), None);
        let written = std::fs::read_to_string(&marker).unwrap();
        // /tmp は /private/tmp へのリンクなので、末尾の名前で比べる。
        assert!(written.trim().ends_with(&*dir.file_name().unwrap().to_string_lossy()));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
