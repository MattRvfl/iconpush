<div align="center">

# iconpush

**Custom iPhone home screen icons. No jailbreak, no Shortcuts banner.**
Plug your iPhone in, pick an icon pack, click *Send*. Done.

![iconpush](docs/screenshot.png)

**[⬇ Download for Windows](https://github.com/MattRvfl/iconpush/releases/latest)** · **[🌐 Web version](https://mattrvfl.github.io/iconpush/)** · **[📖 Guide (FR)](docs/GUIDE.fr.md)** · **[🎨 Make a pack](docs/PACKS.md)**

</div>

## Why?

The usual trick to change an app icon on iPhone uses the **Shortcuts** app: it works, but every tap flashes a "Shortcuts" banner, and you have to build each icon by hand, one by one.

iconpush uses a different, official iOS mechanism: **configuration profiles** with **Web Clips** pointing at the app's URL scheme (`spotify://`, `instagram://`…). One file installs all your icons at once.

| | Shortcuts trick | iconpush |
| --- | --- | --- |
| Banner on every launch | yes | no\* |
| 30 icons | 30 × manual setup | one click |
| Share your setup | ✗ | send the profile or the pack |
| Jailbreak | no | no |

\* depends on the iOS version — see [Compatibility](#compatibility).

## Features

- 📲 **USB push** (Windows app): detects your iPhone, lists the apps **actually installed on it**, and sends the profile directly.
- 🎨 **Icon packs**: Liquid Glass, Mono Noir, Paper, Pastel, Néon, Sunset… generated for 50+ apps.
- 🏷 **Official logos** in one click: downloads brand logos from [Simple Icons](https://simpleicons.org) (CC0) and unlocks 3 more packs (brand colors, OLED black, Liquid Glass).
- 🖼 **Your own images** for any app, or a **whole folder at once** (e.g. icons exported from Figma, matched to apps by file name).
- 📱 **Realistic iPhone preview** (Dynamic Island, dock…) that you can **export as a PNG** to share your setup.
- 🔌 **Driver helper**: detects Apple's USB driver and installs it for you (iTunes 64-bit from apple.com with signature check, or Apple Devices from the Microsoft Store).
- 📊 **iPhone tab** (read-only): battery level, battery health & charge cycles, storage, model, iOS version and installed-app count — nothing is modified on the device.
- 🙈 **Hide labels** for a clean, text-free home screen.
- 🌐 **Web version**: no install, downloads the `.mobileconfig` instead.
- 🦀 **Native Windows app** in Rust: a single ~10 MB `.exe`, no browser, no runtime to install.
- 🔒 **Local & private**: nothing is uploaded anywhere.

## Quick start

1. Download **[iconpush.exe](https://github.com/MattRvfl/iconpush/releases/latest)** and run it. No install needed.
2. If the status says **Apple driver missing**, open the **Pilotes** tab and install it from there.
3. Plug in your iPhone, unlock it and tap **Trust**.
4. Pick a pack and your apps, then click **Send**.
5. On the iPhone: **Settings → Profile Downloaded → Install**.

To remove everything: **Settings → General → VPN & Device Management → iconpush → Remove Profile**.

## Compatibility

| | |
| --- | --- |
| Windows | 10 / 11 (x64), with Apple Devices or iTunes |
| iPhone | iOS 14 or later |
| macOS / Linux | use the [web version](https://mattrvfl.github.io/iconpush/) for now |

Good to know:

- iOS doesn't let any tool **replace** an app's original icon without a jailbreak. iconpush **adds** new icons; hide the originals in the App Library.
- A profile pushed over USB still has to be accepted on the phone (*Profile Downloaded → Install*). Only supervised (company-managed) devices can skip that.
- iOS shows the profile as *Not Verified* because it isn't signed. That's expected.
- Each app entry in [`apps.json`](web/data/apps.json) has a `verified` flag: `false` means nobody has confirmed the URL scheme on a real device yet. Tested one? Open a PR!

## How it works

```
iconpush.exe (egui window)
  ├─ render.rs    draws each icon (resvg + tiny-skia), 180×180 PNG
  ├─ profile.rs   builds the .mobileconfig (one com.apple.webClip.managed per icon)
  └─ worker.rs ──usbmuxd (Apple Mobile Device Service)──▶ iPhone
                   ├─ lockdown            pairing / "Trust this computer"
                   ├─ installation_proxy  list of installed apps
                   └─ MCInstall           "InstallProfile" → Settings › Profile Downloaded
```

- Native UI with [egui](https://github.com/emilk/egui); USB work runs on a background thread so the window never freezes.
- Device protocols via [`idevice`](https://crates.io/crates/idevice) (pure Rust), plus a small `MCInstall` client ([`src/mcinstall.rs`](src/mcinstall.rs)).
- Apps, packs and glyphs live in [`web/`](web/) and are shared with the web version (plain HTML/JS, same rendering rules).

## Build from source

```bash
git clone https://github.com/MattRvfl/iconpush
cd iconpush
cargo build --release      # → target/release/iconpush.exe
```

## Contributing

The easiest and most useful contributions:

- **Test an app** and set `"verified": true` in [`web/data/apps.json`](web/data/apps.json).
- **Add an app** you use (URL scheme + bundle id).
- **Make a pack**: see [docs/PACKS.md](docs/PACKS.md).

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Credits

Glyphs from [Lucide](https://lucide.dev) (ISC license). Device protocols via [idevice](https://github.com/jkcoxson/idevice).
iconpush is not affiliated with Apple. App names belong to their respective owners; packs never include their logos.

## License

[MIT](LICENSE)
