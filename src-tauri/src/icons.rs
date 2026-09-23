use file_icon_provider::get_file_icon;
use image::{DynamicImage, ImageFormat, RgbaImage};
use std::{fs, path::Path};

/// Finder（Windowsはエクスプローラー）が出すアイコンをそのまま取り込む。
/// Retinaでも粗く見えないよう、表示サイズより大きめに取得しておく。
const SYSTEM_ICON_SIZE: u16 = 256;

pub fn cache_system_icon(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("アイコンキャッシュを作成できません: {error}"))?;
    }

    let icon = get_file_icon(source.to_path_buf(), SYSTEM_ICON_SIZE)
        .map_err(|error| format!("システムアイコンを取得できません: {error}"))?;
    let rgba = RgbaImage::from_raw(icon.width, icon.height, icon.pixels)
        .ok_or_else(|| "システムアイコンの画像データが不正です".to_string())?;
    DynamicImage::ImageRgba8(rgba)
        .save_with_format(destination, ImageFormat::Png)
        .map_err(|error| format!("アイコンを保存できません: {error}"))
}

pub fn copy_custom_icon(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("カスタムアイコン用フォルダを作成できません: {error}"))?;
    }

    let image = image::open(source).map_err(|error| format!("PNGを読み込めません: {error}"))?;
    image
        .thumbnail(512, 512)
        .save_with_format(destination, ImageFormat::Png)
        .map_err(|error| format!("カスタムアイコンを保存できません: {error}"))
}
