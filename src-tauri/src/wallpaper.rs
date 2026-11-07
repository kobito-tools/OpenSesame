//! 壁紙を決めた画像に変える。macOS は `NSWorkspace.setDesktopImageURL(_:for:options:)` を使う。
//! macOS はこの方法で、その画面でいま表示している Space の壁紙だけを変える。

use crate::model::WallpaperSetting;
use std::path::Path;
use tauri::AppHandle;

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

/// 壁紙を変える。バックグラウンドのスレッドから呼ぶこと。
pub fn apply(app: &AppHandle, path: &str, setting: &WallpaperSetting) -> Result<(), String> {
    check_image(path)?;
    let (scaling, clipping) = fit_options(&setting.fit)
        .ok_or_else(|| crate::error("invalid-wallpaper-fit", &setting.fit))?;
    platform::apply(app, path, setting, scaling, clipping)
}

#[cfg(target_os = "macos")]
mod platform {
    use crate::display::{find_screen, platform as display};
    use crate::model::WallpaperSetting;
    use objc2::{rc::Retained, runtime::AnyObject, MainThreadMarker};
    use objc2_app_kit::{
        NSScreen, NSWorkspace, NSWorkspaceDesktopImageAllowClippingKey,
        NSWorkspaceDesktopImageScalingKey,
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
    ) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel();
        let path = path.to_string();
        let setting = setting.clone();
        app.run_on_main_thread(move || {
            let result = match MainThreadMarker::new() {
                Some(mtm) => apply_on_main_thread(mtm, &path, &setting, scaling, clipping),
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
        let values: [&AnyObject; 2] = [&scaling, &clipping];
        let options = unsafe {
            NSDictionary::from_slices(
                &[
                    NSWorkspaceDesktopImageScalingKey,
                    NSWorkspaceDesktopImageAllowClippingKey,
                ],
                &values,
            )
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
