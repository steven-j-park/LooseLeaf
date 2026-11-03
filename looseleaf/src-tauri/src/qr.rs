//! Back-cover QR code: one encoder shared by the on-screen preview (via `qr_matrix`) and the
//! export, so what's previewed is exactly what prints.

use crate::project::QrData;
use image::RgbaImage;
use qrcode::{Color, EcLevel, QrCode};

/// Side of the QR block (code + quiet zone) as a fraction of the page width. On a letter page
/// (2.75in wide) that's ~1in, comfortably scannable from a hand-held cover.
/// Keep in sync with QR_SIZE in zine.svelte.ts.
pub const QR_SIZE: f64 = 0.36;
/// Distance from the page edges for the corner positions, as a fraction of the page width.
pub const QR_INSET: f64 = 0.05;
/// White border, in modules, that scanners need around the code.
pub const QR_QUIET: usize = 4;
/// Corner radius of the white block, in modules. Kept inside the quiet zone so the rounding
/// never reaches the code. Keep in sync with QR_RADIUS in zine.svelte.ts.
pub const QR_RADIUS: f64 = 2.0;

/// Dark/light modules, row by row, without the quiet zone. Medium error correction (~15%)
/// tolerates print and paper flaws without making the code much denser.
pub fn matrix(text: &str) -> Result<Vec<Vec<bool>>, String> {
    let code = QrCode::with_error_correction_level(text.as_bytes(), EcLevel::M)
        .map_err(|e| format!("could not make a QR code: {e}"))?;
    let n = code.width();
    let colors = code.to_colors();
    Ok(colors.chunks(n).map(|row| row.iter().map(|c| *c == Color::Dark).collect()).collect())
}

#[tauri::command]
pub fn qr_matrix(text: String) -> Result<Vec<Vec<bool>>, String> {
    matrix(&text)
}

/// Top-left corner of the QR block (side `side`) on a `w` x `h` page.
fn origin(position: &str, side: f64, w: f64, h: f64) -> (f64, f64) {
    let inset = QR_INSET * w;
    let (left, right, top, bottom) = (inset, w - inset - side, inset, h - inset - side);
    match position {
        "top-left" => (left, top),
        "top-right" => (right, top),
        "bottom-left" => (left, bottom),
        "center" => ((w - side) / 2.0, (h - side) / 2.0),
        _ => (right, bottom),
    }
}

/// How much of pixel (x, y) lies inside a `side`-px square with corners rounded to `r`
/// (0 = outside, 1 = inside), anti-aliased over about one pixel.
fn rounded_coverage(x: u32, y: u32, side: f64, r: f64) -> f64 {
    let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
    let (cx, cy) = (px.clamp(r, side - r), py.clamp(r, side - r));
    let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
    (r - d + 0.5).clamp(0.0, 1.0)
}

/// Draw the QR block (rounded white quiet zone + black modules) onto a page. Modules are a
/// whole number of pixels so edges stay crisp at print resolution.
pub fn draw_qr(canvas: &mut RgbaImage, qr: &QrData, w: f64, h: f64) -> Result<(), String> {
    let text = qr.url.trim();
    if text.is_empty() {
        return Ok(());
    }
    let modules = matrix(text)?;
    let total = modules.len() + 2 * QR_QUIET;
    let px = ((QR_SIZE * w) / total as f64).floor().max(1.0) as u32;
    let side = px * total as u32;
    let (ox, oy) = origin(&qr.position, side as f64, w, h);
    let (ox, oy) = (ox.round().max(0.0) as u32, oy.round().max(0.0) as u32);

    let radius = QR_RADIUS * px as f64;
    for y in 0..side {
        for x in 0..side {
            let (cx, cy) = (ox + x, oy + y);
            if cx >= canvas.width() || cy >= canvas.height() {
                continue;
            }
            let cover = rounded_coverage(x, y, side as f64, radius);
            if cover <= 0.0 {
                continue;
            }
            let (mx, my) = ((x / px) as usize, (y / px) as usize);
            let dark = mx >= QR_QUIET
                && my >= QR_QUIET
                && modules.get(my - QR_QUIET).and_then(|r| r.get(mx - QR_QUIET)).copied().unwrap_or(false);
            let ink = if dark { 0.0 } else { 255.0 };
            let dst = canvas.get_pixel_mut(cx, cy);
            for c in 0..3 {
                dst[c] = (dst[c] as f64 * (1.0 - cover) + ink * cover).round() as u8;
            }
            dst[3] = 255;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn decode(img: &RgbaImage) -> Vec<String> {
        let luma = image::DynamicImage::ImageRgba8(img.clone()).to_luma8();
        let mut prepared = rqrr::PreparedImage::prepare(luma);
        prepared.detect_grids().iter().filter_map(|g| g.decode().ok().map(|(_, s)| s)).collect()
    }

    #[test]
    fn empty_url_draws_nothing() {
        let mut canvas = RgbaImage::from_pixel(825, 1275, Rgba([10, 20, 30, 255]));
        draw_qr(&mut canvas, &QrData { url: "  ".into(), position: "center".into() }, 825.0, 1275.0).unwrap();
        assert!(canvas.pixels().all(|p| p.0 == [10, 20, 30, 255]));
    }

    #[test]
    fn every_position_decodes_back_to_the_url() {
        let url = "https://instagram.com/looseleaf.zine";
        for position in ["top-left", "top-right", "bottom-left", "bottom-right", "center"] {
            // Mid-grey "photo" behind it, so the code must bring its own quiet zone.
            let mut canvas = RgbaImage::from_pixel(825, 1275, Rgba([120, 120, 120, 255]));
            draw_qr(&mut canvas, &QrData { url: url.into(), position: position.into() }, 825.0, 1275.0).unwrap();
            assert_eq!(decode(&canvas), vec![url.to_string()], "position {position}");
        }
    }

    #[test]
    fn positions_land_in_the_expected_region() {
        let qr = |position: &str| {
            let mut canvas = RgbaImage::from_pixel(825, 1275, Rgba([120, 120, 120, 255]));
            draw_qr(&mut canvas, &QrData { url: "https://x.co".into(), position: position.into() }, 825.0, 1275.0)
                .unwrap();
            canvas
        };
        let inset = (QR_INSET * 825.0) as u32;
        // Just inside each block's outer edge (clear of the rounded corner) is quiet-zone white.
        assert_eq!(qr("top-left").get_pixel(inset + 30, inset + 2).0, [255; 4]);
        assert_eq!(qr("top-right").get_pixel(825 - inset - 31, inset + 2).0, [255; 4]);
        assert_eq!(qr("bottom-left").get_pixel(inset + 30, 1275 - inset - 3).0, [255; 4]);
        assert_eq!(qr("bottom-right").get_pixel(825 - inset - 31, 1275 - inset - 3).0, [255; 4]);
        let c = qr("center");
        assert_eq!(c.get_pixel(2, 2).0, [120, 120, 120, 255]);
        assert_ne!(c.get_pixel(412, 637).0, [120, 120, 120, 255]);
    }

    #[test]
    fn block_corners_are_rounded() {
        let bg = [120, 120, 120, 255];
        let mut canvas = RgbaImage::from_pixel(825, 1275, Rgba(bg));
        draw_qr(&mut canvas, &QrData { url: "https://x.co".into(), position: "top-left".into() }, 825.0, 1275.0)
            .unwrap();
        let inset = (QR_INSET * 825.0).round() as u32;
        // The very corner of the block is cut away; a little in along the top edge is white.
        assert_eq!(canvas.get_pixel(inset, inset).0, bg);
        assert_eq!(canvas.get_pixel(inset + 30, inset).0, [255; 4]);
    }

    #[test]
    fn matrix_is_square() {
        let m = matrix("https://example.com/u/abc123").unwrap();
        assert!(m.len() >= 21 && m.iter().all(|r| r.len() == m.len()));
    }
}
