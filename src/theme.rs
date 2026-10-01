//! Colors, fonts and small reusable UI pieces.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Frame, Margin, RichText, Sense, Stroke, Vec2,
};

pub const ACCENT: Color32 = Color32::from_rgb(124, 140, 255);
pub const BG: Color32 = Color32::from_rgb(11, 11, 15);
pub const PANEL: Color32 = Color32::from_rgb(22, 22, 28);
pub const CARD: Color32 = Color32::from_rgb(30, 30, 38);
pub const CARD_ON: Color32 = Color32::from_rgb(38, 40, 66);
pub const BORDER: Color32 = Color32::from_rgb(48, 48, 60);
pub const MUTED: Color32 = Color32::from_rgb(154, 154, 163);
pub const OK: Color32 = Color32::from_rgb(52, 199, 123);
pub const WARN: Color32 = Color32::from_rgb(240, 166, 58);
pub const DANGER: Color32 = Color32::from_rgb(232, 17, 35);

pub fn setup(ctx: &egui::Context) {
    // Segoe UI (the Windows system font) when available, for a native look.
    let mut fonts = FontDefinitions::default();
    for (name, file) in [("segoe", "segoeui.ttf"), ("segoe-semibold", "seguisb.ttf"), ("segoe-symbol", "seguisym.ttf")] {
        let path = std::path::Path::new("C:\\Windows\\Fonts").join(file);
        let Ok(bytes) = std::fs::read(path) else { continue };
        fonts.font_data.insert(name.into(), Arc::new(FontData::from_owned(bytes)));
        if name == "segoe-semibold" {
            fonts.families.insert(FontFamily::Name("semibold".into()), vec![name.into(), "segoe".into()]);
            continue;
        }
        let family = fonts.families.entry(FontFamily::Proportional).or_default();
        let at = if name == "segoe" { 0 } else { family.len() };
        family.insert(at, name.into());
    }
    // Fall back to the default font if Segoe UI Semibold is missing.
    fonts.families.entry(FontFamily::Name("semibold".into())).or_insert_with(|| vec!["Ubuntu-Light".into()]);
    ctx.set_fonts(fonts);

    let mut v = egui::Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = PANEL;
    v.extreme_bg_color = Color32::from_rgb(26, 26, 33);
    v.text_edit_bg_color = Some(Color32::from_rgb(26, 26, 33));
    v.selection.bg_fill = ACCENT.gamma_multiply(0.6);
    v.hyperlink_color = ACCENT;
    v.window_corner_radius = CornerRadius::same(14);
    for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(8);
    }
    v.widgets.inactive.weak_bg_fill = CARD;
    v.widgets.inactive.bg_fill = Color32::from_rgb(44, 44, 56); // checkbox boxes
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(220, 220, 228));
    // Always dark, even when Windows itself is in light mode.
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, v);

    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = Vec2::new(8.0, 8.0);
        s.spacing.button_padding = Vec2::new(12.0, 6.0);
        s.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(14.5));
        s.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(14.5));
        s.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(17.0, FontFamily::Name("semibold".into())));
        s.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
    });
}

pub fn semibold(size: f32) -> egui::FontId {
    egui::FontId::new(size, FontFamily::Name("semibold".into()))
}

pub fn step_title(ui: &mut egui::Ui, n: &str, title: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 11.0, ACCENT);
        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, n, semibold(13.0), Color32::WHITE);
        ui.label(RichText::new(title).heading());
    });
    ui.add_space(4.0);
}

pub fn card_frame(selected: bool) -> Frame {
    Frame::new()
        .fill(if selected { CARD_ON } else { CARD })
        .stroke(Stroke::new(1.0, if selected { ACCENT } else { BORDER }))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(8))
}

/// Large card used by the Bibliothèque and Pilotes tabs.
pub fn section(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Plain left-aligned layout (columns are justified, which stretches word spacing).
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), add);
        });
    ui.add_space(12.0);
}

pub fn primary_button(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.into()).font(semibold(15.0)).color(Color32::WHITE))
        .fill(ACCENT)
        .corner_radius(CornerRadius::same(10))
        .min_size(Vec2::new(0.0, 36.0))
}

/// Thin progress bar with a label under it.
pub fn progress(ui: &mut egui::Ui, fraction: f32, label: &str) {
    ui.add(egui::ProgressBar::new(fraction.clamp(0.0, 1.0)).desired_height(8.0).fill(ACCENT).corner_radius(4));
    ui.label(RichText::new(label).small().color(MUTED));
}
