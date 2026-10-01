# Making an icon pack

A pack is an entry in [`web/packs/index.json`](../web/packs/index.json). There are two kinds, and you can mix them.

## 1. Generated pack (no drawing needed)

You describe a **style**; iconpush draws an icon for every app using a symbol from [Lucide](https://lucide.dev).

```json
{
  "id": "ocean",
  "name": "Ocean",
  "author": "your-github-name",
  "description": "Deep blue gradient, white symbols.",
  "tags": ["blue", "gradient"],
  "style": {
    "background": { "type": "linear", "angle": 160, "stops": ["#0ea5e9", "#1e3a8a"] },
    "glass": false,
    "glyph": { "color": "#ffffff", "scale": 0.5, "stroke": 1.8 }
  }
}
```

| Field | Values |
| --- | --- |
| `background.type` | `solid` (+ `color`), `linear` (+ `angle`, `stops`), `category` (+ `colors` per category) |
| `glass` | `true` adds a frosted-glass highlight |
| `glyph.color` | a color, or `"category"` with a `colors` map |
| `glyph.scale` | symbol size, from `0.3` to `0.7` |
| `glyph.stroke` | line thickness, from `1` to `3` |
| `glyph.glow` | optional glow radius in px (`10`–`20` looks good) |

Categories: `music`, `social`, `messaging`, `video`, `tools`, `travel`, `system`.

## 2. Image pack (your own artwork)

Put square PNG files (at least 180×180) in `web/packs/<id>/` and map app ids to them:

```json
{
  "id": "pixel",
  "name": "Pixel Art",
  "author": "your-github-name",
  "description": "Hand-made 8-bit icons.",
  "tags": ["pixel", "retro"],
  "style": {
    "background": { "type": "solid", "color": "#222222" },
    "glyph": { "color": "#ffffff", "scale": 0.5, "stroke": 2 }
  },
  "images": {
    "spotify": "spotify.png",
    "instagram": "instagram.png"
  }
}
```

Apps without an image fall back to the generated `style`. App ids are listed in [`web/data/apps.json`](../web/data/apps.json).

## Rules

- **Only artwork you own or that is openly licensed.** No company logos, even redrawn: they're trademarks.
- Keep PNGs under 100 KB each.
- Test your pack in the web version (open `web/index.html` through a local server, or the desktop app) before opening a pull request.

Once merged, your pack shows up in everyone's iconpush automatically: the app fetches the pack list from this repository.
