//! iconpush — native desktop app: pick an icon pack and apps, preview the home screen,
//! and push the profile to the iPhone over USB.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // no console window in release

mod chrome;
mod data;
mod device;
mod drivers;
mod library;
mod mcinstall;
mod phone;
mod phoneinfo;
mod profile;
mod render;
mod tabs;
mod theme;
mod worker;

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use eframe::egui::{
    self, Align, Color32, ColorImage, CornerRadius, Frame, Layout, Margin, RichText, Sense, Stroke, TextureHandle,
    TextureOptions, Vec2, ViewportCommand,
};
use image::RgbaImage;

use chrome::{Status, Tab};
use data::{App, Pack};
use device::DeviceInfo;
use drivers::InstallState;
use library::Logos;
use tabs::{LibraryAction, LogoState};
use theme::{card_frame, step_title, ACCENT, BG, BORDER, MUTED, OK, PANEL, WARN};
use worker::{Command, Event};

enum Dialog {
    Sent,
    Saved(String),
    Error(String),
}

struct IconPush {
    tab: Tab,
    apps: Vec<App>,
    packs: Vec<Pack>,
    pack: usize,
    selected: HashSet<String>,
    /// App id → (version, image) chosen by the user.
    custom: HashMap<String, (u64, RgbaImage)>,
    custom_version: u64,
    textures: HashMap<String, TextureHandle>,

    logos: Logos,
    logos_version: u64,
    logo_state: Arc<Mutex<LogoState>>,
    import_msg: Option<String>,

    driver_ok: bool,
    driver_checked: f64,
    install_state: Arc<Mutex<InstallState>>,
    store_error: Option<String>,

    pack_search: String,
    app_search: String,
    category: Option<&'static str>,
    only_installed: bool,
    hide_labels: bool,
    export_rect: Option<egui::Rect>,

    phone_info: Option<phoneinfo::PhoneInfo>,
    info_loading: bool,
    info_requested: f64,
    show_serial: bool,

    devices: Vec<DeviceInfo>,
    device_error: Option<String>,
    installed: Option<(String, HashSet<String>)>,
    pushing: bool,
    dialog: Option<Dialog>,

    tx: Sender<Command>,
    rx: Receiver<Event>,

    /// Dev helper: ICONPUSH_SCREENSHOT=out.png saves a screenshot of the window, then quits.
    auto_screenshot: Option<String>,
    /// Dev helper: ICONPUSH_PACK=<id> selects that pack as soon as it exists.
    pending_pack: Option<String>,
}

impl IconPush {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::setup(&cc.egui_ctx);
        let (tx, rx) = worker::spawn(cc.egui_ctx.clone());
        let mut app = Self {
            tab: Tab::Icons,
            apps: data::apps(),
            packs: Vec::new(),
            pack: 0,
            selected: ["spotify", "instagram", "whatsapp", "youtube", "settings", "photos", "messages", "notes"]
                .into_iter()
                .map(String::from)
                .collect(),
            custom: HashMap::new(),
            custom_version: 0,
            textures: HashMap::new(),
            logos: library::load_logos(),
            logos_version: 0,
            logo_state: Arc::new(Mutex::new(LogoState::Idle)),
            import_msg: None,
            driver_ok: drivers::driver_running(),
            driver_checked: 0.0,
            install_state: Arc::new(Mutex::new(InstallState::Idle)),
            store_error: None,
            pack_search: String::new(),
            app_search: String::new(),
            category: None,
            only_installed: true,
            hide_labels: false,
            export_rect: None,
            phone_info: None,
            info_loading: false,
            info_requested: f64::NEG_INFINITY,
            show_serial: false,
            devices: Vec::new(),
            device_error: None,
            installed: None,
            pushing: false,
            dialog: None,
            tx,
            rx,
            auto_screenshot: std::env::var("ICONPUSH_SCREENSHOT").ok(),
            pending_pack: None,
        };
        app.tab = match std::env::var("ICONPUSH_TAB").as_deref() {
            Ok("library") => Tab::Library,
            Ok("drivers") => Tab::Drivers,
            Ok("icons") => Tab::Icons,
            _ => Tab::Phone,
        };
        app.rebuild_packs();
        if std::env::var_os("ICONPUSH_DOWNLOAD_LOGOS").is_some() {
            app.start_logo_download(&cc.egui_ctx); // dev helper, used with ICONPUSH_SCREENSHOT
        }
        if let Ok(id) = std::env::var("ICONPUSH_PACK") {
            app.pending_pack = Some(id);
        }
        app
    }

    /// ICONPUSH_SCREENSHOT: request a screenshot after a few seconds, save it, quit.
    fn auto_screenshot(&mut self, ctx: &egui::Context) {
        let Some(out) = self.auto_screenshot.clone() else { return };
        if let Some(id) = self.pending_pack.clone() {
            if self.packs.iter().any(|p| p.id == id) {
                self.select_pack(&id);
                self.pending_pack = None;
            }
        }
        // Wait for a logo download to finish before taking the picture.
        if matches!(*self.logo_state.lock().unwrap(), LogoState::Running { .. }) || self.pending_pack.is_some() {
            if ctx.input(|i| i.time) < 90.0 {
                ctx.request_repaint();
                return;
            }
        }
        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, user_data, .. } if user_data.data.is_some() => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = shot {
            let [w, h] = image.size;
            let rgba: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
            if let Some(img) = RgbaImage::from_raw(w as u32, h as u32, rgba) {
                let _ = img.save(&out);
            }
            ctx.send_viewport_cmd(ViewportCommand::Close);
        } else if ctx.input(|i| i.time) > 4.0 && ctx.cumulative_frame_nr() % 60 == 0 {
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::new("auto")));
        }
        ctx.request_repaint();
    }

    /// Built-in packs, plus the logo packs once the logos are downloaded.
    fn rebuild_packs(&mut self) {
        let current = self.packs.get(self.pack).map(|p| p.id.clone());
        self.packs = data::packs();
        if !self.logos.is_empty() {
            self.packs.extend(data::logo_packs());
        }
        self.pack = current.and_then(|id| self.packs.iter().position(|p| p.id == id)).unwrap_or(0);
    }

    fn select_pack(&mut self, id: &str) {
        if let Some(i) = self.packs.iter().position(|p| p.id == id) {
            self.pack = i;
        }
    }

    fn device(&self) -> Option<&DeviceInfo> {
        self.devices.first()
    }

    fn installed_set(&self) -> Option<&HashSet<String>> {
        let dev = self.device()?;
        self.installed.as_ref().filter(|(udid, _)| *udid == dev.udid).map(|(_, set)| set)
    }

    // -------------------------------------------------------------- background events

    fn handle_events(&mut self, ctx: &egui::Context) {
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                Event::Devices(Ok(devices)) => {
                    let new_udid = devices.first().map(|d| d.udid.clone());
                    let had = self.device().map(|d| d.udid.clone());
                    self.devices = devices;
                    self.device_error = None;
                    if let Some(udid) = new_udid {
                        if had.as_ref() != Some(&udid) || self.installed.is_none() {
                            let _ = self.tx.send(Command::LoadApps(udid));
                        }
                    }
                }
                Event::Devices(Err(e)) => {
                    self.devices.clear();
                    self.device_error = Some(e);
                    self.phone_info = None;
                }
                Event::Info(result) => {
                    self.info_loading = false;
                    match result {
                        Ok(info) => self.phone_info = Some(info),
                        Err(e) => self.device_error = Some(e),
                    }
                }
                Event::Apps { udid, result } => {
                    if let Ok(set) = result {
                        self.installed = Some((udid, set));
                    }
                }
                Event::Pushed(result) => {
                    self.pushing = false;
                    self.dialog = Some(match result {
                        Ok(()) => Dialog::Sent,
                        Err(e) => Dialog::Error(e),
                    });
                }
            }
        }

        // Logos finished downloading → reload them and add the logo packs.
        let finished = matches!(*self.logo_state.lock().unwrap(), LogoState::Done(_));
        if finished && self.logos.svgs.is_empty() || finished && self.logos_version == 0 {
            self.logos = library::load_logos();
            self.logos_version += 1;
            self.textures.clear();
            self.rebuild_packs();
        }

        // Re-check the Apple driver every few seconds (cheap local TCP probe).
        let now = ctx.input(|i| i.time);
        if now - self.driver_checked > 3.0 {
            self.driver_checked = now;
            self.driver_ok = drivers::driver_running();
            ctx.request_repaint_after(std::time::Duration::from_secs(3));
        }

        // Screenshot requested by "Exporter l'aperçu".
        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, user_data, .. } if user_data.data.is_none() => Some(image.clone()),
                _ => None,
            })
        });
        if let (Some(image), Some(rect)) = (shot, self.export_rect.take()) {
            self.save_preview(&image.region(&rect, Some(ctx.pixels_per_point())));
        }
    }

    // -------------------------------------------------------------- icons

    /// Texture for an app icon in a pack (rendered on first use, then cached).
    fn icon(&mut self, ctx: &egui::Context, pack: usize, app: usize) -> TextureHandle {
        let app_ref = &self.apps[app];
        let custom = self.custom.get(&app_ref.id);
        let key = format!("{}|{}|{}|{}", self.packs[pack].id, app_ref.id, custom.map_or(0, |c| c.0), self.logos_version);
        if let Some(t) = self.textures.get(&key) {
            return t.clone();
        }
        let img = render::render_icon(app_ref, &self.packs[pack], custom.map(|c| &c.1), &self.logos);
        let color = ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
        let tex = ctx.load_texture(&key, color, TextureOptions::LINEAR);
        self.textures.insert(key, tex.clone());
        tex
    }

    fn visible_apps(&self) -> Vec<usize> {
        let q = self.app_search.trim().to_lowercase();
        let installed = if self.only_installed { self.installed_set() } else { None };
        (0..self.apps.len())
            .filter(|&i| {
                let a = &self.apps[i];
                (q.is_empty() || a.name.to_lowercase().contains(&q) || a.id.contains(&q))
                    && self.category.is_none_or(|c| a.category == c)
                    && installed.is_none_or(|set| set.contains(&a.bundle_id))
            })
            .collect()
    }

    fn selected_apps(&self) -> Vec<usize> {
        (0..self.apps.len()).filter(|&i| self.selected.contains(&self.apps[i].id)).collect()
    }

    fn build_profile(&self) -> Vec<u8> {
        let pack = &self.packs[self.pack];
        let clips = self
            .selected_apps()
            .into_iter()
            .map(|i| {
                let app = &self.apps[i];
                let img = render::render_icon(app, pack, self.custom.get(&app.id).map(|c| &c.1), &self.logos);
                profile::Clip {
                    label: if self.hide_labels { profile::BLANK_LABEL.into() } else { app.name.clone() },
                    url: app.url.clone(),
                    png: render::png(&img),
                }
            })
            .collect();
        profile::build(&format!("iconpush — {}", pack.name), clips)
    }

    fn set_custom(&mut self, app_id: &str, img: &image::DynamicImage) {
        self.custom_version += 1;
        self.custom.insert(app_id.to_string(), (self.custom_version, render::cover(img)));
        self.selected.insert(app_id.to_string());
    }

    fn pick_custom_image(&mut self, app_id: &str) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Choisis une image pour l'icône")
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            return;
        };
        match image::open(&path) {
            Ok(img) => self.set_custom(app_id, &img),
            Err(e) => self.dialog = Some(Dialog::Error(format!("Image illisible : {e}"))),
        }
    }

    fn import_folder(&mut self) {
        let Some(dir) = rfd::FileDialog::new().set_title("Dossier contenant tes icônes").pick_folder() else { return };
        let matches = library::match_folder(&dir, &self.apps);
        let mut names = Vec::new();
        let mut failed = 0;
        for (app_id, path) in &matches {
            match image::open(path) {
                Ok(img) => {
                    self.set_custom(app_id, &img);
                    if let Some(app) = self.apps.iter().find(|a| &a.id == app_id) {
                        names.push(app.name.clone());
                    }
                }
                Err(_) => failed += 1,
            }
        }
        self.import_msg = Some(if names.is_empty() {
            "Aucune image reconnue : nomme tes fichiers comme les apps (spotify.png, instagram.png…).".into()
        } else {
            format!(
                "✓ {} icône{} importée{} : {}{}",
                names.len(),
                if names.len() > 1 { "s" } else { "" },
                if names.len() > 1 { "s" } else { "" },
                names.join(", "),
                if failed > 0 { format!(" ({failed} fichier(s) illisible(s))") } else { String::new() }
            )
        });
    }

    fn start_logo_download(&mut self, ctx: &egui::Context) {
        let state = self.logo_state.clone();
        let apps = self.apps.clone();
        let ctx = ctx.clone();
        *state.lock().unwrap() = LogoState::Running { done: 0, total: 0 };
        self.logos_version = 0; // reload when done
        std::thread::spawn(move || {
            let result = library::download_logos(&apps, |done, total| {
                *state.lock().unwrap() = LogoState::Running { done, total };
                ctx.request_repaint();
            });
            *state.lock().unwrap() = match result {
                Ok(l) => LogoState::Done(l.svgs.len()),
                Err(e) => LogoState::Error(e),
            };
            ctx.request_repaint();
        });
    }

    fn save_preview(&mut self, image: &ColorImage) {
        let [w, h] = image.size;
        let rgba: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
        let Some(img) = RgbaImage::from_raw(w as u32, h as u32, rgba) else { return };
        let dir = library::data_dir().join("exports");
        let _ = std::fs::create_dir_all(&dir);
        let Some(path) = rfd::FileDialog::new()
            .set_directory(&dir)
            .set_file_name(format!("iconpush-{}.png", self.packs[self.pack].id))
            .add_filter("Image PNG", &["png"])
            .save_file()
        else {
            return;
        };
        self.dialog = Some(match img.save(&path) {
            Ok(()) => Dialog::Saved(path.display().to_string()),
            Err(e) => Dialog::Error(format!("Export impossible : {e}")),
        });
    }

    fn status(&self) -> Status {
        match (self.device(), &self.device_error) {
            (Some(d), _) => Status {
                dot: OK,
                text: format!("{} · iOS {}", d.name, d.ios_version),
                tooltip: format!("{} · {}", d.model, d.udid),
                link: None,
            },
            (None, _) if !self.driver_ok => Status {
                dot: WARN,
                text: "Pilote Apple manquant".into(),
                tooltip: "Clique pour l'installer".into(),
                link: Some(Tab::Drivers),
            },
            (None, Some(e)) => Status { dot: WARN, text: "iPhone non accessible".into(), tooltip: e.clone(), link: None },
            (None, None) => Status { dot: WARN, text: "Branche ton iPhone en USB".into(), tooltip: String::new(), link: None },
        }
    }

    // -------------------------------------------------------------- Icônes tab

    fn packs_panel(&mut self, ui: &mut egui::Ui) {
        step_title(ui, "1", "Choisis un pack");
        ui.add(egui::TextEdit::singleline(&mut self.pack_search).hint_text("Rechercher (glass, logos, néon…)").desired_width(f32::INFINITY));
        ui.add_space(6.0);

        let sample: Vec<usize> = ["spotify", "instagram", "whatsapp", "youtube"]
            .iter()
            .filter_map(|id| self.apps.iter().position(|a| a.id == *id))
            .collect();
        let q = self.pack_search.trim().to_lowercase();

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let mut any = false;
            for p in 0..self.packs.len() {
                let pack = &self.packs[p];
                let hay = format!("{} {} {} {}", pack.name, pack.description, pack.tags.join(" "), pack.author).to_lowercase();
                if !q.is_empty() && !hay.contains(&q) {
                    continue;
                }
                any = true;
                let (name, desc) = (pack.name.clone(), format!("{} — {}", pack.description, pack.author));
                let icons: Vec<TextureHandle> = sample.iter().map(|&a| self.icon(ui.ctx(), p, a)).collect();

                let resp = card_frame(p == self.pack)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            egui::Grid::new(("pack-icons", p)).spacing(Vec2::splat(4.0)).show(ui, |ui| {
                                for (i, t) in icons.iter().enumerate() {
                                    ui.add(egui::Image::new(t).fit_to_exact_size(Vec2::splat(26.0)).corner_radius(7));
                                    if i % 2 == 1 {
                                        ui.end_row();
                                    }
                                }
                            });
                            ui.vertical(|ui| {
                                ui.label(RichText::new(name).font(theme::semibold(15.0)));
                                ui.label(RichText::new(desc).small().color(MUTED));
                            });
                        });
                    })
                    .response
                    .interact(Sense::click());
                if resp.clicked() {
                    self.pack = p;
                }
                resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            }
            if !any {
                ui.label(RichText::new("Aucun pack ne correspond.").color(MUTED));
            }
            if self.logos.is_empty() {
                ui.add_space(6.0);
                if ui.link("+ Débloquer les packs de logos officiels").clicked() {
                    self.tab = Tab::Library;
                }
            }
        });
    }

    fn apps_panel(&mut self, ui: &mut egui::Ui) {
        step_title(ui, "2", "Choisis tes apps");
        ui.horizontal(|ui| {
            // Right-to-left so the category menu always keeps its room.
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let label = self
                    .category
                    .and_then(|c| data::CATEGORIES.iter().find(|(id, _)| *id == c))
                    .map_or("Toutes les catégories", |(_, l)| *l);
                egui::ComboBox::from_id_salt("category").selected_text(label).width(170.0).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.category, None, "Toutes les catégories");
                    for (id, l) in data::CATEGORIES {
                        ui.selectable_value(&mut self.category, Some(*id), *l);
                    }
                });
                ui.add(egui::TextEdit::singleline(&mut self.app_search).hint_text("Rechercher une app").desired_width(ui.available_width()));
            });
        });

        ui.horizontal(|ui| {
            if self.installed_set().is_some() {
                ui.checkbox(&mut self.only_installed, "Seulement les apps de mon iPhone");
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.link("Tout décocher").clicked() {
                    for i in self.visible_apps() {
                        self.selected.remove(&self.apps[i].id);
                    }
                }
                if ui.link("Tout cocher").clicked() {
                    for i in self.visible_apps() {
                        self.selected.insert(self.apps[i].id.clone());
                    }
                }
            });
        });
        ui.add_space(4.0);

        let visible = self.visible_apps();
        let installed = self.installed_set().cloned();
        let mut pick_for: Option<String> = None;
        let mut clear_custom: Option<String> = None;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let gap = ui.spacing().item_spacing.x;
            let cols = ((ui.available_width() + gap) / (190.0 + gap)).floor().max(1.0) as usize;
            let card_w = (ui.available_width() - gap * (cols as f32 - 1.0)) / cols as f32;
            for row in visible.chunks(cols) {
                ui.horizontal_top(|ui| {
                    for &i in row {
                        let tex = self.icon(ui.ctx(), self.pack, i);
                        let app = &self.apps[i];
                        let on = self.selected.contains(&app.id);
                        let has_custom = self.custom.contains_key(&app.id);
                        let note = match &installed {
                            Some(set) if !set.contains(&app.bundle_id) => "pas sur ton iPhone",
                            _ if has_custom => "ton image",
                            _ if app.verified => "✓ testé",
                            _ => "",
                        };
                        let (id, name) = (app.id.clone(), app.name.clone());

                        let resp = card_frame(on)
                            .show(ui, |ui| {
                                ui.set_width(card_w - 18.0);
                                ui.horizontal(|ui| {
                                    ui.set_min_height(38.0);
                                    ui.add(egui::Image::new(&tex).fit_to_exact_size(Vec2::splat(36.0)).corner_radius(9));
                                    ui.vertical(|ui| {
                                        ui.add(egui::Label::new(RichText::new(&name)).truncate());
                                        if !note.is_empty() {
                                            ui.label(RichText::new(note).small().color(MUTED));
                                        }
                                    });
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if has_custom {
                                            if ui.small_button("✕").on_hover_text("Revenir à l'icône du pack").clicked() {
                                                clear_custom = Some(id.clone());
                                            }
                                        } else if ui.small_button("Image").on_hover_text("Mettre ta propre image").clicked() {
                                            pick_for = Some(id.clone());
                                        }
                                    });
                                });
                            })
                            .response
                            .interact(Sense::click());
                        if resp.clicked() {
                            if on {
                                self.selected.remove(&id);
                            } else {
                                self.selected.insert(id);
                            }
                        }
                    }
                });
            }
            if visible.is_empty() {
                ui.label(RichText::new("Aucune app ne correspond.").color(MUTED));
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new("« ✓ testé » = vérifié sur un vrai iPhone. Pour les autres, dis-nous si ça marche sur GitHub.")
                    .small()
                    .color(MUTED),
            );
        });

        if let Some(id) = clear_custom {
            self.custom.remove(&id);
        }
        if let Some(id) = pick_for {
            self.pick_custom_image(&id);
        }
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        step_title(ui, "3", "Aperçu et envoi");

        let apps = self.selected_apps();
        let textures: Vec<TextureHandle> = apps.iter().map(|&i| self.icon(ui.ctx(), self.pack, i)).collect();
        let icons: Vec<phone::Icon> = apps
            .iter()
            .zip(textures)
            .map(|(&i, texture)| phone::Icon { texture, name: &self.apps[i].name })
            .collect();

        let controls = 240.0;
        let height = (ui.available_height() - controls).max(260.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
        let phone_rect = phone::draw(ui, rect, &icons, self.hide_labels);
        drop(icons);

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.hide_labels, "Masquer les noms");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Exporter l'aperçu").on_hover_text("Enregistre l'aperçu en image PNG, pour le partager").clicked() {
                    self.export_rect = Some(phone_rect.expand(6.0));
                    ui.ctx().send_viewport_cmd(ViewportCommand::Screenshot(Default::default()));
                }
            });
        });
        let n = apps.len();
        ui.label(
            RichText::new(if n == 0 {
                "Coche au moins une app.".to_string()
            } else {
                format!("{n} icône{} · pack « {} » · les 4 premières vont dans le dock", if n > 1 { "s" } else { "" }, self.packs[self.pack].name)
            })
            .small()
            .color(MUTED),
        );
        ui.add_space(4.0);

        let device = self.device().cloned();
        let label = match (&device, self.pushing) {
            (_, true) => "Envoi…".to_string(),
            (Some(d), false) => format!("Envoyer sur {}", d.name),
            (None, false) => "Branche ton iPhone".to_string(),
        };
        let send = egui::Button::new(RichText::new(label).font(theme::semibold(16.0)).color(Color32::WHITE))
            .fill(ACCENT)
            .corner_radius(CornerRadius::same(12))
            .min_size(Vec2::new(ui.available_width(), 44.0));
        if ui.add_enabled(device.is_some() && n > 0 && !self.pushing, send).clicked() {
            if let Some(d) = device {
                self.pushing = true;
                let _ = self.tx.send(Command::Push { udid: d.udid, profile: self.build_profile() });
            }
        }

        let save = egui::Button::new("Enregistrer le fichier .mobileconfig…").min_size(Vec2::new(ui.available_width(), 32.0));
        if ui.add_enabled(n > 0, save).on_hover_text("Pour l'envoyer autrement (AirDrop, mail, iCloud Drive)").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(format!("iconpush-{}.mobileconfig", self.packs[self.pack].id))
                .add_filter("Profil iOS", &["mobileconfig"])
                .save_file()
            {
                self.dialog = Some(match std::fs::write(&path, self.build_profile()) {
                    Ok(()) => Dialog::Saved(path.display().to_string()),
                    Err(e) => Dialog::Error(format!("Enregistrement impossible : {e}")),
                });
            }
        }

        if !self.driver_ok {
            if ui.link(RichText::new("⚠ Pilote Apple manquant : l'installer").color(WARN)).clicked() {
                self.tab = Tab::Drivers;
            }
        } else if let Some(e) = &self.device_error {
            ui.label(RichText::new(e).small().color(WARN));
        } else {
            ui.label(RichText::new("Le profil arrive dans Réglages : il te restera à toucher « Installer ».").small().color(MUTED));
        }
    }

    fn dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = &self.dialog else { return };
        let (title, body): (&str, String) = match dialog {
            Dialog::Sent => ("Envoyé sur ton iPhone", String::new()),
            Dialog::Saved(path) => ("Fichier enregistré", format!("{path}\n")),
            Dialog::Error(e) => ("Ça n'a pas marché", format!("{e}\n\nVérifie que l'iPhone est déverrouillé et que tu as touché « Se fier à cet ordinateur ».")),
        };
        let show_steps = matches!(dialog, Dialog::Sent) || matches!(dialog, Dialog::Saved(p) if p.ends_with(".mobileconfig"));
        let mut close = false;
        let resp = egui::Modal::new(egui::Id::new("dialog")).show(ctx, |ui| {
            ui.set_width(440.0);
            ui.label(RichText::new(title).font(theme::semibold(18.0)));
            ui.add_space(6.0);
            if !body.is_empty() {
                ui.label(body);
            }
            if show_steps {
                ui.label("Sur ton iPhone :");
                for (i, s) in [
                    "Ouvre Réglages.",
                    "Touche « Profil téléchargé » (tout en haut).",
                    "Touche « Installer » et entre ton code. « Non vérifié » est normal.",
                    "Tes icônes apparaissent sur l'écran d'accueil.",
                ]
                .iter()
                .enumerate()
                {
                    ui.label(format!("{}. {s}", i + 1));
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Pour tout retirer : Réglages → Général → VPN et gestion de l'appareil → iconpush → Supprimer.")
                        .small()
                        .color(MUTED),
                );
            }
            ui.add_space(8.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(theme::primary_button("OK")).clicked() {
                    close = true;
                }
            });
        });
        if close || resp.should_close() {
            self.dialog = None;
        }
    }
}

impl eframe::App for IconPush {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let window = ui.max_rect();
        self.handle_events(&ctx);

        let status = self.status();
        chrome::title_bar(ui, &mut self.tab, &status);

        let side = Frame::new().fill(PANEL).inner_margin(Margin::same(14)).stroke(Stroke::new(1.0, BORDER));
        let central = Frame::new().fill(BG).inner_margin(Margin::same(14));
        match self.tab {
            Tab::Phone => {
                // Refresh the phone info every 20 s while this tab is open (battery level changes).
                let now = ctx.input(|i| i.time);
                let udid = self.device().map(|d| d.udid.clone());
                if let Some(udid) = &udid {
                    let stale = self.phone_info.as_ref().is_none_or(|i| i.name != self.device().map(|d| d.name.clone()).unwrap_or_default());
                    if !self.info_loading && (stale || now - self.info_requested > 20.0) {
                        self.info_loading = true;
                        self.info_requested = now;
                        let _ = self.tx.send(Command::LoadInfo(udid.clone()));
                    }
                }
                let refresh = egui::CentralPanel::default()
                    .frame(central.inner_margin(Margin::same(28)))
                    .show(ui, |ui| {
                        tabs::phone(
                            ui,
                            self.phone_info.as_ref(),
                            self.info_loading,
                            udid.is_some(),
                            self.device_error.as_deref(),
                            &mut self.show_serial,
                        )
                    })
                    .inner;
                if refresh {
                    self.info_requested = f64::NEG_INFINITY;
                }
                ctx.request_repaint_after(std::time::Duration::from_secs(5));
            }
            Tab::Icons => {
                egui::Panel::left("packs").frame(side).resizable(false).exact_size(290.0).show(ui, |ui| self.packs_panel(ui));
                egui::Panel::right("preview").frame(side).resizable(false).exact_size(340.0).show(ui, |ui| self.preview_panel(ui));
                egui::CentralPanel::default().frame(central).show(ui, |ui| self.apps_panel(ui));
            }
            Tab::Library => {
                let state = self.logo_state.lock().unwrap().clone();
                let logo_apps = self.apps.iter().filter(|a| a.logo.is_some()).count();
                let action = egui::CentralPanel::default()
                    .frame(central.inner_margin(Margin::same(28)))
                    .show(ui, |ui| tabs::library(ui, self.logos.svgs.len(), logo_apps, &state, self.import_msg.as_deref()))
                    .inner;
                match action {
                    LibraryAction::DownloadLogos => self.start_logo_download(&ctx),
                    LibraryAction::ImportFolder => self.import_folder(),
                    LibraryAction::ShowLogoPacks => {
                        self.select_pack("logos-color");
                        self.tab = Tab::Icons;
                    }
                    LibraryAction::None => {}
                }
            }
            Tab::Drivers => {
                egui::CentralPanel::default()
                    .frame(central.inner_margin(Margin::same(28)))
                    .show(ui, |ui| tabs::drivers(ui, self.driver_ok, &self.install_state, &mut self.store_error));
            }
        }

        self.dialog(&ctx);
        chrome::edges(&ctx, window);
        self.auto_screenshot(&ctx);
    }
}

fn app_icon() -> egui::IconData {
    let size = 64u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let squares = [(10, 10, [255, 255, 255]), (34, 10, [124, 140, 255]), (10, 34, [124, 140, 255]), (34, 34, [255, 255, 255])];
    for (x0, y0, c) in squares {
        for y in y0..y0 + 20 {
            for x in x0..x0 + 20 {
                let i = ((y * size + x) * 4) as usize;
                rgba[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
    }
    egui::IconData { rgba, width: size, height: size }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("iconpush")
            .with_decorations(false) // our own title bar (see chrome.rs)
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1000.0, 620.0])
            .with_maximized(true)
            .with_icon(Arc::new(app_icon())),
        ..Default::default()
    };
    eframe::run_native("iconpush", options, Box::new(|cc| Ok(Box::new(IconPush::new(cc)))))
}
