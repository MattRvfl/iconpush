# Contributing to iconpush

Thanks! Here is where help matters most, from easiest to hardest.

## 1. Test an app (5 minutes, no code)

Each app in [`web/data/apps.json`](web/data/apps.json) has a `verified` flag. To verify one:

1. Make a profile with that app in the web version or the desktop app and install it.
2. Tap the icon. Does the app open?
3. Open an issue or a PR with: the app, your iOS version, and what happened (opens directly / goes through Safari / doesn't work).

## 2. Add an app

Add an entry to `web/data/apps.json`:

```json
{ "id": "myapp", "name": "My App", "url": "myapp://", "bundleId": "com.company.myapp", "glyph": "star", "category": "tools", "verified": false }
```

- `url`: the app's URL scheme. Lists exist online; you can also find it in the app's `Info.plist` (`CFBundleURLSchemes`).
- `bundleId`: lets the desktop app detect if the app is installed.
- `glyph`: any [Lucide](https://lucide.dev/icons) icon name. Download its SVG into `web/glyphs/`.

## 3. Make a pack

See [docs/PACKS.md](docs/PACKS.md).

## 4. Code

- `src/` — the native Rust app: UI (`main.rs`, egui), icon rendering (`render.rs`), profile builder (`profile.rs`), USB thread (`worker.rs`, `device.rs`, `mcinstall.rs`).
- `web/` — the web version (plain HTML/CSS/JS modules, no build step) **and** the shared data (`data/apps.json`, `packs/`, `glyphs/`) embedded in the desktop app. If you change how icons are drawn, update both `src/render.rs` and `web/js/render.js`.

Build: `cargo build`. Run: `cargo run`.

Keep pull requests focused, and describe how you tested (iOS version, device).
