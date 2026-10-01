//! Realistic iPhone mockup for the home screen preview: Dynamic Island, status bar,
//! icon grid, page dots, glass dock and home indicator.

use eframe::egui::{self, epaint, Color32, Pos2, Rect, Stroke, StrokeKind, TextureHandle, Vec2};

pub struct Icon<'a> {
    pub texture: TextureHandle,
    pub name: &'a str,
}

fn gradient_rect(painter: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = epaint::Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
}

fn short(name: &str, max: usize) -> String {
    if name.chars().count() > max {
        name.chars().take(max.saturating_sub(1)).chain(['…']).collect()
    } else {
        name.to_string()
    }
}

/// Draw the phone in `rect` (the phone keeps a 19.5:9 ratio inside it). Returns the phone's rect.
/// The first four icons go in the dock, the rest on the home screen grid.
pub fn draw(ui: &mut egui::Ui, rect: Rect, icons: &[Icon], hide_labels: bool) -> Rect {
    let h = rect.height().min(rect.width() * 2.1);
    let w = h / 2.1;
    let phone = Rect::from_center_size(rect.center(), Vec2::new(w, h));
    let p = ui.painter();
    let k = w / 300.0; // design units: phone is 300 wide

    // Side buttons.
    let btn = Color32::from_rgb(52, 52, 56);
    p.rect_filled(Rect::from_min_size(Pos2::new(phone.left() - 3.0 * k, phone.top() + 110.0 * k), Vec2::new(4.0 * k, 26.0 * k)), 2.0, btn);
    p.rect_filled(Rect::from_min_size(Pos2::new(phone.left() - 3.0 * k, phone.top() + 150.0 * k), Vec2::new(4.0 * k, 48.0 * k)), 2.0, btn);
    p.rect_filled(Rect::from_min_size(Pos2::new(phone.left() - 3.0 * k, phone.top() + 208.0 * k), Vec2::new(4.0 * k, 48.0 * k)), 2.0, btn);
    p.rect_filled(Rect::from_min_size(Pos2::new(phone.right() - 1.0 * k, phone.top() + 170.0 * k), Vec2::new(4.0 * k, 80.0 * k)), 2.0, btn);

    // Body and bezel.
    let body_r = 48.0 * k;
    p.rect_filled(phone, body_r, Color32::from_rgb(28, 28, 30));
    p.rect_stroke(phone, body_r, Stroke::new(2.5 * k, Color32::from_rgb(70, 70, 76)), StrokeKind::Inside);
    let screen = phone.shrink(11.0 * k);
    let screen_r = 38.0 * k;

    // Wallpaper: vertical gradient + two soft blobs, all clipped to the screen.
    let wall = p.with_clip_rect(screen);
    wall.rect_filled(screen, screen_r, Color32::from_rgb(18, 16, 40));
    gradient_rect(&wall, screen.shrink2(Vec2::new(0.0, screen_r * 0.6)), Color32::from_rgb(44, 46, 112), Color32::from_rgb(22, 14, 44));
    for (c, rel, rad) in [
        (Color32::from_rgba_unmultiplied(120, 130, 255, 50), Vec2::new(0.15, 0.2), 0.75),
        (Color32::from_rgba_unmultiplied(255, 110, 180, 42), Vec2::new(0.9, 0.78), 0.7),
    ] {
        wall.circle_filled(screen.min + screen.size() * rel, screen.width() * rad, c);
    }
    // Round the wallpaper corners by painting the bezel color outside the rounded screen.
    p.rect_stroke(screen.expand(6.0 * k), screen_r + 6.0 * k, Stroke::new(12.0 * k, Color32::from_rgb(28, 28, 30)), StrokeKind::Middle);

    // Status bar and Dynamic Island.
    let white = Color32::WHITE;
    let top = screen.top() + 22.0 * k;
    p.text(Pos2::new(screen.left() + 42.0 * k, top), egui::Align2::CENTER_CENTER, "9:41", crate::theme::semibold(15.0 * k), white);
    let island = Rect::from_center_size(Pos2::new(screen.center().x, top), Vec2::new(96.0 * k, 28.0 * k));
    p.rect_filled(island, 14.0 * k, Color32::BLACK);
    let right = screen.right() - 30.0 * k;
    // Battery.
    let batt = Rect::from_center_size(Pos2::new(right, top), Vec2::new(24.0 * k, 11.5 * k));
    p.rect_stroke(batt, 3.0 * k, Stroke::new(1.0 * k, Color32::from_white_alpha(140)), StrokeKind::Inside);
    p.rect_filled(batt.shrink(2.0 * k), 1.5 * k, white);
    p.rect_filled(Rect::from_center_size(Pos2::new(batt.right() + 2.0 * k, top), Vec2::new(2.0 * k, 4.0 * k)), 1.0, Color32::from_white_alpha(140));
    // Signal bars.
    for i in 0..4 {
        let bh = (4.0 + i as f32 * 2.0) * k;
        let x = right - 38.0 * k + i as f32 * 4.0 * k;
        p.rect_filled(Rect::from_min_max(Pos2::new(x, top + 5.0 * k - bh), Pos2::new(x + 2.6 * k, top + 5.0 * k)), 0.8, white);
    }

    // Layout: grid from the top, dock at the bottom.
    let pad = 18.0 * k;
    let cell = (screen.width() - pad * 2.0) / 4.0;
    let icon = cell * 0.78;
    let radius = (icon * 0.225) as u8;
    let label_h = if hide_labels { 0.0 } else { 16.0 * k };
    let row_h = icon + label_h + 14.0 * k;
    let dock_h = icon + 26.0 * k;
    let dock = Rect::from_min_size(
        Pos2::new(screen.left() + 10.0 * k, screen.bottom() - dock_h - 22.0 * k),
        Vec2::new(screen.width() - 20.0 * k, dock_h),
    );
    let grid_top = screen.top() + 58.0 * k;
    let grid_rows = ((dock.top() - 26.0 * k - grid_top) / row_h).floor().max(0.0) as usize;
    let max_chars = (cell / (5.6 * k)).floor().max(4.0) as usize;

    let (dock_icons, grid_icons) = icons.split_at(icons.len().min(4));
    for (n, ic) in grid_icons.iter().take(grid_rows * 4).enumerate() {
        let cx = screen.left() + pad + cell * ((n % 4) as f32 + 0.5);
        let y = grid_top + row_h * (n / 4) as f32;
        let r = Rect::from_center_size(Pos2::new(cx, y + icon / 2.0), Vec2::splat(icon));
        egui::Image::new(&ic.texture).corner_radius(radius).paint_at(ui, r);
        if !hide_labels {
            ui.painter().text(
                Pos2::new(cx, r.bottom() + 4.0 * k),
                egui::Align2::CENTER_TOP,
                short(ic.name, max_chars),
                egui::FontId::proportional(10.5 * k),
                white,
            );
        }
    }

    // Page dots.
    let dots_y = dock.top() - 13.0 * k;
    for i in 0..3 {
        let c = if i == 0 { white } else { Color32::from_white_alpha(90) };
        ui.painter().circle_filled(Pos2::new(screen.center().x + (i as f32 - 1.0) * 12.0 * k, dots_y), 3.0 * k, c);
    }

    // Glass dock.
    let p = ui.painter();
    p.rect_filled(dock, 30.0 * k, Color32::from_white_alpha(38));
    p.rect_stroke(dock, 30.0 * k, Stroke::new(1.0, Color32::from_white_alpha(40)), StrokeKind::Inside);
    let dock_cell = dock.width() / 4.0;
    for (n, ic) in dock_icons.iter().enumerate() {
        let cx = dock.left() + dock_cell * (n as f32 + 0.5);
        let r = Rect::from_center_size(Pos2::new(cx, dock.center().y), Vec2::splat(icon));
        egui::Image::new(&ic.texture).corner_radius(radius).paint_at(ui, r);
    }

    // Home indicator.
    let bar = Rect::from_center_size(Pos2::new(screen.center().x, screen.bottom() - 9.0 * k), Vec2::new(110.0 * k, 4.5 * k));
    ui.painter().rect_filled(bar, 3.0 * k, white);

    phone
}
