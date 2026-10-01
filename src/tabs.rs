//! The Bibliothèque (icon library) and Pilotes (Apple driver) tabs.

use std::sync::{Arc, Mutex};

use eframe::egui::{self, Color32, RichText, Vec2};

use crate::drivers::{self, InstallState};
use crate::theme::{self, MUTED, OK, WARN};

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
