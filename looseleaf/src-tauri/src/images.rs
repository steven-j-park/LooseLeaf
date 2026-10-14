//! Image import: decode with EXIF orientation applied and cache a preview and a thumbnail.

use image::{DynamicImage, ImageDecoder, ImageReader, imageops::FilterType};
use serde::Serialize;
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use tauri::Manager;

const PREVIEW_PX: u32 = 1600;
const THUMB_PX: u32 = 200;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    /// Dimensions after EXIF orientation.
    pub width: u32,
    pub height: u32,
    pub preview_path: String,
    pub thumb_path: String,
}

/// Decode an image and apply its EXIF orientation. Used for import and for export.
pub fn load_oriented(path: &Path) -> Result<DynamicImage, String> {
    let mut decoder = ImageReader::open(path)
        .map_err(|e| format!("could not open {}: {e}", path.display()))?
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_decoder()
        .map_err(|e| format!("could not decode {}: {e}", path.display()))?;
    let orientation = decoder.orientation().map_err(|e| e.to_string())?;
    let mut img = DynamicImage::from_decoder(decoder)
        .map_err(|e| format!("could not decode {}: {e}", path.display()))?;
    img.apply_orientation(orientation);
    Ok(img)
}

/// Cache key from path + size + mtime so edited files regenerate their previews.
fn cache_key(path: &Path) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    meta.len().hash(&mut h);
    mtime.hash(&mut h);
    Ok(format!("{:016x}", h.finish()))
}

fn write_jpeg(img: &DynamicImage, max: u32, out: &Path) -> Result<(), String> {
    let small = if img.width() > max || img.height() > max {
        img.resize(max, max, FilterType::Triangle)
    } else {
        img.clone()
    };
    DynamicImage::ImageRgb8(small.to_rgb8())
        .save_with_format(out, image::ImageFormat::Jpeg)
        .map_err(|e| format!("could not write {}: {e}", out.display()))
}

fn prepare(path: &Path, cache: &Path) -> Result<ImageInfo, String> {
    fs::create_dir_all(cache).map_err(|e| e.to_string())?;
    let key = cache_key(path)?;
    let preview: PathBuf = cache.join(format!("{key}_preview.jpg"));
    let thumb: PathBuf = cache.join(format!("{key}_thumb.jpg"));
    let dims = cache.join(format!("{key}.dims"));

    // Cache hit: both files and the saved dimensions exist.
    if preview.exists() && thumb.exists() {
        if let Some((w, h)) = fs::read_to_string(&dims).ok().and_then(|s| {
            let (w, h) = s.trim().split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        }) {
            return Ok(info(w, h, &preview, &thumb));
        }
    }

    let img = load_oriented(path)?;
    write_jpeg(&img, PREVIEW_PX, &preview)?;
    write_jpeg(&img, THUMB_PX, &thumb)?;
    fs::write(&dims, format!("{}x{}", img.width(), img.height())).map_err(|e| e.to_string())?;
    Ok(info(img.width(), img.height(), &preview, &thumb))
}

fn info(width: u32, height: u32, preview: &Path, thumb: &Path) -> ImageInfo {
    ImageInfo {
        width,
        height,
        preview_path: preview.to_string_lossy().into_owned(),
        thumb_path: thumb.to_string_lossy().into_owned(),
    }
}

#[tauri::command]
pub async fn prepare_image(app: tauri::AppHandle, path: String) -> Result<ImageInfo, String> {
    let cache = app.path().app_cache_dir().map_err(|e| e.to_string())?.join("previews");
    tauri::async_runtime::spawn_blocking(move || prepare(Path::new(&path), &cache))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_preview_and_thumbnail_and_reuses_cache() {
        let dir = std::env::temp_dir().join("looseleaf_img_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let src = dir.join("big.png");
        image::RgbImage::from_pixel(3000, 2000, image::Rgb([200, 30, 30])).save(&src).unwrap();

        let a = prepare(&src, &dir.join("cache")).unwrap();
        assert_eq!((a.width, a.height), (3000, 2000));
        let (pw, ph) = image::image_dimensions(&a.preview_path).unwrap();
        assert_eq!((pw, ph), (1600, 1067));
        let (tw, _) = image::image_dimensions(&a.thumb_path).unwrap();
        assert_eq!(tw, 200);

        let b = prepare(&src, &dir.join("cache")).unwrap();
        assert_eq!((b.width, b.height), (3000, 2000));
        let _ = fs::remove_dir_all(&dir);
    }
}
