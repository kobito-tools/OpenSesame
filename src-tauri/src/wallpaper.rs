//! 壁紙を決めた画像に変える。macOS は `NSWorkspace.setDesktopImageURL(_:for:options:)` を使う。
//! macOS はこの方法で、その画面でいま表示している Space の壁紙だけを変える。

use crate::model::{TargetPath, WallpaperSetting, BUILTIN_WALLPAPERS};
use std::path::{Path, PathBuf};
use tauri::{path::BaseDirectory, AppHandle, Manager};

/// 表示の仕方ごとの、NSImageScaling の値と、画面からはみ出すのを許すか。
/// システム設定の「画面いっぱいに表示」「画面サイズに合わせる」「画面全体に拡大」「中央に配置」に当たる。
pub fn fit_options(fit: &str) -> Option<(usize, bool)> {
    const PROPORTIONALLY_UP_OR_DOWN: usize = 3;
    const AXES_INDEPENDENTLY: usize = 1;
    const NONE: usize = 2;
    match fit {
        "fill" => Some((PROPORTIONALLY_UP_OR_DOWN, true)),
        "fit" => Some((PROPORTIONALLY_UP_OR_DOWN, false)),
        "stretch" => Some((AXES_INDEPENDENTLY, true)),
        "center" => Some((NONE, true)),
        _ => None,
    }
}

/// 画像があるかを先に確かめる。無ければ翻訳できるエラーにする。
pub fn check_image(path: &str) -> Result<(), String> {
    if Path::new(path).is_file() {
        Ok(())
    } else {
        Err(crate::error("image-not-found", path))
    }
}

/// アプリに入っている壁紙の名前か。
pub fn is_builtin(name: &str) -> bool {
    BUILTIN_WALLPAPERS.contains(&name)
}

/// 余白の色 `#RRGGBB` を 0〜1 の RGB にする。形が違えば None。
pub fn parse_color(color: &str) -> Option<(f64, f64, f64)> {
    let hex = color.strip_prefix('#')?;
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok().map(|v| f64::from(v) / 255.0);
    Some((channel(0)?, channel(2)?, channel(4)?))
}

/// 写し先のファイル名。中身が変わると名前も変わるよう大きさを含める
/// （macOS は同じパスの壁紙を描き直さないことがあるため）。
fn builtin_copy_name(name: &str, len: u64) -> String {
    format!("{name}-{len}.jpg")
}

/// アプリに入っている壁紙を、アプリのデータフォルダへ写してそのパスを返す。
/// アプリの中を直接指すと、アプリを置き直したり作り直したりしたときに壁紙が消えるため。
fn builtin_path(app: &AppHandle, name: &str) -> Result<PathBuf, String> {
    if !is_builtin(name) {
        return Err(crate::error("image-not-found", name));
    }
    let failed = |detail: String| crate::error("wallpaper-failed", &detail);
    let source = app
        .path()
        .resolve(format!("wallpapers/{name}.jpg"), BaseDirectory::Resource)
        .map_err(|e| failed(e.to_string()))?;
    let len = std::fs::metadata(&source)
        .map_err(|_| crate::error("image-not-found", &source.to_string_lossy()))?
        .len();
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| failed(e.to_string()))?
        .join("wallpapers");
    let target = dir.join(builtin_copy_name(name, len));
    if !target.is_file() {
        std::fs::create_dir_all(&dir).map_err(|e| failed(e.to_string()))?;
        // 途中で止まっても壊れたファイルを残さないよう、別名で書いてから置き換える。
        let partial = dir.join(format!(".{}.partial", builtin_copy_name(name, len)));
        std::fs::copy(&source, &partial).map_err(|e| failed(e.to_string()))?;
        std::fs::rename(&partial, &target).map_err(|e| failed(e.to_string()))?;
    }
    Ok(target)
}

/// 壁紙を変える。バックグラウンドのスレッドから呼ぶこと。
pub fn apply(app: &AppHandle, target: &TargetPath, setting: &WallpaperSetting) -> Result<(), String> {
    let path = if target.kind == "builtin" {
        builtin_path(app, &target.value)?.to_string_lossy().into_owned()
    } else {
        check_image(&target.value)?;
        target.value.clone()
    };
    let (scaling, clipping) = fit_options(&setting.fit)
        .ok_or_else(|| crate::error("invalid-wallpaper-fit", &setting.fit))?;
    let fill = match setting.fill_color.as_deref() {
        Some(color) => Some(parse_color(color).ok_or_else(|| crate::error("invalid-fill-color", color))?),
        None => None,
    };
    platform::apply(app, &path, setting, scaling, clipping, fill)
}

#[cfg(target_os = "macos")]
mod platform {
    use crate::display::{find_screen, platform as display};
    use crate::model::WallpaperSetting;
    use objc2::{rc::Retained, runtime::AnyObject, MainThreadMarker};
    use objc2_app_kit::{
        NSColor, NSScreen, NSWorkspace, NSWorkspaceDesktopImageAllowClippingKey,
        NSWorkspaceDesktopImageFillColorKey, NSWorkspaceDesktopImageScalingKey,
    };
    use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};
    use std::{sync::mpsc, time::Duration};
    use tauri::AppHandle;

    pub fn apply(
        app: &AppHandle,
        path: &str,
        setting: &WallpaperSetting,
        scaling: usize,
        clipping: bool,
        fill: Option<(f64, f64, f64)>,
    ) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel();
        let path = path.to_string();
        let setting = setting.clone();
        app.run_on_main_thread(move || {
            let result = match MainThreadMarker::new() {
                Some(mtm) => apply_on_main_thread(mtm, &path, &setting, scaling, clipping, fill),
                None => Err(crate::error("wallpaper-failed", "")),
            };
            let _ = sender.send(result);
        })
        .map_err(|failure| crate::error("wallpaper-failed", &failure.to_string()))?;
        receiver
            .recv_timeout(Duration::from_secs(10))
            .map_err(|_| crate::error("wallpaper-failed", ""))?
    }

    fn apply_on_main_thread(
        mtm: MainThreadMarker,
        path: &str,
        setting: &WallpaperSetting,
        scaling: usize,
        clipping: bool,
        fill: Option<(f64, f64, f64)>,
    ) -> Result<(), String> {
        let screens: Vec<Retained<NSScreen>> = NSScreen::screens(mtm).iter().collect();
        let targets: Vec<&NSScreen> = match &setting.screen {
            None => screens.iter().map(|screen| &**screen).collect(),
            Some(wanted) => {
                let names = display::screen_names_on_main_thread(mtm);
                let known = display::screens_with(&names);
                let found = find_screen(wanted, &known)
                    .ok_or_else(|| crate::error("display-not-found", &wanted.name))?;
                let screen = screens
                    .iter()
                    .find(|screen| display::screen_number(screen) == Some(found.id))
                    .ok_or_else(|| crate::error("display-not-found", &wanted.name))?;
                vec![&**screen]
            }
        };
        let url = NSURL::fileURLWithPath(&NSString::from_str(path));
        let scaling = NSNumber::numberWithUnsignedInteger(scaling);
        let clipping = NSNumber::numberWithBool(clipping);
        // 余白の色は、指定したときだけ渡す（渡さなければ macOS が決める）。
        let color = fill
            .map(|(red, green, blue)| NSColor::colorWithSRGBRed_green_blue_alpha(red, green, blue, 1.0));
        let options = unsafe {
            match &color {
                Some(color) => {
                    let values: [&AnyObject; 3] = [&scaling, &clipping, color];
                    NSDictionary::from_slices(
                        &[
                            NSWorkspaceDesktopImageScalingKey,
                            NSWorkspaceDesktopImageAllowClippingKey,
                            NSWorkspaceDesktopImageFillColorKey,
                        ],
                        &values,
                    )
                }
                None => {
                    let values: [&AnyObject; 2] = [&scaling, &clipping];
                    NSDictionary::from_slices(
                        &[
                            NSWorkspaceDesktopImageScalingKey,
                            NSWorkspaceDesktopImageAllowClippingKey,
                        ],
                        &values,
                    )
                }
            }
        };
        let workspace = NSWorkspace::sharedWorkspace();
        for screen in targets {
            unsafe { workspace.setDesktopImageURL_forScreen_options_error(&url, screen, &options) }
                .map_err(|failure| {
                    crate::error(
                        "wallpaper-failed",
                        &failure.localizedDescription().to_string(),
                    )
                })?;
        }
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use crate::model::WallpaperSetting;
    use tauri::AppHandle;

    pub fn apply(
        _app: &AppHandle,
        _path: &str,
        _setting: &WallpaperSetting,
        _scaling: usize,
        _clipping: bool,
        _fill: Option<(f64, f64, f64)>,
    ) -> Result<(), String> {
        Err(crate::error("unsupported-feature", ""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_fit_has_options() {
        for fit in crate::model::WALLPAPER_FITS {
            assert!(fit_options(fit).is_some(), "{fit}");
        }
        assert_eq!(fit_options("fill"), Some((3, true)));
        assert_eq!(fit_options("fit"), Some((3, false)));
        assert_eq!(fit_options("tile"), None);
    }

    #[test]
    fn fill_colors_are_hex_rgb() {
        assert_eq!(parse_color("#FF8000"), Some((1.0, 128.0 / 255.0, 0.0)));
        assert_eq!(parse_color("#ffffff"), Some((1.0, 1.0, 1.0)));
        for bad in ["FF8000", "#FF80", "#FF800000", "#GG8000", "", "#ＦＦ8000"] {
            assert_eq!(parse_color(bad), None, "{bad}");
        }
    }

    #[test]
    fn builtin_names_are_fixed_and_copies_change_with_the_image() {
        assert!(is_builtin("kobito-day") && is_builtin("kobito-evening") && is_builtin("kobito-night"));
        assert!(!is_builtin("../kobito-day") && !is_builtin("kobito-day.jpg"));
        assert_ne!(builtin_copy_name("kobito-day", 100), builtin_copy_name("kobito-day", 101));
    }

    #[test]
    fn every_builtin_image_is_in_the_bundle_folder() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("wallpapers");
        for name in BUILTIN_WALLPAPERS {
            assert!(dir.join(format!("{name}.jpg")).is_file(), "{name}");
        }
    }

    #[test]
    fn a_missing_image_is_reported_with_its_path() {
        assert_eq!(
            check_image("/nonexistent/opensesame.png").unwrap_err(),
            "applauncher-error:image-not-found:/nonexistent/opensesame.png"
        );
        let file =
            std::env::temp_dir().join(format!("opensesame-wall-{}.png", uuid::Uuid::new_v4()));
        std::fs::write(&file, b"x").unwrap();
        assert!(check_image(&file.to_string_lossy()).is_ok());
        assert!(check_image(&std::env::temp_dir().to_string_lossy()).is_err());
        std::fs::remove_file(file).unwrap();
    }
}
