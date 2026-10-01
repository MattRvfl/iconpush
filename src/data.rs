//! Apps, packs and glyphs. The files live in `web/` so the desktop app and the web
//! version share exactly the same data; they are embedded in the executable.

use std::collections::HashMap;

use include_dir::{include_dir, Dir};
use serde::Deserialize;

pub static WEB: Dir = include_dir!("$CARGO_MANIFEST_DIR/web");

#[derive(Deserialize, Clone, Debug)]
pub struct App {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(rename = "bundleId")]
    pub bundle_id: String,
    pub glyph: String,
    /// Simple Icons slug of the official logo, if there is one.
    #[serde(default)]
    pub logo: Option<String>,
    pub category: String,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Pack {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub style: Style,
    /// App id → image file inside `web/packs/<pack id>/`.
    #[serde(default)]
    pub images: HashMap<String, String>,
    /// Draw the official brand logo (downloaded from Simple Icons) instead of a Lucide symbol.
    #[serde(default)]
    pub logos: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Style {
    pub background: Background,
    #[serde(default)]
    pub glass: bool,
    pub glyph: GlyphStyle,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Background {
    Solid { color: String },
    Linear { angle: f32, stops: Vec<String> },
    Category { colors: HashMap<String, String> },
    /// The app's brand color (logo packs only).
    Brand,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GlyphStyle {
    /// A color, or "category" to pick from `colors`.
    pub color: String,
    #[serde(default)]
    pub colors: HashMap<String, String>,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default = "default_stroke")]
    pub stroke: f32,
    #[serde(default)]
    pub glow: f32,
}

fn default_scale() -> f32 {
    0.5
}
fn default_stroke() -> f32 {
    2.0
}

#[derive(Deserialize)]
struct AppsFile {
    apps: Vec<App>,
}

#[derive(Deserialize)]
struct PacksFile {
    packs: Vec<Pack>,
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &str) -> T {
    let file = WEB.get_file(path).unwrap_or_else(|| panic!("{path} manquant dans web/"));
    serde_json::from_slice(file.contents()).unwrap_or_else(|e| panic!("{path} invalide : {e}"))
}

pub fn apps() -> Vec<App> {
    read_json::<AppsFile>("data/apps.json").apps
}

pub fn packs() -> Vec<Pack> {
    read_json::<PacksFile>("packs/index.json").packs
}

/// Packs built on the official logos; shown once the logos are downloaded (Bibliothèque tab).
pub fn logo_packs() -> Vec<Pack> {
    let glyph = |color: &str, scale: f32| GlyphStyle {
        color: color.into(),
        colors: HashMap::new(),
        scale,
        stroke: 1.8,
        glow: 0.0,
    };
    let pack = |id: &str, name: &str, description: &str, background: Background, glass: bool, glyph: GlyphStyle| Pack {
        id: id.into(),
        name: name.into(),
        author: "Simple Icons (CC0)".into(),
        description: description.into(),
        tags: vec!["logos".into(), "officiel".into(), "marques".into()],
        style: Style { background, glass, glyph },
        images: HashMap::new(),
        logos: true,
    };
    vec![
        pack("logos-color", "Logos officiels", "Le vrai logo, sur la couleur de la marque.", Background::Brand, false, glyph("#ffffff", 0.52)),
        pack(
            "logos-black",
            "Logos sur noir",
            "Logo en couleur sur fond noir, très OLED.",
            Background::Solid { color: "#000000".into() },
            false,
            glyph("brand", 0.5),
        ),
        pack(
            "logos-glass",
            "Logos Liquid Glass",
            "Logos blancs sur verre dépoli sombre.",
            Background::Linear { angle: 160.0, stops: vec!["#2b3245".into(), "#0f1220".into()] },
            true,
            glyph("#ffffff", 0.48),
        ),
    ]
}

pub fn glyph_svg(name: &str) -> Option<&'static str> {
    WEB.get_file(format!("glyphs/{name}.svg"))?.contents_utf8()
}

pub fn pack_image(pack: &Pack, app_id: &str) -> Option<&'static [u8]> {
    let file = pack.images.get(app_id)?;
    Some(WEB.get_file(format!("packs/{}/{file}", pack.id))?.contents())
}

pub const CATEGORIES: &[(&str, &str)] = &[
    ("music", "Musique"),
    ("social", "Réseaux"),
    ("messaging", "Messages"),
    ("video", "Vidéo"),
    ("tools", "Outils"),
    ("travel", "Déplacements"),
    ("system", "Apple"),
];

/// "#rrggbb" → (r, g, b). Invalid colors become mid gray.
pub fn hex(color: &str) -> (u8, u8, u8) {
    let c = color.trim_start_matches('#');
    let p = |i: usize| u8::from_str_radix(c.get(i..i + 2).unwrap_or("88"), 16).unwrap_or(0x88);
    (p(0), p(2), p(4))
}
