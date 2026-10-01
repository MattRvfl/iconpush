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

- `web/` — the UI (plain HTML/CSS/JS modules, no build step). Shared by the website and the desktop app.
- `src/` — the Rust desktop app: local server (`main.rs`), USB device access (`device.rs`), profile service (`mcinstall.rs`).

Build: `cargo build`. Run: `cargo run`, then the browser opens.

Keep pull requests focused, and describe how you tested (iOS version, device).
