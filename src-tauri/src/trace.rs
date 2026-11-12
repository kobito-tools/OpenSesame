//! 反応の速さを測る記録。`OPENSESAME_TRACE=1` を付けて起動したときだけ、標準エラーへ出す。
//! 例: `OPENSESAME_TRACE=1 "./OpenSesame!.app/Contents/MacOS/OpenSesame"`

use std::{sync::OnceLock, time::Instant};

fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("OPENSESAME_TRACE").is_some_and(|value| value != "0"))
}

/// `started` からの経過時間を、何をしたかと一緒に出す。
pub fn log(what: &str, started: Instant) {
    if enabled() {
        eprintln!("[trace] {what}: {:.1} ms", started.elapsed().as_secs_f64() * 1000.0);
    }
}

/// 時間ではなく、できごと（失敗の理由など）を出す。
pub fn note(what: &str) {
    if enabled() {
        eprintln!("[trace] {what}");
    }
}
