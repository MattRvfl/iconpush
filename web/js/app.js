import { renderIcon } from './render.js';
import { buildMobileconfig, BLANK_LABEL } from './mobileconfig.js';

const REMOTE_PACKS = 'https://raw.githubusercontent.com/MattRvfl/iconpush/main/web/packs/index.json';
const PREVIEW_APPS = ['spotify', 'instagram', 'whatsapp', 'settings'];

const $ = (id) => document.getElementById(id);

// The desktop app opens this page with ?t=<secret>; its local API only answers when we echo it.
const TOKEN = (() => {
  const t = new URLSearchParams(location.search).get('t');
  try {
    if (t) sessionStorage.setItem('iconpush:token', t);
    return t || sessionStorage.getItem('iconpush:token') || '';
  } catch (e) {
    return t || '';
  }
})();
const apiHeaders = () => ({ 'X-Iconpush-Token': TOKEN });

const state = {
  usb: false, // true when served by the desktop app (USB features available)
  device: null,
  installed: null, // Set of bundle ids, or null when unknown
  apps: [],
  packs: [],
  pack: null,
  selected: new Set(['spotify', 'instagram', 'whatsapp', 'youtube', 'settings', 'photos', 'messages', 'notes']),
  custom: new Map(), // app id -> data URL chosen by the user
  hideLabels: false,
};

// ---------------------------------------------------------------- icon cache

const iconCache = new Map();
function iconFor(app, pack = state.pack) {
  const custom = state.custom.get(app.id);
  const key = `${pack.id}|${app.id}|${custom ? custom.length : 0}`;
  if (!iconCache.has(key)) iconCache.set(key, renderIcon(app, pack, custom));
  return iconCache.get(key);
}

// ---------------------------------------------------------------- data loading

async function fetchJson(url, timeoutMs = 4000) {
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    const headers = url.startsWith('api/') ? apiHeaders() : {};
    const res = await fetch(url, { signal: ctrl.signal, cache: 'no-cache', headers });
    if (!res.ok) throw new Error(`${res.status} ${url}`);
    return await res.json();
  } finally {
    clearTimeout(t);
  }
}

async function loadPacks() {
  // Newest community packs come from GitHub; fall back to the bundled copy offline.
  try {
    const remote = await fetchJson(REMOTE_PACKS);
    if (Array.isArray(remote.packs) && remote.packs.length) return remote.packs;
  } catch (e) { /* offline or not published yet */ }
  return (await fetchJson('packs/index.json')).packs;
}

// ---------------------------------------------------------------- USB (desktop app only)

async function detectUsb() {
  if (!TOKEN) {
    state.usb = false;
    return;
  }
  try {
    const status = await fetchJson('api/status', 2000);
    state.usb = status && status.app === 'iconpush';
  } catch (e) {
    state.usb = false;
  }
}

async function refreshDevice() {
  const el = $('device');
  if (!state.usb) {
    el.className = 'device';
    el.textContent = 'Version web : le profil sera téléchargé';
    return;
  }
  let status;
  try {
    status = await fetchJson('api/status', 4000);
  } catch (e) {
    el.className = 'device wait';
    el.textContent = 'Application iconpush fermée ?';
    return;
  }
  const dev = status.devices && status.devices[0];
  const changed = (dev && dev.udid) !== (state.device && state.device.udid);
  state.device = dev || null;
  state.error = dev ? null : (status.error || null);
  if (!dev) {
    el.className = 'device wait';
    el.textContent = status.error ? '⚠ iPhone non accessible' : 'Branche ton iPhone en USB…';
    el.title = status.error || '';
    state.installed = null;
  } else {
    el.className = 'device ok';
    el.textContent = `${dev.name} · iOS ${dev.iosVersion}`;
    el.title = `${dev.model} · ${dev.udid}`;
    if (changed) {
      try {
        const res = await fetchJson(`api/apps?udid=${encodeURIComponent(dev.udid)}`, 20000);
        state.installed = new Set(res.bundleIds);
      } catch (e) {
        state.installed = null;
      }
    }
  }
  renderGoButton();
  if (changed) {
    $('only-installed-wrap').hidden = !state.installed;
    renderApps();
    renderGoButton();
  }
}

// ---------------------------------------------------------------- rendering

function renderPacks() {
  const q = $('pack-search').value.trim().toLowerCase();
  const list = $('packs');
  list.innerHTML = '';
  const sample = PREVIEW_APPS.map((id) => state.apps.find((a) => a.id === id)).filter(Boolean);

  for (const pack of state.packs) {
    const hay = `${pack.name} ${pack.description} ${(pack.tags || []).join(' ')} ${pack.author}`.toLowerCase();
    if (q && !hay.includes(q)) continue;

    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'pack' + (pack === state.pack ? ' selected' : '');
    btn.innerHTML = `<div class="pack-icons"></div><div><strong></strong><span></span></div>`;
    btn.querySelector('strong').textContent = pack.name;
    btn.querySelector('span').textContent = `${pack.description} — par ${pack.author}`;
    const icons = btn.querySelector('.pack-icons');
    for (const app of sample) {
      const img = document.createElement('img');
      img.alt = '';
      iconFor(app, pack).then((src) => { img.src = src; }).catch(() => {});
      icons.appendChild(img);
    }
    btn.addEventListener('click', () => {
      state.pack = pack;
      renderPacks();
      renderApps();
      renderPhone();
    });
    list.appendChild(btn);
  }
  if (!list.children.length) list.innerHTML = '<p class="hint">Aucun pack ne correspond.</p>';
}

function isInstalled(app) {
  return !state.installed || state.installed.has(app.bundleId);
}

function visibleApps() {
  const q = $('app-search').value.trim().toLowerCase();
  const cat = $('category').value;
  const onlyInstalled = state.installed && $('only-installed').checked;
  return state.apps.filter((a) =>
    (!q || a.name.toLowerCase().includes(q) || a.id.includes(q)) &&
    (!cat || a.category === cat) &&
    (!onlyInstalled || isInstalled(a))
  );
}

function renderApps() {
  const list = $('apps');
  list.innerHTML = '';
  for (const app of visibleApps()) {
    const row = document.createElement('label');
    row.className = 'app' + (state.selected.has(app.id) ? ' on' : '');
    const note = state.installed && !isInstalled(app) ? 'pas sur ton iPhone' : (app.verified ? '✓ testé' : '');
    row.innerHTML = `<input type="checkbox"><img alt=""><span class="name"><span></span><small></small></span><button type="button" class="custom" title="Mettre ma propre image">🖼</button>`;
    row.querySelector('input').checked = state.selected.has(app.id);
    row.querySelector('.name span').textContent = app.name;
    row.querySelector('small').textContent = note;
    const img = row.querySelector('img');
    iconFor(app).then((src) => { img.src = src; }).catch(() => {});

    row.querySelector('input').addEventListener('change', (e) => {
      if (e.target.checked) state.selected.add(app.id);
      else state.selected.delete(app.id);
      row.classList.toggle('on', e.target.checked);
      renderPhone();
    });
    row.querySelector('.custom').addEventListener('click', (e) => {
      e.preventDefault();
      pickCustomImage(app);
    });
    list.appendChild(row);
  }
  if (!list.children.length) list.innerHTML = '<p class="hint">Aucune app ne correspond.</p>';
}

function selectedApps() {
  return state.apps.filter((a) => state.selected.has(a.id));
}

async function renderPhone() {
  const grid = $('phone-grid');
  grid.classList.toggle('no-labels', state.hideLabels);
  const apps = selectedApps();
  grid.innerHTML = '';
  for (const app of apps) {
    const fig = document.createElement('figure');
    fig.innerHTML = '<img alt=""><figcaption></figcaption>';
    fig.querySelector('figcaption').textContent = app.name;
    iconFor(app).then((src) => { fig.querySelector('img').src = src; }).catch(() => {});
    grid.appendChild(fig);
  }
  $('count').textContent = apps.length
    ? `${apps.length} icône${apps.length > 1 ? 's' : ''} · pack « ${state.pack.name} »`
    : 'Coche au moins une app.';
  renderGoButton();
}

function renderGoButton() {
  const go = $('go');
  const hint = $('go-hint');
  const n = state.selected.size;
  if (state.usb) {
    go.textContent = state.device ? `📲 Envoyer sur ${state.device.name}` : '📲 Branche ton iPhone';
    go.disabled = !state.device || !n;
    hint.textContent = state.error || 'Le profil arrive dans Réglages : il te restera à taper « Installer ».';
  } else {
    go.textContent = '⬇ Télécharger le profil';
    go.disabled = !n;
    hint.innerHTML = 'Pour l’envoyer directement en USB, utilise l’<a href="https://github.com/MattRvfl/iconpush/releases" target="_blank" rel="noopener">application Windows</a>.';
  }
}

// ---------------------------------------------------------------- custom images

let pickingFor = null;
function pickCustomImage(app) {
  pickingFor = app;
  $('file').value = '';
  $('file').click();
}

$('file').addEventListener('change', () => {
  const file = $('file').files[0];
  if (!file || !pickingFor) return;
  const reader = new FileReader();
  reader.onload = () => {
    state.custom.set(pickingFor.id, reader.result);
    state.selected.add(pickingFor.id);
    renderApps();
    renderPhone();
  };
  reader.readAsDataURL(file);
});

// ---------------------------------------------------------------- build + send

function dialog(title, html) {
  $('dialog-title').textContent = title;
  $('dialog-body').innerHTML = html;
  $('dialog').showModal();
}

async function buildProfile() {
  const clips = [];
  for (const app of selectedApps()) {
    clips.push({
      label: state.hideLabels ? BLANK_LABEL : app.name,
      url: app.url,
      png: await iconFor(app),
    });
  }
  return buildMobileconfig({ name: `iconpush — ${state.pack.name}`, clips });
}

const INSTALL_STEPS = `
  <ol>
    <li>Ouvre <b>Réglages</b> sur ton iPhone.</li>
    <li>Touche <b>Profil téléchargé</b> (tout en haut).</li>
    <li>Touche <b>Installer</b> et entre ton code. L’avertissement « non vérifié » est normal.</li>
    <li>Tes nouvelles icônes apparaissent sur l’écran d’accueil. Cache les apps d’origine dans la Bibliothèque d’apps si tu veux.</li>
  </ol>
  <p class="hint">Pour tout retirer : Réglages → Général → VPN et gestion de l’appareil → iconpush → Supprimer.</p>`;

$('go').addEventListener('click', async () => {
  const go = $('go');
  go.disabled = true;
  const label = go.textContent;
  go.textContent = 'Préparation…';
  try {
    const xml = await buildProfile();
    if (state.usb) {
      go.textContent = 'Envoi…';
      const res = await fetch(`api/push?udid=${encodeURIComponent(state.device.udid)}`, { method: 'POST', body: xml, headers: apiHeaders() });
      const out = await res.json().catch(() => ({}));
      if (!res.ok || !out.ok) throw new Error(out.error || `erreur ${res.status}`);
      dialog('✅ Envoyé sur ton iPhone', INSTALL_STEPS);
    } else {
      const a = document.createElement('a');
      a.href = URL.createObjectURL(new Blob([xml], { type: 'application/x-apple-aspen-config' }));
      a.download = `iconpush-${state.pack.id}.mobileconfig`;
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 5000);
      dialog('⬇ Profil téléchargé', `
        <p><b>Sur iPhone :</b> ouvre cette page dans <b>Safari</b>, puis touche « Autoriser ».</p>
        <p><b>Sur ordinateur :</b> envoie le fichier sur ton iPhone (AirDrop, iCloud Drive, mail) et ouvre-le depuis l’app Fichiers.</p>
        ${INSTALL_STEPS}`);
    }
  } catch (e) {
    dialog('❌ Ça n’a pas marché', `<p>${String(e.message || e).replace(/</g, '&lt;')}</p>
      <p class="hint">Vérifie que l’iPhone est déverrouillé et que tu as touché « Se fier à cet ordinateur ».</p>`);
  } finally {
    go.textContent = label;
    renderGoButton();
  }
});

// ---------------------------------------------------------------- wiring

$('pack-search').addEventListener('input', renderPacks);
$('app-search').addEventListener('input', renderApps);
$('category').addEventListener('change', renderApps);
$('only-installed').addEventListener('change', renderApps);
$('hide-labels').addEventListener('change', (e) => {
  state.hideLabels = e.target.checked;
  renderPhone();
});
$('select-all').addEventListener('click', () => {
  visibleApps().forEach((a) => state.selected.add(a.id));
  renderApps();
  renderPhone();
});
$('select-none').addEventListener('click', () => {
  visibleApps().forEach((a) => state.selected.delete(a.id));
  renderApps();
  renderPhone();
});

async function main() {
  const [apps, packs] = await Promise.all([fetchJson('data/apps.json'), loadPacks(), detectUsb()]);
  state.apps = apps.apps;
  state.packs = packs;
  state.pack = packs[0];
  renderPacks();
  renderApps();
  renderPhone();
  await refreshDevice();
  if (state.usb) setInterval(refreshDevice, 3000);
}

main().catch((e) => dialog('❌ Erreur de chargement', `<p>${String(e.message || e)}</p>`));
