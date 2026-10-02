//! Custom window chrome: the title bar is part of the app (logo, tabs, iPhone status,
//! window buttons), with dragging, double-click to maximize and resizing from the edges.

use eframe::egui::{
    self, Align, Color32, CornerRadius, CursorIcon, Frame, Id, Layout, Margin, Pos2, Rect, RichText, Sense, Stroke,
    Vec2, ViewportCommand,
};

use crate::theme::{self, ACCENT, BORDER, CARD, DANGER, MUTED, PANEL};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Phone,
    Icons,
    Library,
    Drivers,
}

pub struct Status {
    pub dot: Color32,
    pub text: String,
    pub tooltip: String,
    /// Clicking the status chip opens this tab (e.g. Drivers when the driver is missing).
    pub link: Option<Tab>,
}

fn is_maximized(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.viewport().maximized.unwrap_or(false))
}

fn logo(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
    let s = 8.0;
    for (i, c) in [Color32::WHITE, ACCENT, ACCENT, Color32::WHITE].into_iter().enumerate() {
        let min = rect.min + Vec2::new((i % 2) as f32 * (s + 2.0), (i / 2) as f32 * (s + 2.0));
        ui.painter().rect_filled(Rect::from_min_size(min, Vec2::splat(s)), 2.5, c);
    }
}

fn tab_button(ui: &mut egui::Ui, current: &mut Tab, tab: Tab, label: &str) {
    let on = *current == tab;
    let text = RichText::new(label).font(theme::semibold(14.0)).color(if on { Color32::WHITE } else { MUTED });
    let btn = egui::Button::new(text)
        .fill(if on { CARD } else { Color32::TRANSPARENT })
        .stroke(Stroke::new(1.0, if on { BORDER } else { Color32::TRANSPARENT }))
        .corner_radius(CornerRadius::same(8));
    if ui.add(btn).clicked() {
        *current = tab;
    }
}

/// One of the three window buttons, drawn by hand like Windows 11.
fn window_button(ui: &mut egui::Ui, kind: u8, maximized: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(46.0, 34.0), Sense::click());
    let hovered = resp.hovered();
    let p = ui.painter();
    if hovered {
        p.rect_filled(rect, 0.0, if kind == 2 { DANGER } else { Color32::from_white_alpha(18) });
    }
    let c = rect.center();
    let stroke = Stroke::new(1.0, Color32::from_rgb(230, 230, 236));
    match kind {
        0 => {
            p.line_segment([c - Vec2::new(5.0, 0.0), c + Vec2::new(5.0, 0.0)], stroke);
        }
        1 if maximized => {
            p.rect_stroke(Rect::from_center_size(c + Vec2::new(-1.5, 1.5), Vec2::splat(8.0)), 1.0, stroke, egui::StrokeKind::Middle);
            p.line_segment([c + Vec2::new(-2.5, -4.5), c + Vec2::new(4.5, -4.5)], stroke);
            p.line_segment([c + Vec2::new(4.5, -4.5), c + Vec2::new(4.5, 2.5)], stroke);
        }
        1 => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::splat(10.0)), 1.0, stroke, egui::StrokeKind::Middle);
        }
        _ => {
            p.line_segment([c + Vec2::new(-5.0, -5.0), c + Vec2::new(5.0, 5.0)], stroke);
            p.line_segment([c + Vec2::new(-5.0, 5.0), c + Vec2::new(5.0, -5.0)], stroke);
        }
    }
    resp.clicked()
}

pub fn title_bar(ui: &mut egui::Ui, tab: &mut Tab, status: &Status) {
    let ctx = ui.ctx().clone();
    let maximized = is_maximized(&ctx);
    let bar = Frame::new().fill(PANEL).inner_margin(Margin { left: 14, right: 0, top: 0, bottom: 0 });

    egui::Panel::top("title-bar").frame(bar).exact_size(44.0).show(ui, |ui| {
        // Empty space of the bar: drag to move, double-click to maximize.
        let drag = ui.interact(ui.max_rect(), Id::new("title-drag"), Sense::click_and_drag());
        if drag.double_clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
        } else if drag.drag_started() {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        ui.horizontal_centered(|ui| {
            logo(ui);
            ui.label(RichText::new("iconpush").font(theme::semibold(16.0)));
            ui.add_space(18.0);
            tab_button(ui, tab, Tab::Phone, "iPhone");
            tab_button(ui, tab, Tab::Icons, "Icônes");
            tab_button(ui, tab, Tab::Library, "Bibliothèque");
            tab_button(ui, tab, Tab::Drivers, "Pilotes");

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                if window_button(ui, 2, maximized) {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
                if window_button(ui, 1, maximized) {
                    ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                }
                if window_button(ui, 0, maximized) {
                    ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                }
                ui.add_space(12.0);

                let chip = Frame::new()
                    .fill(CARD)
                    .corner_radius(CornerRadius::same(255))
                    .inner_margin(Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            ui.label(&status.text);
                            let (r, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                            ui.painter().circle_filled(r.center(), 4.5, status.dot);
                        });
                    })
                    .response
                    .interact(Sense::click());
                let chip = if status.tooltip.is_empty() { chip } else { chip.on_hover_text(&status.tooltip) };
                if let Some(link) = status.link {
                    if chip.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        *tab = link;
                    }
                }
            });
        });
    });
}

/// Let the user resize the borderless window from its edges, and draw a thin border.
pub fn edges(ctx: &egui::Context, window: Rect) {
    if is_maximized(ctx) {
        return;
    }
    egui::Area::new(Id::new("window-border"))
        .order(egui::Order::Foreground)
        .interactable(false)
        .fixed_pos(Pos2::ZERO)
        .show(ctx, |ui| {
            ui.painter().rect_stroke(window, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Inside);
        });

    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else { return };
    let m = 6.0;
    let (l, r) = (pos.x - window.left() < m, window.right() - pos.x < m);
    let (t, b) = (pos.y - window.top() < m, window.bottom() - pos.y < m);
    use egui::viewport::ResizeDirection as D;
    let dir = match (l, r, t, b) {
        (true, _, true, _) => Some((D::NorthWest, CursorIcon::ResizeNwSe)),
        (_, true, true, _) => Some((D::NorthEast, CursorIcon::ResizeNeSw)),
        (true, _, _, true) => Some((D::SouthWest, CursorIcon::ResizeNeSw)),
        (_, true, _, true) => Some((D::SouthEast, CursorIcon::ResizeNwSe)),
        (true, ..) => Some((D::West, CursorIcon::ResizeHorizontal)),
        (_, true, ..) => Some((D::East, CursorIcon::ResizeHorizontal)),
        (_, _, true, _) => Some((D::North, CursorIcon::ResizeVertical)),
        (.., true) => Some((D::South, CursorIcon::ResizeVertical)),
        _ => None,
    };
    if let Some((dir, cursor)) = dir {
        ctx.set_cursor_icon(cursor);
        if ctx.input(|i| i.pointer.primary_pressed()) {
            ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
        }
    }
}
