//! Export: composite the 8 pages onto one folded-zine sheet and write a PDF.
//!
//! The geometry mirrors the on-screen preview (PageViewer.svelte): every size is a fraction of
//! the page width, and image framing uses the same `object-fit` / `object-position` math.

use crate::images::load_oriented;
use crate::project::{CaptionStyle, PageData, Project, SlotData};
use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use image::{
    DynamicImage, ExtendedColorType, Rgba, RgbaImage,
    codecs::jpeg::JpegEncoder,
    imageops::{self, FilterType},
};
use std::{fs, path::Path};

const FONT_BYTES: &[u8] = include_bytes!("../assets/ArchivoBlack-Regular.ttf");
const DPI: f64 = 300.0;

/// Landscape sheet size in inches: (width, height).
fn sheet_inches(paper: &str) -> Result<(f64, f64), String> {
    match paper {
        "letter" => Ok((11.0, 8.5)),
        "a4" => Ok((297.0 / 25.4, 210.0 / 25.4)),
        other => Err(format!("unknown paper size '{other}'")),
    }
}

/// Page numbers (0-based) per sheet cell. The top row is printed upside down so the sheet folds
/// into a booklet: top 5 4 3 2, bottom 6 7 8 1.
const TOP_ROW: [usize; 4] = [4, 3, 2, 1];
const BOTTOM_ROW: [usize; 4] = [5, 6, 7, 0];

fn parse_aspect(aspect: &str) -> Option<f64> {
    let (w, h) = aspect.split_once(':')?;
    Some(w.parse::<f64>().ok()? / h.parse::<f64>().ok()?)
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Rectangles (page-local px) for each present image, mirroring the stacked flex layout.
fn slot_frames(slots: &[&SlotData], w: f64, h: f64) -> Vec<Rect> {
    let n = slots.len();
    if n == 0 {
        return vec![];
    }
    let pad = if n > 1 { 0.02 * w } else { 0.0 };
    let gap = pad;
    let (ix, iy, iw, ih) = (pad, pad, w - 2.0 * pad, h - 2.0 * pad);
    let cell_h = (ih - gap * (n as f64 - 1.0)) / n as f64;
    slots
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let cell = Rect { x: ix, y: iy + i as f64 * (cell_h + gap), w: iw, h: cell_h };
            match parse_aspect(&s.aspect) {
                None => cell,
                Some(r) => {
                    let fw = cell.w.min(r * cell.h);
                    let fh = fw / r;
                    Rect { x: cell.x + (cell.w - fw) / 2.0, y: cell.y + (cell.h - fh) / 2.0, w: fw, h: fh }
                }
            }
        })
        .collect()
}

/// Draw `img` into `frame` like CSS `object-fit: cover|contain` with `object-position`.
fn draw_image(canvas: &mut RgbaImage, img: &DynamicImage, frame: Rect, fill: bool, ox: f64, oy: f64) {
    let (iw, ih) = (img.width() as f64, img.height() as f64);
    let (sx, sy) = (frame.w / iw, frame.h / ih);
    let s = if fill { sx.max(sy) } else { sx.min(sy) };
    let (sw, sh) = (iw * s, ih * s);
    let left = frame.x + (frame.w - sw) * ox / 100.0;
    let top = frame.y + (frame.h - sh) * oy / 100.0;

    // Visible part of the scaled image, in canvas px.
    let x0 = left.max(frame.x).round();
    let y0 = top.max(frame.y).round();
    let x1 = (left + sw).min(frame.x + frame.w).round();
    let y1 = (top + sh).min(frame.y + frame.h).round();
    let (vw, vh) = ((x1 - x0) as i64, (y1 - y0) as i64);
    if vw < 1 || vh < 1 {
        return;
    }

    // Matching source crop.
    let cx = (((x0 - left) / s).round().max(0.0) as u32).min(img.width() - 1);
    let cy = (((y0 - top) / s).round().max(0.0) as u32).min(img.height() - 1);
    let cw = ((vw as f64 / s).round().max(1.0) as u32).min(img.width() - cx);
    let ch = ((vh as f64 / s).round().max(1.0) as u32).min(img.height() - cy);
    let part = img.crop_imm(cx, cy, cw, ch).resize_exact(vw as u32, vh as u32, FilterType::Lanczos3);
    imageops::overlay(canvas, &part.to_rgba8(), x0 as i64, y0 as i64);
}

// ---- cover caption text ----

struct Fonts<'a> {
    font: FontRef<'a>,
}

impl Fonts<'_> {
    fn scale(&self, px: f64) -> PxScale {
        let upm = self.font.units_per_em().unwrap_or(1000.0) as f64;
        PxScale::from((px * self.font.height_unscaled() as f64 / upm) as f32)
    }

    fn width(&self, text: &str, px: f64) -> f64 {
        let scaled = self.font.as_scaled(self.scale(px));
        let mut width = 0.0f32;
        let mut prev = None;
        for ch in text.chars() {
            let id = self.font.glyph_id(ch);
            if let Some(p) = prev {
                width += scaled.kern(p, id);
            }
            width += scaled.h_advance(id);
            prev = Some(id);
        }
        width as f64
    }

    /// Ascent and descent (positive) in px for an em of `px`.
    fn ascent_descent(&self, px: f64) -> (f64, f64) {
        let scaled = self.font.as_scaled(self.scale(px));
        (scaled.ascent() as f64, -scaled.descent() as f64)
    }

    /// Greedy word wrap with `white-space: pre-wrap; overflow-wrap: break-word` behaviour.
    fn wrap(&self, text: &str, px: f64, max_w: f64) -> Vec<String> {
        let mut lines = Vec::new();
        for para in text.split('\n') {
            let mut line = String::new();
            for word in para.split(' ') {
                let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
                if self.width(&candidate, px) <= max_w || line.is_empty() && word.is_empty() {
                    line = candidate;
                    continue;
                }
                if !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                }
                // A single word wider than the line is broken by character.
                let mut chunk = String::new();
                for ch in word.chars() {
                    let next = format!("{chunk}{ch}");
                    if self.width(&next, px) > max_w && !chunk.is_empty() {
                        lines.push(std::mem::take(&mut chunk));
                    }
                    chunk.push(ch);
                }
                line = chunk;
            }
            lines.push(line);
        }
        lines
    }

    fn draw(&self, canvas: &mut RgbaImage, text: &str, px: f64, x: f64, baseline: f64, color: [u8; 3]) {
        let scale = self.scale(px);
        let scaled = self.font.as_scaled(scale);
        let mut caret = x as f32;
        let mut prev = None;
        for ch in text.chars() {
            let id = self.font.glyph_id(ch);
            if let Some(p) = prev {
                caret += scaled.kern(p, id);
            }
            let glyph = id.with_scale_and_position(scale, point(caret, baseline as f32));
            caret += scaled.h_advance(id);
            prev = Some(id);
            if let Some(outlined) = self.font.outline_glyph(glyph) {
                let b = outlined.px_bounds();
                let (cw, chh) = (canvas.width() as i32, canvas.height() as i32);
                outlined.draw(|gx, gy, cov| {
                    let (px_x, px_y) = (b.min.x as i32 + gx as i32, b.min.y as i32 + gy as i32);
                    if px_x < 0 || px_y < 0 || px_x >= cw || px_y >= chh {
                        return;
                    }
                    let dst = canvas.get_pixel_mut(px_x as u32, px_y as u32);
                    for c in 0..3 {
                        dst[c] = (dst[c] as f32 * (1.0 - cov) + color[c] as f32 * cov).round() as u8;
                    }
                });
            }
        }
    }
}

struct Line {
    text: String,
    px: f64,
}

/// Draw the caption / sub-caption / author block like the cover overlay in PageViewer.svelte.
fn draw_caption(canvas: &mut RgbaImage, fonts: &Fonts, page: &PageData, w: f64, h: f64) {
    let cs: &CaptionStyle = &page.caption_style;
    let base = w * match cs.size.as_str() {
        "sm" => 0.06,
        "lg" => 0.13,
        _ => 0.09,
    };
    let parts = [(&page.caption, 1.0), (&page.sub_caption, 0.7), (&page.author, 0.5)];
    if parts.iter().all(|(t, _)| t.is_empty()) {
        return;
    }

    // Box width, like the browser's shrink-to-fit: the text's max-content width, reduced toward
    // the room available but never below its widest word (break-word only splits words that
    // can't fit even then), and capped by max-width.
    let (mut min_content, mut max_content) = (0.0f64, 0.0f64);
    for (text, factor) in &parts {
        let px = base * factor;
        for para in text.split('\n') {
            max_content = max_content.max(fonts.width(para, px));
            for word in para.split(' ') {
                min_content = min_content.max(fonts.width(word, px));
            }
        }
    }
    // Aligned: a flex item inside the overlay's content box (page minus 5% per side), whose
    // max-width is 90% of that box. Custom: absolutely positioned, so the room is what remains
    // right of the anchor point and max-width is 90% of the page.
    let (avail, max_width) = if cs.custom {
        (w - cs.x / 100.0 * w, 0.9 * w)
    } else {
        (0.9 * w, 0.81 * w)
    };
    let block_w = max_width.min(min_content.max(max_content.min(avail)));

    let mut lines: Vec<Line> = Vec::new();
    for (text, factor) in parts {
        if text.is_empty() {
            continue;
        }
        let px = base * factor;
        for t in fonts.wrap(text, px, block_w) {
            lines.push(Line { text: t, px });
        }
    }
    let block_h: f64 = lines.iter().map(|l| l.px).sum(); // line-height is 1 em

    let inset = 0.05 * w;
    let (bx, by) = if cs.custom {
        (cs.x / 100.0 * w - block_w / 2.0, cs.y / 100.0 * h - block_h / 2.0)
    } else {
        let bx = match cs.h.as_str() {
            "left" => inset,
            "right" => w - inset - block_w,
            _ => inset + (0.9 * w - block_w) / 2.0,
        };
        let by = match cs.v.as_str() {
            "top" => inset,
            "middle" => inset + (h - 2.0 * inset - block_h) / 2.0,
            _ => h - inset - block_h,
        };
        (bx, by)
    };

    let color = if cs.color == "black" { [0, 0, 0] } else { [255, 255, 255] };
    let mut y = by;
    for l in &lines {
        let lw = fonts.width(&l.text, l.px);
        let x = match cs.h.as_str() {
            "left" => bx,
            "right" => bx + block_w - lw,
            _ => bx + (block_w - lw) / 2.0,
        };
        let (asc, desc) = fonts.ascent_descent(l.px);
        let baseline = y + (l.px - (asc + desc)) / 2.0 + asc;
        fonts.draw(canvas, &l.text, l.px, x, baseline, color);
        y += l.px;
    }
}

fn render_page(page: &PageData, index: usize, w: u32, h: u32, fonts: &Fonts) -> Result<RgbaImage, String> {
    let mut canvas = RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]));
    let present: Vec<&SlotData> = page.images.iter().filter(|s| s.path.is_some()).collect();
    let frames = slot_frames(&present, w as f64, h as f64);
    for (slot, frame) in present.iter().zip(frames) {
        let path = slot.path.as_deref().unwrap_or_default();
        let img = load_oriented(Path::new(path))?;
        draw_image(&mut canvas, &img, frame, slot.mode == "fill", slot.offset_x, slot.offset_y);
    }
    if index == 0 || index == 7 {
        draw_caption(&mut canvas, fonts, page, w as f64, h as f64);
    }
    Ok(canvas)
}

/// Build the full sheet bitmap for a project.
pub fn render_sheet(project: &Project) -> Result<RgbaImage, String> {
    let (wi, hi) = sheet_inches(&project.paper)?;
    let (sheet_w, sheet_h) = ((wi * DPI).round() as u32, (hi * DPI).round() as u32);
    let (cell_w, cell_h) = (sheet_w / 4, sheet_h / 2);
    let fonts = Fonts { font: FontRef::try_from_slice(FONT_BYTES).map_err(|e| e.to_string())? };

    // Render pages in parallel (a few at a time, since each holds a decoded photo in memory).
    let cells: Vec<(usize, usize, usize)> = [TOP_ROW, BOTTOM_ROW]
        .iter()
        .enumerate()
        .flat_map(|(row, pages)| pages.iter().enumerate().map(move |(col, &page)| (row, col, page)))
        .collect();
    let workers = std::thread::available_parallelism().map_or(2, |n| n.get()).clamp(1, 4);
    let mut rendered: Vec<(usize, usize, RgbaImage)> = Vec::new();
    for batch in cells.chunks(workers) {
        let results: Vec<Result<(usize, usize, RgbaImage), String>> = std::thread::scope(|scope| {
            let handles: Vec<_> = batch
                .iter()
                .map(|&(row, col, page_index)| {
                    let fonts = &fonts;
                    scope.spawn(move || {
                        let mut cell = render_page(&project.pages[page_index], page_index, cell_w, cell_h, fonts)?;
                        if row == 0 {
                            cell = imageops::rotate180(&cell);
                        }
                        Ok((row, col, cell))
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap_or_else(|_| Err("a page failed to render".to_string())))
                .collect()
        });
        for r in results {
            rendered.push(r?);
        }
    }

    let mut sheet = RgbaImage::from_pixel(sheet_w, sheet_h, Rgba([255, 255, 255, 255]));
    for (row, col, cell) in rendered {
        imageops::overlay(&mut sheet, &cell, (col as u32 * cell_w) as i64, (row as u32 * cell_h) as i64);
    }
    Ok(sheet)
}

fn write_pdf(sheet: &RgbaImage, paper: &str, out: &Path) -> Result<(), String> {
    use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect as PdfRect, Ref};

    let mut jpeg = Vec::new();
    let rgb = DynamicImage::ImageRgba8(sheet.clone()).to_rgb8();
    JpegEncoder::new_with_quality(&mut jpeg, 92)
        .encode(rgb.as_raw(), rgb.width(), rgb.height(), ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;

    let (wi, hi) = sheet_inches(paper)?;
    let (pw, ph) = ((wi * 72.0) as f32, (hi * 72.0) as f32);

    let (catalog, pages, page, content, image) = (Ref::new(1), Ref::new(2), Ref::new(3), Ref::new(4), Ref::new(5));
    let mut pdf = Pdf::new();
    pdf.catalog(catalog).pages(pages);
    pdf.pages(pages).kids([page]).count(1);
    {
        let mut p = pdf.page(page);
        p.media_box(PdfRect::new(0.0, 0.0, pw, ph));
        p.parent(pages);
        p.contents(content);
        p.resources().x_objects().pair(Name(b"Im0"), image);
        p.finish();
    }
    {
        let mut img = pdf.image_xobject(image, &jpeg);
        img.filter(Filter::DctDecode);
        img.width(rgb.width() as i32);
        img.height(rgb.height() as i32);
        img.color_space().device_rgb();
        img.bits_per_component(8);
        img.finish();
    }
    let mut c = Content::new();
    c.save_state();
    c.transform([pw, 0.0, 0.0, ph, 0.0, 0.0]);
    c.x_object(Name(b"Im0"));
    c.restore_state();
    pdf.stream(content, &c.finish());

    fs::write(out, pdf.finish()).map_err(|e| format!("could not write {}: {e}", out.display()))
}

fn export(project: &Project, out: &Path) -> Result<(), String> {
    let is_pdf = out.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
    if !is_pdf {
        return Err("export only supports PDF (use a .pdf file name)".into());
    }
    let sheet = render_sheet(project)?;
    write_pdf(&sheet, &project.paper, out)
}

#[tauri::command]
pub async fn export_zine(project: Project, path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || export(&project, Path::new(&path)))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::PAGE_COUNT;
    use image::{Rgb, RgbImage};

    fn slot(path: Option<String>, mode: &str, aspect: &str) -> SlotData {
        SlotData { path, mode: mode.into(), offset_x: 50.0, offset_y: 50.0, aspect: aspect.into() }
    }

    fn page(images: Vec<SlotData>, caption: &str) -> PageData {
        PageData {
            caption: caption.into(),
            sub_caption: String::new(),
            author: String::new(),
            caption_style: CaptionStyle {
                h: "center".into(),
                v: "middle".into(),
                size: "lg".into(),
                color: "black".into(),
                custom: false,
                x: 50.0,
                y: 50.0,
            },
            images,
        }
    }

    #[test]
    fn text_width_scales_with_em_size() {
        let fonts = Fonts { font: FontRef::try_from_slice(FONT_BYTES).unwrap() };
        let upm = fonts.font.units_per_em().unwrap() as f64;
        let text = "My Zine Title";
        let units: f64 = text.chars().map(|c| fonts.font.h_advance_unscaled(fonts.font.glyph_id(c)) as f64).sum();
        // Kerning aside, width at font-size px is advance-units / units-per-em * px.
        let w = fonts.width(text, 100.0);
        println!("upm={upm} units={units} width@100={w} expected~{}", units / upm * 100.0);
        assert!((w - units / upm * 100.0).abs() < 0.05 * w, "width {w} vs {}", units / upm * 100.0);
    }

    #[test]
    fn custom_caption_near_edge_keeps_words_whole() {
        // A caption anchored near the right edge has little room, but like the browser it
        // stays as wide as its longest word instead of splitting it across lines.
        let fonts = Fonts { font: FontRef::try_from_slice(FONT_BYTES).unwrap() };
        let mut p = page(vec![slot(None, "fit", "auto"), slot(None, "fit", "auto")], "Wonderful");
        p.caption_style.custom = true;
        p.caption_style.x = 90.0;
        p.caption_style.y = 50.0;
        p.caption_style.color = "black".into();
        let mut canvas = RgbaImage::from_pixel(825, 1275, Rgba([255, 255, 255, 255]));
        draw_caption(&mut canvas, &fonts, &p, 825.0, 1275.0);
        // One line of text: dark pixels only within a single line-height band around y=50%.
        let (mut top, mut bottom) = (u32::MAX, 0);
        for (_, y, px) in canvas.enumerate_pixels() {
            if px.0[0] < 128 {
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
        let line_h = 825.0 * 0.13;
        assert!((bottom - top) as f64 <= line_h, "word was split across lines: {top}..{bottom}");
    }

    /// Timing check: `cargo test --release export_timing -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn export_timing() {
        use std::time::Instant;
        let dir = std::env::temp_dir().join("looseleaf_timing");
        fs::create_dir_all(&dir).unwrap();
        let photo = dir.join("big.jpg");
        // 24 MP gradient so JPEG decode/resize cost is realistic.
        let img = RgbImage::from_fn(6000, 4000, |x, y| Rgb([(x / 24) as u8, (y / 16) as u8, ((x + y) / 40) as u8]));
        img.save(&photo).unwrap();
        let p = photo.to_string_lossy().to_string();
        let pages: Vec<PageData> = (0..PAGE_COUNT)
            .map(|_| page(vec![slot(Some(p.clone()), "fill", "auto"), slot(Some(p.clone()), "fit", "auto")], "Title"))
            .collect();
        let project = Project { paper: "letter".into(), pages };

        let t = Instant::now();
        let sheet = render_sheet(&project).unwrap();
        println!("render_sheet (16 x 24MP photos): {:?}", t.elapsed());
        let t = Instant::now();
        write_pdf(&sheet, "letter", &dir.join("t.pdf")).unwrap();
        println!("write_pdf: {:?}", t.elapsed());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn fill_covers_frame_and_fit_letterboxes() {
        // 2:1 landscape image into a 100x100 frame.
        let img = DynamicImage::ImageRgb8(RgbImage::from_pixel(200, 100, Rgb([255, 0, 0])));
        let frame = Rect { x: 0.0, y: 0.0, w: 100.0, h: 100.0 };

        let mut fill = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        draw_image(&mut fill, &img, frame, true, 50.0, 50.0);
        assert_eq!(fill.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(fill.get_pixel(99, 99).0, [255, 0, 0, 255]);

        let mut fit = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        draw_image(&mut fit, &img, frame, false, 50.0, 50.0);
        assert_eq!(fit.get_pixel(50, 5).0, [255, 255, 255, 255]); // letterbox band
        assert_eq!(fit.get_pixel(50, 50).0, [255, 0, 0, 255]);
    }

    #[test]
    fn fill_offset_pans_the_crop() {
        // Left half blue, right half red; fill into a square shows one side depending on offset.
        let mut src = RgbImage::from_pixel(200, 100, Rgb([0, 0, 255]));
        for x in 100..200 {
            for y in 0..100 {
                src.put_pixel(x, y, Rgb([255, 0, 0]));
            }
        }
        let img = DynamicImage::ImageRgb8(src);
        let frame = Rect { x: 0.0, y: 0.0, w: 100.0, h: 100.0 };
        let mut left = RgbaImage::new(100, 100);
        draw_image(&mut left, &img, frame, true, 0.0, 50.0);
        assert_eq!(left.get_pixel(50, 50).0, [0, 0, 255, 255]);
        let mut right = RgbaImage::new(100, 100);
        draw_image(&mut right, &img, frame, true, 100.0, 50.0);
        assert_eq!(right.get_pixel(50, 50).0, [255, 0, 0, 255]);
    }

    #[test]
    fn two_images_stack_and_aspect_frames_center() {
        let a = slot(Some("a".into()), "fit", "auto");
        let b = slot(Some("b".into()), "fit", "1:1");
        let frames = slot_frames(&[&a, &b], 1000.0, 1500.0);
        assert_eq!(frames.len(), 2);
        assert!(frames[0].y < frames[1].y);
        assert!((frames[1].w - frames[1].h).abs() < 1e-6);
    }

    #[test]
    fn exports_pdf_with_expected_sheet() {
        let dir = std::env::temp_dir().join("looseleaf_export_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let photo = dir.join("p.png");
        RgbImage::from_pixel(1200, 800, Rgb([20, 120, 200])).save(&photo).unwrap();
        let p = photo.to_string_lossy().to_string();

        let mut pages: Vec<PageData> =
            (0..PAGE_COUNT).map(|_| page(vec![slot(None, "fit", "auto"), slot(None, "fit", "auto")], "")).collect();
        pages[0] = page(vec![slot(Some(p.clone()), "fill", "auto"), slot(None, "fit", "auto")], "Hello");
        pages[1] = page(vec![slot(Some(p.clone()), "fit", "auto"), slot(Some(p), "fill", "3:2")], "");
        let project = Project { paper: "letter".into(), pages };

        let sheet = render_sheet(&project).unwrap();
        assert_eq!((sheet.width(), sheet.height()), (3300, 2550));
        // Front cover is bottom-right (cell col 3, row 1) and filled blue; caption text drew black pixels.
        assert_eq!(sheet.get_pixel(3300 - 20, 2550 - 20).0, [20, 120, 200, 255]);
        let cover = imageops::crop_imm(&sheet, 2475, 1275, 825, 1275).to_image();
        assert!(cover.pixels().any(|p| p.0 == [0, 0, 0, 255]), "caption should be drawn");
        // Page 2 sits top-right, rotated 180 degrees; its fit image is centred.
        assert_eq!(sheet.get_pixel(2475 + 400, 374).0[2], 200);

        let pdf = dir.join("out.pdf");
        export(&project, &pdf).unwrap();
        let bytes = fs::read(&pdf).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(export(&project, &dir.join("out.png")).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
