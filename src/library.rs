//! Icon library: official brand logos downloaded from Simple Icons (CC0, simpleicons.org)
//! and importing a folder of images (for example icons exported from Figma).
//!
//! Everything is stored next to the executable, in `iconpush-data/`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use crate::data::App;

const SIMPLE_ICONS: &str = "https://cdn.jsdelivr.net/npm/simple-icons@latest";

/// `iconpush-data/` next to the exe (keeps everything on the same drive as the app).
pub fn data_dir() -> PathBuf {
    let base = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(std::env::temp_dir);
    base.join("iconpush-data")
}

fn logos_dir() -> PathBuf {
    data_dir().join("logos")
}

pub fn agent() -> ureq::Agent {
    use ureq::tls::{TlsConfig, TlsProvider};
    ureq::Agent::config_builder()
        .tls_config(TlsConfig::builder().provider(TlsProvider::NativeTls).build())
        .user_agent(concat!("iconpush/", env!("CARGO_PKG_VERSION"), " (+https://github.com/MattRvfl/iconpush)"))
        .timeout_connect(Some(Duration::from_secs(15)))
        .build()
        .into()
}

/// Downloaded logos: slug → SVG source, slug → brand color (#rrggbb).
#[derive(Default, Clone)]
pub struct Logos {
    pub svgs: HashMap<String, String>,
    pub colors: HashMap<String, String>,
}

impl Logos {
    pub fn is_empty(&self) -> bool {
        self.svgs.is_empty()
    }
}

pub fn load_logos() -> Logos {
    let dir = logos_dir();
    let colors: HashMap<String, String> = std::fs::read(dir.join("colors.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let mut svgs = HashMap::new();
    for slug in colors.keys() {
        if let Ok(svg) = std::fs::read_to_string(dir.join(format!("{slug}.svg"))) {
            svgs.insert(slug.clone(), svg);
        }
    }
    Logos { svgs, colors }
}

#[derive(Deserialize)]
struct SiEntry {
    slug: String,
    hex: String,
}

/// Download the logos needed by `apps`. `progress(done, total)` is called after each file.
pub fn download_logos(apps: &[App], progress: impl Fn(usize, usize)) -> Result<Logos, String> {
    let wanted: Vec<&str> = apps.iter().filter_map(|a| a.logo.as_deref()).collect();
    let dir = logos_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("dossier {} : {e}", dir.display()))?;

    let agent = agent();
    let raw = agent
        .get(&format!("{SIMPLE_ICONS}/data/simple-icons.json"))
        .call()
        .map_err(|e| format!("Simple Icons injoignable : {e}"))?
        .body_mut()
        .with_config()
        .limit(20 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| format!("liste Simple Icons : {e}"))?;
    let index: Vec<SiEntry> = serde_json::from_slice(&raw).map_err(|e| format!("liste Simple Icons illisible : {e}"))?;

    let colors: HashMap<String, String> = index
        .into_iter()
        .filter(|e| wanted.contains(&e.slug.as_str()))
        .map(|e| (e.slug, format!("#{}", e.hex)))
        .collect();

    let total = colors.len();
    let mut logos = Logos { colors: colors.clone(), ..Default::default() };
    for (done, slug) in colors.keys().enumerate() {
        let svg = agent
            .get(&format!("{SIMPLE_ICONS}/icons/{slug}.svg"))
            .call()
            .map_err(|e| format!("logo {slug} : {e}"))?
            .body_mut()
            .read_to_string()
            .map_err(|e| format!("logo {slug} : {e}"))?;
        if !svg.trim_start().starts_with("<svg") {
            return Err(format!("logo {slug} : réponse inattendue"));
        }
        std::fs::write(dir.join(format!("{slug}.svg")), &svg).map_err(|e| e.to_string())?;
        logos.svgs.insert(slug.clone(), svg);
        progress(done + 1, total);
    }
    std::fs::write(dir.join("colors.json"), serde_json::to_vec_pretty(&colors).unwrap_or_default())
        .map_err(|e| e.to_string())?;
    Ok(logos)
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'à' | 'â' => 'a',
            'ç' => 'c',
            c => c,
        })
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

/// Match image files in `dir` to apps by file name: "spotify.png", "Spotify.png",
/// "Google Maps.png", "googlemaps@3x.png"… all work. Returns (app id, file path).
pub fn match_folder(dir: &Path, apps: &[App]) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let stem = normalize(stem.split('@').next().unwrap_or(stem));
        if let Some(app) = apps.iter().find(|a| {
            normalize(&a.id) == stem
                || normalize(&a.name) == stem
                || a.logo.as_deref().is_some_and(|l| normalize(l) == stem)
        }) {
            out.push((app.id.clone(), path));
        }
    }
    out
}
