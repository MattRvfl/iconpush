//! The Bibliothèque (icon library) and Pilotes (Apple driver) tabs.

use std::sync::{Arc, Mutex};

use eframe::egui::{self, Color32, RichText, Vec2};

use crate::drivers::{self, InstallState};
use crate::theme::{self, MUTED, OK, WARN};

// ------------------------------------------------------------------ iPhone

fn gb(bytes: u64) -> String {
    // Apple counts storage in base 1000, like Settings.
    format!("{:.1} Go", bytes as f64 / 1e9)
}

fn bar(ui: &mut egui::Ui, fraction: f32, color: Color32) {
    ui.add(egui::ProgressBar::new(fraction.clamp(0.0, 1.0)).desired_height(10.0).fill(color).corner_radius(5));
}

fn big_value(ui: &mut egui::Ui, value: &str, unit: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(value).font(theme::semibold(34.0)));
        ui.label(RichText::new(unit).color(MUTED));
    });
}

fn row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(value);
        });
    });
}

/// Returns true when the user asks for a refresh.
pub fn phone(
    ui: &mut egui::Ui,
    info: Option<&crate::phoneinfo::PhoneInfo>,
    loading: bool,
    connected: bool,
    error: Option<&str>,
    show_serial: &mut bool,
) -> bool {
    let mut refresh = false;
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.set_max_width(980.0);

        let Some(info) = info.filter(|_| connected) else {
            ui.label(RichText::new("Ton iPhone").font(theme::semibold(22.0)));
            ui.add_space(14.0);
            theme::section(ui, |ui| {
                if connected || loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Lecture des informations de l'iPhone…");
                    });
                } else {
                    ui.label(RichText::new("Aucun iPhone connecté").font(theme::semibold(17.0)));
                    ui.label("1. Branche ton iPhone avec un câble USB.");
                    ui.label("2. Déverrouille-le.");
                    ui.label("3. Touche « Se fier » quand il te le demande, puis entre ton code.");
                    if let Some(e) = error {
                        ui.add_space(6.0);
                        ui.label(RichText::new(e).small().color(WARN));
                    }
                }
            });
            return;
        };

        // Header.
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(&info.name).font(theme::semibold(24.0)));
                ui.label(RichText::new(format!("{} · iOS {} ({})", info.model_name, info.ios, info.build)).color(MUTED));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if loading {
                    ui.spinner();
                } else if ui.button("Actualiser").clicked() {
                    refresh = true;
                }
            });
        });
        ui.add_space(12.0);

        ui.columns(2, |cols| {
            // Battery level.
            theme::section(&mut cols[0], |ui| {
                ui.label(RichText::new("Batterie").font(theme::semibold(16.0)));
                match info.battery_level {
                    Some(level) => {
                        big_value(ui, &level.to_string(), "%");
                        let color = if level <= 20 { Color32::from_rgb(240, 80, 80) } else { OK };
                        bar(ui, level as f32 / 100.0, color);
                        ui.label(match info.charging {
                            Some(true) => "En charge",
                            Some(false) => "Sur batterie",
                            None => "",
                        });
                    }
                    None => {
                        ui.label(RichText::new("Indisponible").color(MUTED));
                    }
                }
            });

            // Battery health.
            theme::section(&mut cols[1], |ui| {
                ui.label(RichText::new("Santé de la batterie").font(theme::semibold(16.0)));
                match info.health_percent() {
                    Some(h) => {
                        big_value(ui, &format!("{h:.0}"), "% de capacité maximale");
                        let (color, verdict) = if h >= 80.0 {
                            (OK, "Bon état")
                        } else {
                            (WARN, "Usée : Apple conseille un remplacement sous 80 %")
                        };
                        bar(ui, h as f32 / 100.0, color);
                        ui.label(RichText::new(verdict).color(color));
                    }
                    None => {
                        ui.label(RichText::new("Indisponible sur ce modèle ou cette version d'iOS").color(MUTED));
                    }
                }
                ui.add_space(4.0);
                if let Some(c) = info.cycle_count {
                    row(ui, "Cycles de charge", &c.to_string());
                }
                if let (Some(max), Some(design)) = (info.max_mah, info.design_mah) {
                    row(ui, "Capacité réelle / d'origine", &format!("{max} / {design} mAh"));
                }
                if let Some(t) = info.temperature_c {
                    row(ui, "Température", &format!("{t:.1} °C"));
                }
                ui.label(
                    RichText::new("Calculé à partir des données brutes de la batterie : peut différer d'un point de Réglages.")
                        .small()
                        .color(MUTED),
                );
            });

            // Storage.
            theme::section(&mut cols[0], |ui| {
                ui.label(RichText::new("Stockage").font(theme::semibold(16.0)));
                match (info.disk_total, info.disk_free) {
                    (Some(total), Some(free)) if total > 0 => {
                        let used = total.saturating_sub(free);
                        big_value(ui, &gb(used), &format!("utilisés sur {}", gb(total)));
                        let f = used as f32 / total as f32;
                        bar(ui, f, if f > 0.9 { WARN } else { theme::ACCENT });
                        ui.label(format!("{} libres", gb(free)));
                    }
                    _ => {
                        ui.label(RichText::new("Indisponible").color(MUTED));
                    }
                }
            });

            // Device details.
            theme::section(&mut cols[1], |ui| {
                ui.label(RichText::new("Appareil").font(theme::semibold(16.0)));
                row(ui, "Modèle", &info.model_name);
                row(ui, "Identifiant", &info.model_id);
                row(ui, "iOS", &format!("{} ({})", info.ios, info.build));
                if !info.region.is_empty() {
                    row(ui, "Région", &info.region);
                }
                if let Some(n) = info.user_apps {
                    row(ui, "Apps installées", &n.to_string());
                }
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Numéro de série").color(MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(if *show_serial { "Masquer" } else { "Afficher" }).clicked() {
                            *show_serial = !*show_serial;
                        }
                        ui.label(if *show_serial { info.serial.clone() } else { "••••••••••".into() });
                    });
                });
            });
        });

        ui.label(RichText::new("Lecture seule : iconpush ne modifie rien sur ton iPhone.").small().color(MUTED));
    });
    refresh
}

// ------------------------------------------------------------------ library

#[derive(Clone, PartialEq)]
pub enum LogoState {
    Idle,
    Running { done: usize, total: usize },
    Done(usize),
    Error(String),
}

pub enum LibraryAction {
    None,
    DownloadLogos,
    ImportFolder,
    ShowLogoPacks,
}

pub fn library(ui: &mut egui::Ui, logos_count: usize, logo_apps: usize, state: &LogoState, import_msg: Option<&str>) -> LibraryAction {
    let mut action = LibraryAction::None;
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.set_max_width(860.0);
        ui.label(RichText::new("Bibliothèque d'icônes").font(theme::semibold(22.0)));
        ui.label(RichText::new("Ajoute de nouvelles icônes à iconpush, depuis des sources sûres.").color(MUTED));
        ui.add_space(14.0);

        theme::section(ui, |ui| {
            ui.label(RichText::new("Logos officiels des apps").font(theme::semibold(17.0)));
            ui.label(
                "Les vrais logos de Spotify, Instagram, WhatsApp, YouTube… téléchargés depuis Simple Icons, \
                 une bibliothèque open source (licence CC0) utilisée par des milliers de projets. \
                 Ils débloquent 3 packs : Logos officiels, Logos sur noir et Logos Liquid Glass.",
            );
            ui.add_space(8.0);
            match state {
                LogoState::Running { done, total } => {
                    let f = if *total == 0 { 0.0 } else { *done as f32 / *total as f32 };
                    theme::progress(ui, f, &format!("Téléchargement… {done} / {total} logos"));
                }
                _ => {
                    ui.horizontal(|ui| {
                        let label = if logos_count == 0 { format!("Télécharger les logos ({logo_apps})") } else { "Mettre à jour les logos".into() };
                        if ui.add(theme::primary_button(label)).clicked() {
                            action = LibraryAction::DownloadLogos;
                        }
                        if logos_count > 0 && ui.button("Voir les packs de logos").clicked() {
                            action = LibraryAction::ShowLogoPacks;
                        }
                    });
                    match state {
                        LogoState::Done(n) => {
                            ui.label(RichText::new(format!("✓ {n} logos téléchargés. Les packs sont dans l'onglet Icônes.")).color(OK));
                        }
                        LogoState::Error(e) => {
                            ui.label(RichText::new(format!("Échec : {e}")).color(WARN));
                        }
                        _ if logos_count > 0 => {
                            ui.label(RichText::new(format!("✓ {logos_count} logos disponibles.")).color(OK));
                        }
                        _ => {}
                    }
                }
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Les logos restent la propriété de leurs marques : ils sont faits pour ton usage personnel, \
                     pas pour être revendus ou redistribués dans un pack.",
                )
                .small()
                .color(MUTED),
            );
        });

        theme::section(ui, |ui| {
            ui.label(RichText::new("Importer tes propres icônes (Figma, Canva, Photoshop…)").font(theme::semibold(17.0)));
            ui.label("Choisis un dossier d'images : iconpush associe chaque fichier à son app grâce à son nom.");
            ui.add_space(6.0);
            ui.label(RichText::new("Depuis Figma :").strong());
            for step in [
                "1. Ouvre ton fichier d'icônes (ou un pack trouvé sur Figma Community).",
                "2. Renomme chaque icône comme l'app : spotify, instagram, whatsapp, « Google Maps »…",
                "3. Sélectionne-les, puis Export → PNG → 3x → Export.",
                "4. Clique sur « Choisir un dossier » et sélectionne le dossier exporté.",
            ] {
                ui.label(step);
            }
            ui.add_space(8.0);
            if ui.add(theme::primary_button("Choisir un dossier…")).clicked() {
                action = LibraryAction::ImportFolder;
            }
            if let Some(msg) = import_msg {
                ui.label(RichText::new(msg).color(MUTED));
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new("Formats acceptés : PNG, JPG, WebP. Les images sont recadrées en carré automatiquement.")
                    .small()
                    .color(MUTED),
            );
        });
    });
    action
}

// ------------------------------------------------------------------ drivers

fn mb(bytes: u64) -> String {
    format!("{:.0} Mo", bytes as f64 / 1_048_576.0)
}

pub fn drivers(ui: &mut egui::Ui, driver_ok: bool, state: &Arc<Mutex<InstallState>>, store_error: &mut Option<String>) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.set_max_width(860.0);
        ui.label(RichText::new("Pilotes Apple").font(theme::semibold(22.0)));
        ui.label(
            RichText::new("Windows a besoin du pilote USB d'Apple pour communiquer avec un iPhone. Il est fourni avec iTunes et avec l'app Appareils Apple.")
                .color(MUTED),
        );
        ui.add_space(14.0);

        theme::section(ui, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 8.0, if driver_ok { OK } else { WARN });
                ui.label(
                    RichText::new(if driver_ok { "Pilote Apple actif" } else { "Pilote Apple introuvable" }).font(theme::semibold(18.0)),
                );
            });
            ui.label(if driver_ok {
                "Tout est prêt : branche ton iPhone, déverrouille-le et touche « Se fier »."
            } else {
                "Installe l'une des deux options ci-dessous, puis débranche et rebranche ton iPhone."
            });
        });

        let current = state.lock().unwrap().clone();
        ui.columns(2, |cols| {
            // Option 1: iTunes 64-bit, downloaded from apple.com and verified.
            theme::section(&mut cols[0], |ui| {
                ui.label(RichText::new("iTunes 64 bits").font(theme::semibold(17.0)));
                ui.label(RichText::new("Recommandé si l'app Appareils Apple ne s'installe pas").small().color(MUTED));
                ui.add_space(6.0);
                ui.label("Téléchargé directement depuis apple.com (≈ 195 Mo). iconpush vérifie la signature numérique d'Apple avant de lancer l'installation.");
                ui.add_space(8.0);
                match &current {
                    InstallState::Downloading { done, total } => {
                        let f = if *total == 0 { 0.0 } else { *done as f32 / *total as f32 };
                        theme::progress(ui, f, &format!("Téléchargement… {} / {}", mb(*done), if *total > 0 { mb(*total) } else { "?".into() }));
                    }
                    InstallState::Verifying => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Vérification de la signature Apple…");
                        });
                    }
                    InstallState::Launched => {
                        ui.label(RichText::new("✓ Installateur lancé. Accepte la demande de Windows, suis les étapes, puis rebranche ton iPhone.").color(OK));
                    }
                    InstallState::Error(e) => {
                        ui.label(RichText::new(format!("Échec : {e}")).color(WARN));
                    }
                    InstallState::Idle => {}
                }
                let busy = matches!(current, InstallState::Downloading { .. } | InstallState::Verifying);
                let label = if matches!(current, InstallState::Error(_)) { "Réessayer" } else { "Télécharger et installer" };
                if ui.add_enabled(!busy && !driver_ok, theme::primary_button(label)).clicked() {
                    *state.lock().unwrap() = InstallState::Downloading { done: 0, total: 0 };
                    drivers::install_itunes(state.clone(), ui.ctx().clone());
                }
            });

            // Option 2: Apple Devices from the Microsoft Store.
            theme::section(&mut cols[1], |ui| {
                ui.label(RichText::new("Appareils Apple").font(theme::semibold(17.0)));
                ui.label(RichText::new("L'app officielle d'Apple, plus légère").small().color(MUTED));
                ui.add_space(6.0);
                ui.label("Disponible sur le Microsoft Store (Windows 10 22H2 ou plus récent). Elle se met à jour toute seule.");
                ui.add_space(8.0);
                if ui.add_enabled(!driver_ok, theme::primary_button("Ouvrir le Microsoft Store")).clicked() {
                    *store_error = drivers::open_store().err();
                }
                if let Some(e) = store_error {
                    ui.label(RichText::new(format!("Impossible d'ouvrir le Store : {e}")).color(WARN));
                }
            });
        });

        ui.label(
            RichText::new(
                "Note : ces pilotes s'installent forcément sur le disque système (C:), comme tout pilote Windows. \
                 Le fichier d'installation d'iTunes est téléchargé dans le dossier iconpush-data, à côté d'iconpush.",
            )
            .small()
            .color(Color32::from_rgb(130, 130, 140)),
        );
    });
}
