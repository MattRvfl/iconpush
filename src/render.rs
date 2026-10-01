//! Draws a 180×180 icon for an app in a pack style (Rust port of web/js/render.js).

use image::{imageops, RgbaImage};
use resvg::tiny_skia::{
    Color, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, RadialGradient, Rect, SpreadMode, Stroke,
    Transform,
};
use resvg::usvg;

use crate::data::{self, App, Background, GlyphStyle, Pack};
use crate::library::Logos;

pub const SIZE: u32 = 180;
const S: f32 = SIZE as f32;

fn color(hex: &str, alpha: f32) -> Color {
    let (r, g, b) = data::hex(hex);
    Color::from_rgba8(r, g, b, (alpha * 255.0).round() as u8)
}

fn brand_color<'a>(app: &App, logos: &'a Logos) -> Option<&'a str> {
    app.logo.as_ref().and_then(|slug| logos.colors.get(slug)).map(String::as_str)
}

fn luminance(hex: &str) -> f32 {
    let (r, g, b) = data::hex(hex);
    (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) / 255.0
}

fn pick(value: &str, colors: &std::collections::HashMap<String, String>, app: &App, logos: &Logos) -> String {
    match value {
        "category" => colors.get(&app.category).cloned().unwrap_or_else(|| "#888888".into()),
        // Brand color, except near-black brands (X, Apple Pay…) which would vanish on dark backgrounds.
        "brand" => match brand_color(app, logos) {
            Some(c) if luminance(c) > 0.12 => c.to_string(),
            _ => "#ffffff".into(),
        },
        v => v.to_string(),
    }
}

fn fill_full(pixmap: &mut Pixmap, paint: &Paint) {
    let rect = Rect::from_xywh(0.0, 0.0, S, S).unwrap();
    pixmap.fill_rect(rect, paint, Transform::identity(), None);
}

fn paint_background(pixmap: &mut Pixmap, bg: &Background, app: &App, logos: &Logos) {
    match bg {
        Background::Solid { color: c } => pixmap.fill(color(c, 1.0)),
        Background::Brand => pixmap.fill(color(brand_color(app, logos).unwrap_or("#2c2c34"), 1.0)),
        Background::Category { colors } => {
            pixmap.fill(color(colors.get(&app.category).map(String::as_str).unwrap_or("#cccccc"), 1.0))
        }
        Background::Linear { angle, stops } => {
            // Same convention as CSS: 180deg = top → bottom.
            let a = (angle - 90.0).to_radians();
            let r = S / 2.0;
            let (dx, dy) = (a.cos() * r, a.sin() * r);
            let n = stops.len().max(2) - 1;
            let grad = stops
                .iter()
                .enumerate()
                .map(|(i, c)| GradientStop::new(i as f32 / n as f32, color(c, 1.0)))
                .collect();
            let mut paint = Paint::default();
            if let Some(shader) = LinearGradient::new(
                Point::from_xy(r - dx, r - dy),
                Point::from_xy(r + dx, r + dy),
                grad,
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                paint.shader = shader;
                fill_full(pixmap, &paint);
            }
        }
    }
}

fn paint_glass(pixmap: &mut Pixmap) {
    let mut paint = Paint::default();

    // Soft highlight on the top part.
    if let Some(shader) = LinearGradient::new(
        Point::from_xy(0.0, 0.0),
        Point::from_xy(0.0, S * 0.6),
        vec![
            GradientStop::new(0.0, Color::from_rgba8(255, 255, 255, 71)),
            GradientStop::new(1.0, Color::from_rgba8(255, 255, 255, 0)),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    ) {
        paint.shader = shader;
        pixmap.fill_rect(Rect::from_xywh(0.0, 0.0, S, S * 0.6).unwrap(), &paint, Transform::identity(), None);
    }

    // Bluish glow from the top-left corner.
    let center = Point::from_xy(S * 0.3, S * 0.2);
    if let Some(shader) = RadialGradient::new(
        center,
        0.0,
        center,
        S * 0.7,
        vec![
            GradientStop::new(0.0, Color::from_rgba8(160, 190, 255, 64)),
            GradientStop::new(1.0, Color::from_rgba8(160, 190, 255, 0)),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    ) {
        paint.shader = shader;
        fill_full(pixmap, &paint);
    }

    // Faint inner rim.
    let mut rim = Paint::default();
    rim.set_color_rgba8(255, 255, 255, 46);
    rim.anti_alias = true;
    let path = PathBuilder::from_rect(Rect::from_xywh(1.5, 1.5, S - 3.0, S - 3.0).unwrap());
    let stroke = Stroke { width: 3.0, ..Stroke::default() };
    pixmap.stroke_path(&path, &rim, &stroke, Transform::identity(), None);
}

/// Render the symbol for `app` with the given color, centered: the official logo (filled
/// shapes) when the pack uses logos and it's downloaded, otherwise the Lucide glyph (strokes).
fn glyph_layer(glyph: &GlyphStyle, app: &App, hex_color: &str, logo: Option<&str>) -> Option<Pixmap> {
    let svg = match logo {
        Some(svg) => svg.replacen("<svg", &format!("<svg fill=\"{hex_color}\""), 1),
        None => data::glyph_svg(&app.glyph)?
            .replacen("stroke=\"currentColor\"", &format!("stroke=\"{hex_color}\""), 1)
            .replacen("stroke-width=\"2\"", &format!("stroke-width=\"{}\"", glyph.stroke), 1),
    };
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok()?;

    let size = S * glyph.scale;
    let offset = (S - size) / 2.0;
    let k = size / tree.size().width();
    let mut layer = Pixmap::new(SIZE, SIZE)?;
    resvg::render(&tree, Transform::from_row(k, 0.0, 0.0, k, offset, offset), &mut layer.as_mut());
    Some(layer)
}

/// Blur the alpha of `layer` and tint it, for the neon glow (3 box-blur passes ≈ gaussian).
fn glow_layer(layer: &Pixmap, hex_color: &str, radius: f32) -> Option<Pixmap> {
    let (w, h) = (SIZE as usize, SIZE as usize);
    let mut a: Vec<f32> = layer.pixels().iter().map(|p| p.alpha() as f32).collect();
    let r = (radius / 3.0).max(1.0) as isize;
    let mut tmp = vec![0.0f32; w * h];
    for _ in 0..3 {
        // Horizontal pass a → tmp, then vertical pass tmp → a.
        for y in 0..h {
            for x in 0..w {
                let mut sum = 0.0;
                for d in -r..=r {
                    let xx = (x as isize + d).clamp(0, w as isize - 1) as usize;
                    sum += a[y * w + xx];
                }
                tmp[y * w + x] = sum / (2 * r + 1) as f32;
            }
        }
        for x in 0..w {
            for y in 0..h {
                let mut sum = 0.0;
                for d in -r..=r {
                    let yy = (y as isize + d).clamp(0, h as isize - 1) as usize;
                    sum += tmp[yy * w + x];
                }
                a[y * w + x] = sum / (2 * r + 1) as f32;
            }
        }
    }

    let (cr, cg, cb) = data::hex(hex_color);
    let mut out = Pixmap::new(SIZE, SIZE)?;
    for (px, alpha) in out.pixels_mut().iter_mut().zip(a) {
        let al = (alpha * 1.6).min(255.0) as u8;
        let c = resvg::tiny_skia::ColorU8::from_rgba(cr, cg, cb, al).premultiply();
        *px = c;
    }
    Some(out)
}

fn to_rgba(pixmap: &Pixmap) -> RgbaImage {
    let mut img = RgbaImage::new(SIZE, SIZE);
    for (dst, src) in img.pixels_mut().zip(pixmap.pixels()) {
        let c = src.demultiply();
        *dst = image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]);
    }
    img
}

/// Scale and center-crop any image to a 180×180 square.
pub fn cover(img: &image::DynamicImage) -> RgbaImage {
    let img = img.to_rgba8();
    let k = (S / img.width() as f32).max(S / img.height() as f32);
    let (w, h) = (((img.width() as f32) * k).ceil() as u32, ((img.height() as f32) * k).ceil() as u32);
    let scaled = imageops::resize(&img, w.max(SIZE), h.max(SIZE), imageops::FilterType::Lanczos3);
    imageops::crop_imm(&scaled, (scaled.width() - SIZE) / 2, (scaled.height() - SIZE) / 2, SIZE, SIZE).to_image()
}

/// Render the icon for `app` in `pack`. `custom` replaces it with the user's own image.
pub fn render_icon(app: &App, pack: &Pack, custom: Option<&RgbaImage>, logos: &Logos) -> RgbaImage {
    if let Some(img) = custom {
        return img.clone();
    }
    if let Some(bytes) = data::pack_image(pack, &app.id) {
        if let Ok(img) = image::load_from_memory(bytes) {
            return cover(&img);
        }
    }

    let mut pixmap = Pixmap::new(SIZE, SIZE).expect("pixmap");
    let style = &pack.style;
    paint_background(&mut pixmap, &style.background, app, logos);
    if style.glass {
        paint_glass(&mut pixmap);
    }

    let hex_color = pick(&style.glyph.color, &style.glyph.colors, app, logos);
    let logo = if pack.logos { app.logo.as_ref().and_then(|slug| logos.svgs.get(slug)).map(String::as_str) } else { None };
    if let Some(layer) = glyph_layer(&style.glyph, app, &hex_color, logo) {
        let paint = resvg::tiny_skia::PixmapPaint::default();
        if style.glyph.glow > 0.0 {
            if let Some(glow) = glow_layer(&layer, &hex_color, style.glyph.glow) {
                pixmap.draw_pixmap(0, 0, glow.as_ref(), &paint, Transform::identity(), None);
            }
        }
        pixmap.draw_pixmap(0, 0, layer.as_ref(), &paint, Transform::identity(), None);
    }
    to_rgba(&pixmap)
}

pub fn png(img: &RgbaImage) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).expect("png encoding");
    out.into_inner()
}
