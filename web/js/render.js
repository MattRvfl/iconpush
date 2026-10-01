// Draws one 180×180 icon for an app in a pack style, on a canvas. Returns a PNG data URL.
// Shared by the web version and the desktop app (which serves this same file).

const SIZE = 180;
const svgCache = new Map();

async function loadGlyph(name) {
  if (!svgCache.has(name)) {
    svgCache.set(name, fetch(`glyphs/${name}.svg`).then((r) => {
      if (!r.ok) throw new Error(`glyph ${name} introuvable`);
      return r.text();
    }));
  }
  return svgCache.get(name);
}

function loadImage(src) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`image illisible : ${src.slice(0, 60)}`));
    img.src = src;
  });
}

function pick(value, colors, app) {
  return value === 'category' ? (colors[app.category] || '#888888') : value;
}

function paintBackground(ctx, bg, app) {
  if (bg.type === 'linear') {
    const a = ((bg.angle || 180) - 90) * (Math.PI / 180);
    const r = SIZE / 2;
    const g = ctx.createLinearGradient(
      r - Math.cos(a) * r, r - Math.sin(a) * r,
      r + Math.cos(a) * r, r + Math.sin(a) * r
    );
    bg.stops.forEach((c, i) => g.addColorStop(i / (bg.stops.length - 1), c));
    ctx.fillStyle = g;
  } else if (bg.type === 'category') {
    ctx.fillStyle = bg.colors[app.category] || '#cccccc';
  } else {
    ctx.fillStyle = bg.color;
  }
  ctx.fillRect(0, 0, SIZE, SIZE);
}

function paintGlass(ctx) {
  // Soft top highlight and a faint inner rim, like frosted glass.
  const hi = ctx.createLinearGradient(0, 0, 0, SIZE * 0.6);
  hi.addColorStop(0, 'rgba(255,255,255,0.28)');
  hi.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = hi;
  ctx.fillRect(0, 0, SIZE, SIZE * 0.6);

  const glow = ctx.createRadialGradient(SIZE * 0.3, SIZE * 0.2, 0, SIZE * 0.3, SIZE * 0.2, SIZE * 0.7);
  glow.addColorStop(0, 'rgba(160,190,255,0.25)');
  glow.addColorStop(1, 'rgba(160,190,255,0)');
  ctx.fillStyle = glow;
  ctx.fillRect(0, 0, SIZE, SIZE);

  ctx.strokeStyle = 'rgba(255,255,255,0.18)';
  ctx.lineWidth = 3;
  ctx.strokeRect(1.5, 1.5, SIZE - 3, SIZE - 3);
}

async function paintGlyph(ctx, glyph, app) {
  const color = pick(glyph.color, glyph.colors || {}, app);
  let svg = await loadGlyph(app.glyph);
  svg = svg
    .replace(/stroke="currentColor"/, `stroke="${color}"`)
    .replace(/stroke-width="[\d.]+"/, `stroke-width="${glyph.stroke || 2}"`);
  const url = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }));
  try {
    const img = await loadImage(url);
    const s = SIZE * (glyph.scale || 0.5);
    const o = (SIZE - s) / 2;
    if (glyph.glow) {
      ctx.shadowColor = color;
      ctx.shadowBlur = glyph.glow;
    }
    ctx.drawImage(img, o, o, s, s);
    if (glyph.glow) ctx.drawImage(img, o, o, s, s); // second pass = stronger glow
    ctx.shadowBlur = 0;
  } finally {
    URL.revokeObjectURL(url);
  }
}

/** Render an icon. `custom` (optional) is an image URL that replaces the generated icon. */
export async function renderIcon(app, pack, custom) {
  const canvas = document.createElement('canvas');
  canvas.width = SIZE;
  canvas.height = SIZE;
  const ctx = canvas.getContext('2d');

  const image = custom || (pack.images && pack.images[app.id] && `packs/${pack.id}/${pack.images[app.id]}`);
  if (image) {
    const img = await loadImage(image);
    // Cover-fit the image into the square.
    const k = Math.max(SIZE / img.width, SIZE / img.height);
    const w = img.width * k;
    const h = img.height * k;
    ctx.drawImage(img, (SIZE - w) / 2, (SIZE - h) / 2, w, h);
  } else {
    const st = pack.style;
    paintBackground(ctx, st.background, app);
    if (st.glass) paintGlass(ctx);
    await paintGlyph(ctx, st.glyph, app);
  }
  return canvas.toDataURL('image/png');
}
