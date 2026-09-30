use crate::{
    keys::{key_matches, RESERVED_KEY},
    model::{LauncherConfig, LauncherItem},
    window_layout,
};
use std::{
    collections::HashSet,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex, RwLock,
    },
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};

/// 起動キーの正規名。設定保存のたびに差し替える。
pub type ActivationKeys = Arc<RwLock<HashSet<String>>>;
/// キーボード監視の起動に失敗した理由。設定画面から後追いで読めるように保持する。
pub type InputError = Arc<Mutex<Option<String>>>;

/// 権限が付与されるまで待つ間隔。付与後は再起動なしで監視が始まる。
const RETRY_INTERVAL: Duration = Duration::from_secs(2);

/// ポップアップはキーボード配列のみを出すので、サイズは固定でよい。
/// キーボード(16ユニット×5行)とパネルの余白にぴったり合わせる。
/// 余白を残すとバイブランシーの面だけが広がって不格好になる。
const POPUP_WIDTH: f64 = 1100.0;
const POPUP_HEIGHT: f64 = 364.0;

/// 入力をそのまま流すか、元のアプリへ渡さず飲み込むか。
pub enum Verdict {
    Pass,
    Swallow,
}

/// ポップアップ操作はフックのコールバック内で行わない。
/// コールバックが遅いとOSにタップを切られるため、専用スレッドへ送る。
enum PopupCommand {
    Show,
    Hide,
}

/// プラットフォーム非依存の状態遷移。キーは設定ファイル上の正規名で扱う。
pub struct Machine {
    app: AppHandle,
    config: Arc<RwLock<LauncherConfig>>,
    activation: ActivationKeys,
    popup: Sender<PopupCommand>,
    pressed: HashSet<String>,
    consumed: HashSet<String>,
    armed: bool,
}

impl Machine {
    pub fn on_press(&mut self, key: &str) -> Verdict {
        let first_press = self.pressed.insert(key.to_string());
        let activation = match self.activation.read() {
            Ok(keys) => keys.clone(),
            Err(_) => return Verdict::Pass,
        };

        if !self.armed && activation_is_down(&self.pressed, &activation) {
            self.armed = true;
            let _ = self.popup.send(PopupCommand::Show);
        }

        if !self.armed || activation.contains(key) {
            return Verdict::Pass;
        }

        if key == RESERVED_KEY {
            self.consumed.insert(key.to_string());
            if first_press {
                let app = self.app.clone();
                let _ = self.popup.send(PopupCommand::Hide);
                thread::spawn(move || show_settings(&app));
            }
            return Verdict::Swallow;
        }

        if let Some(action) = find_window_action(&self.config, key) {
            self.consumed.insert(key.to_string());
            if first_press {
                let app = self.app.clone();
                thread::spawn(move || {
                    let _ = window_layout::apply(&app, &action);
                });
            }
            return Verdict::Swallow;
        }

        if let Some(item) = find_item(&self.config, key) {
            self.consumed.insert(key.to_string());
            if first_press {
                launch(item);
            }
            return Verdict::Swallow;
        }

        Verdict::Pass
    }

    pub fn on_release(&mut self, key: &str) -> Verdict {
        let swallow = self.consumed.remove(key);
        self.pressed.remove(key);
        let activation = match self.activation.read() {
            Ok(keys) => keys.clone(),
            Err(_) => return Verdict::Pass,
        };
        if self.armed && !activation_is_down(&self.pressed, &activation) {
            self.armed = false;
            let _ = self.popup.send(PopupCommand::Hide);
        }
        if swallow {
            Verdict::Swallow
        } else {
            Verdict::Pass
        }
    }
}

pub fn start(
    app: AppHandle,
    config: Arc<RwLock<LauncherConfig>>,
    activation: ActivationKeys,
    input_error: InputError,
) {
    let popup = start_popup_worker(app.clone());
    let machine = Arc::new(Mutex::new(Machine {
        app: app.clone(),
        config,
        activation,
        popup,
        pressed: HashSet::new(),
        consumed: HashSet::new(),
        armed: false,
    }));

    thread::spawn(move || {
        // 権限は設定画面から後追いで付与されるため、成功するまで待ち続ける。
        loop {
            if !accessibility_granted() {
                report(&app, &input_error, Some(error("accessibility-denied", "")));
                thread::sleep(RETRY_INTERVAL);
                continue;
            }
            match install_hook(&app, machine.clone()) {
                Ok(hook) => {
                    report(&app, &input_error, None);
                    watch_hook(&app, &input_error, hook);
                    return;
                }
                Err(detail) => {
                    report(&app, &input_error, Some(error("event-tap-failed", &detail)));
                    thread::sleep(RETRY_INTERVAL);
                }
            }
        }
    });
}

/// 権限の取り消しなどでOSにタップを切られたら、権限が戻るのを待って張り直す。
/// コールバック内で即座に張り直すと切断と再接続が延々と繰り返されるため、ここで間隔を空ける。
#[cfg(target_os = "macos")]
fn watch_hook(app: &AppHandle, input_error: &InputError, tap: crate::tap_macos::Tap) {
    loop {
        thread::sleep(RETRY_INTERVAL);
        if !accessibility_granted() {
            report(app, input_error, Some(error("accessibility-denied", "")));
            continue;
        }
        if !tap.is_enabled() {
            tap.enable();
        }
        report(app, input_error, None);
    }
}

/// rdevのフックは自前で復帰するので見張る必要はない。
#[cfg(not(target_os = "macos"))]
fn watch_hook(_app: &AppHandle, _input_error: &InputError, _hook: ()) {}

/// macOSのフックはメインスレッドのランループに繋ぐ必要がある。
#[cfg(target_os = "macos")]
fn install_hook(
    app: &AppHandle,
    machine: Arc<Mutex<Machine>>,
) -> Result<crate::tap_macos::Tap, String> {
    let (sender, receiver) = mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = sender.send(crate::tap_macos::install(machine));
    })
    .map_err(|error| error.to_string())?;
    receiver
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "main thread did not respond".to_string())?
}

/// macOS以外はrdevのフックをそのまま使う。
#[cfg(not(target_os = "macos"))]
fn install_hook(_app: &AppHandle, machine: Arc<Mutex<Machine>>) -> Result<(), String> {
    use rdev::{grab, Event, EventType};
    let handle = thread::spawn(move || {
        grab(move |event: Event| -> Option<Event> {
            let Ok(mut machine) = machine.lock() else {
                return Some(event);
            };
            let verdict = match event.event_type {
                EventType::KeyPress(key) => crate::keys::key_name(key).map(|name| machine.on_press(name)),
                EventType::KeyRelease(key) => {
                    crate::keys::key_name(key).map(|name| machine.on_release(name))
                }
                _ => None,
            };
            match verdict {
                Some(Verdict::Swallow) => None,
                _ => Some(event),
            }
        })
    });
    // grabは成功すると戻らないので、短時間で戻ってきたら失敗とみなす。
    thread::sleep(Duration::from_millis(400));
    if handle.is_finished() {
        return Err("hook exited immediately".into());
    }
    Ok(())
}

fn start_popup_worker(app: AppHandle) -> Sender<PopupCommand> {
    let (sender, receiver) = mpsc::channel::<PopupCommand>();
    thread::spawn(move || {
        while let Ok(command) = receiver.recv() {
            // ウィンドウやモニタの照会はAppKitに触れるのでメインスレッドで行う。
            let handle = app.clone();
            let _ = app.run_on_main_thread(move || match command {
                PopupCommand::Show => show_popup(&handle),
                PopupCommand::Hide => hide_popup(&handle),
            });
        }
    });
    sender
}

/// 表示言語を切り替えられるよう、UIに出る失敗はコードで返す。
fn error(code: &str, detail: &str) -> String {
    if detail.is_empty() {
        format!("applauncher-error:{code}")
    } else {
        format!("applauncher-error:{code}:{detail}")
    }
}

/// 現在の監視状態を保存しつつ、設定画面へ通知する。
fn report(app: &AppHandle, input_error: &InputError, message: Option<String>) {
    let changed = match input_error.lock() {
        Ok(mut slot) => {
            let changed = *slot != message;
            slot.clone_from(&message);
            changed
        }
        Err(_) => true,
    };
    if !changed {
        return;
    }
    let _ = app.emit("input-monitor-status", message.clone());
    if let Some(message) = message {
        let _ = app.emit("input-monitor-error", message);
    }
}

/// macOSのアクセシビリティ許可。Windowsでは権限の概念がないので常に真。
#[cfg(target_os = "macos")]
fn accessibility_granted() -> bool {
    // CoreFoundationの Boolean は unsigned char なので、Rust側も u8 で受ける。
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> u8;
    }
    unsafe { AXIsProcessTrusted() != 0 }
}

#[cfg(not(target_os = "macos"))]
fn accessibility_granted() -> bool {
    true
}

fn activation_is_down(pressed: &HashSet<String>, activation: &HashSet<String>) -> bool {
    !activation.is_empty() && activation.iter().all(|key| pressed.contains(key))
}

fn find_item(config: &Arc<RwLock<LauncherConfig>>, pressed_name: &str) -> Option<LauncherItem> {
    config
        .read()
        .ok()?
        .items
        .iter()
        .find(|item| key_matches(&item.key, pressed_name))
        .cloned()
}

fn find_window_action(config: &Arc<RwLock<LauncherConfig>>, pressed_name: &str) -> Option<String> {
    config
        .read()
        .ok()?
        .window_actions
        .iter()
        .find(|binding| {
            binding
                .key
                .as_deref()
                .is_some_and(|key| key_matches(key, pressed_name))
        })
        .map(|binding| binding.action.clone())
}

fn launch(item: LauncherItem) {
    thread::spawn(move || {
        let _ = open::that(item.target.value);
    });
}

fn show_popup(app: &AppHandle) {
    let Some(window) = app.get_webview_window("popup") else {
        return;
    };
    let _ = window.set_size(tauri::LogicalSize::new(POPUP_WIDTH, POPUP_HEIGHT));
    if let Ok(cursor) = app.cursor_position() {
        if let Ok(Some(monitor)) = app.monitor_from_point(cursor.x, cursor.y) {
            let scale = monitor.scale_factor();
            let monitor_position = monitor.position();
            let monitor_size = monitor.size();
            let x =
                monitor_position.x + ((monitor_size.width as f64 - POPUP_WIDTH * scale) / 2.0) as i32;
            let y = monitor_position.y
                + ((monitor_size.height as f64 - POPUP_HEIGHT * scale) / 2.0) as i32;
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        }
    }
    // フルスクリーンのSpaceへ移るたびに効くよう、表示の直前に毎回設定し直す。
    #[cfg(target_os = "macos")]
    crate::float_popup_over_fullscreen(&window);
    let _ = window.show();
    // 表示のたびにキーを浮き上がらせるので、画面側へ再生の合図を送る。
    let _ = app.emit_to("popup", "popup-shown", ());
}

fn hide_popup(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.hide();
    }
}

fn show_settings(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        hide_popup(&handle);
        crate::show_settings_window(&handle);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn activation_requires_every_key() {
        let activation = keys(&["ControlLeft", "AltLeft"]);
        let mut pressed = keys(&["ControlLeft"]);
        assert!(!activation_is_down(&pressed, &activation));
        pressed.insert("AltLeft".into());
        assert!(activation_is_down(&pressed, &activation));
    }

    #[test]
    fn empty_activation_never_arms() {
        let pressed = keys(&["ControlLeft", "AltLeft"]);
        assert!(!activation_is_down(&pressed, &HashSet::new()));
    }
}
