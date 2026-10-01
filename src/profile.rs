//! Builds the .mobileconfig: one Web Clip (com.apple.webClip.managed) per icon.
//! Format reference: https://developer.apple.com/documentation/devicemanagement/webclip

use plist::{Dictionary, Value};

/// U+2800 (braille blank) shows as an empty label on the home screen.
pub const BLANK_LABEL: &str = "\u{2800}";

pub struct Clip {
    pub label: String,
    pub url: String,
    pub png: Vec<u8>,
}

/// Random UUID-shaped string (uppercase), without pulling a uuid crate.
pub fn random_uuid() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut bytes = Vec::with_capacity(16);
    for i in 0..2u64 {
        let mut h = RandomState::new().build_hasher();
        h.write_u64(i);
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        );
        bytes.extend_from_slice(&h.finish().to_be_bytes());
    }
    let hex: String = bytes.iter().map(|b| format!("{b:02X}")).collect();
    format!("{}-{}-{}-{}-{}", &hex[0..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..32])
}

pub fn build(name: &str, clips: Vec<Clip>) -> Vec<u8> {
    let payloads = clips
        .into_iter()
        .map(|c| {
            let id = random_uuid();
            let mut d = Dictionary::new();
            d.insert("FullScreen".into(), false.into());
            d.insert("Icon".into(), Value::Data(c.png));
            d.insert("IsRemovable".into(), true.into());
            let display = if c.label.trim_matches(|ch: char| ch.is_whitespace() || ch == '\u{2800}').is_empty() {
                c.url.clone()
            } else {
                c.label.clone()
            };
            d.insert("Label".into(), c.label.into());
            d.insert("PayloadDisplayName".into(), display.into());
            d.insert("PayloadIdentifier".into(), format!("dev.iconpush.clip.{id}").into());
            d.insert("PayloadType".into(), "com.apple.webClip.managed".into());
            d.insert("PayloadUUID".into(), id.into());
            d.insert("PayloadVersion".into(), 1.into());
            d.insert("Precomposed".into(), true.into());
            d.insert("URL".into(), c.url.into());
            Value::Dictionary(d)
        })
        .collect::<Vec<_>>();

    let root_id = random_uuid();
    let mut root = Dictionary::new();
    root.insert("PayloadContent".into(), Value::Array(payloads));
    root.insert(
        "PayloadDescription".into(),
        "Icônes personnalisées créées avec iconpush (https://github.com/MattRvfl/iconpush).".into(),
    );
    root.insert("PayloadDisplayName".into(), name.into());
    root.insert("PayloadIdentifier".into(), format!("dev.iconpush.{root_id}").into());
    root.insert("PayloadOrganization".into(), "iconpush".into());
    root.insert("PayloadRemovalDisallowed".into(), false.into());
    root.insert("PayloadType".into(), "Configuration".into());
    root.insert("PayloadUUID".into(), root_id.into());
    root.insert("PayloadVersion".into(), 1.into());

    let mut out = Vec::new();
    Value::Dictionary(root).to_writer_xml(&mut out).expect("plist xml");
    out
}
