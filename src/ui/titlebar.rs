//! Scaleri's own title bar, which replaces the Windows one: logo, launch status, window buttons,
//! dragging, double-click to maximise, and resizing from the window edges.

use egui::{pos2, Color32, CursorIcon, Rect, ResizeDirection, Sense, Ui, Vec2, ViewportCommand};

use super::theme::{self, bold_font};

pub const HEIGHT: f32 = 42.0;
const BUTTON_WIDTH: f32 = 46.0;
const RESIZE_MARGIN: f32 = 6.0;

/// What the middle of the title bar shows.
pub enum Status {
    Idle,
    Preparing {
        name: String,
        label: String,
        fraction: Option<f32>,
    },
    Playing(String),
}

#[derive(Clone, Copy, PartialEq)]
enum Button {
    Minimize,
    Maximize,
    Close,
}

pub fn show(ui: &mut Ui, logo: &egui::TextureHandle, status: &Status, latest: Option<&str>) {
    let ctx = ui.ctx().clone();
    let rect = ui.max_rect();
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

    // The whole bar drags the window; the buttons are added afterwards so they win clicks.
    let bar = ui.interact(rect, ui.scope_id().with("titlebar"), Sense::click_and_drag());
    if bar.double_clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    } else if bar.drag_started_by(egui::PointerButton::Primary) {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }

    let p = ui.painter();
    p.rect_filled(rect, 0.0, theme::TITLEBAR);
    // A strip of grass and dirt along the bottom edge
    let strip = Rect::from_min_max(pos2(rect.min.x, rect.max.y - 4.0), rect.max);
    p.rect_filled(strip, 0.0, Color32::from_rgb(0x5a, 0x3d, 0x26));
    p.rect_filled(
        Rect::from_min_max(strip.min, pos2(strip.max.x, strip.min.y + 2.0)),
        0.0,
        theme::GREEN,
    );

    // Logo and name over the sidebar
    let logo_rect = Rect::from_min_size(rect.min + Vec2::new(16.0, 8.0), Vec2::splat(24.0));
    egui::Image::new(logo).paint_at(ui, logo_rect);
    p.text(
        pos2(logo_rect.max.x + 10.0, logo_rect.center().y),
        egui::Align2::LEFT_CENTER,
        "SCALERI",
        bold_font(17.0),
        theme::GREEN_LIGHT,
    );

    // Status in the middle
    let status_x = rect.min.x + 236.0;
    let mid_y = rect.min.y + (HEIGHT - 4.0) / 2.0;
    match status {
        Status::Idle => {
            if let Some(latest) = latest {
                pill(
                    ui,
                    pos2(status_x, mid_y),
                    &format!("Latest release: Minecraft {latest}"),
                    theme::MUTED,
                    None,
                );
            }
        }
        Status::Preparing { name, label, fraction } => {
            let text = match fraction {
                Some(f) => format!("{name} · {label} {:.0}%", f * 100.0),
                None => format!("{name} · {label}"),
            };
            pill(ui, pos2(status_x, mid_y), &text, theme::GOLD, *fraction);
        }
        Status::Playing(name) => {
            pill(
                ui,
                pos2(status_x, mid_y),
                &format!("Playing {name}"),
                theme::GREEN_LIGHT,
                None,
            );
        }
    }

    // Window buttons
    let mut x = rect.max.x - BUTTON_WIDTH * 3.0;
    for button in [Button::Minimize, Button::Maximize, Button::Close] {
        let r = Rect::from_min_size(pos2(x, rect.min.y), Vec2::new(BUTTON_WIDTH, HEIGHT - 4.0));
        let response = ui.interact(r, ui.scope_id().with(("window-button", button as u8)), Sense::click());
        paint_button(
            ui,
            r,
            button,
            maximized,
            response.hovered(),
            response.is_pointer_button_down_on(),
        );
        if response.clicked() {
            match button {
                Button::Minimize => ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
                Button::Maximize => ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized)),
                Button::Close => ctx.send_viewport_cmd(ViewportCommand::Close),
            }
        }
        x += BUTTON_WIDTH;
    }
}

/// A small status capsule with a coloured square, optionally with a progress fill.
fn pill(ui: &Ui, left_center: egui::Pos2, text: &str, color: Color32, fraction: Option<f32>) {
    let p = ui.painter();
    let galley = p.layout_no_wrap(text.to_string(), egui::FontId::proportional(13.0), theme::TEXT);
    let size = Vec2::new(galley.size().x + 34.0, 26.0);
    let rect = Rect::from_min_size(pos2(left_center.x, left_center.y - size.y / 2.0), size);
    p.rect_filled(rect, 0.0, Color32::from_rgb(0x1c, 0x1c, 0x1d));
    if let Some(f) = fraction {
        let fill = Rect::from_min_size(rect.min, Vec2::new(rect.width() * f.clamp(0.0, 1.0), rect.height()));
        p.rect_filled(fill, 0.0, Color32::from_rgb(0x2b, 0x3d, 0x23));
    }
    p.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, theme::EDGE), egui::StrokeKind::Inside);
    let dot = Rect::from_center_size(pos2(rect.min.x + 13.0, rect.center().y), Vec2::splat(8.0));
    p.rect_filled(dot, 0.0, color);
    p.galley(
        pos2(rect.min.x + 24.0, rect.center().y - galley.size().y / 2.0),
        galley,
        theme::TEXT,
    );
}

fn paint_button(ui: &Ui, rect: Rect, button: Button, maximized: bool, hovered: bool, pressed: bool) {
    let p = ui.painter();
    if hovered {
        let fill = match (button, pressed) {
            (Button::Close, false) => theme::RED,
            (Button::Close, true) => Color32::from_rgb(0x8a, 0x1c, 0x16),
            (_, false) => theme::PANEL,
            (_, true) => theme::STONE,
        };
        p.rect_filled(rect, 0.0, fill);
    }
    let color = if hovered { Color32::WHITE } else { theme::TEXT };
    let c = rect.center();
    let px = 2.0;
    let dot = |x: f32, y: f32| {
        p.rect_filled(
            Rect::from_min_size(pos2(c.x + x * px, c.y + y * px), Vec2::splat(px)),
            0.0,
            color,
        );
    };
    match button {
        Button::Minimize => {
            for x in -3..3 {
                dot(x as f32, 1.0);
            }
        }
        Button::Maximize if maximized => {
            // Restore: a square with another one peeking out behind it
            for i in -3..1 {
                dot(i as f32, -1.0);
                dot(i as f32, 2.0);
            }
            for i in -1..3 {
                dot(-3.0, i as f32);
                dot(0.0, i as f32);
            }
            for i in -1..3 {
                dot(i as f32, -3.0);
            }
            for i in -3..1 {
                dot(2.0, i as f32);
            }
        }
        Button::Maximize => {
            for i in -3..3 {
                dot(i as f32, -3.0);
                dot(i as f32, 2.0);
                dot(-3.0, i as f32);
                dot(2.0, i as f32);
            }
        }
        Button::Close => {
            for i in -3..3 {
                dot(i as f32, i as f32);
                dot(i as f32, -i as f32 - 1.0);
            }
        }
    }
}

/// Lets the borderless window be resized from its edges and corners.
pub fn resize_edges(ctx: &egui::Context) {
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    if maximized {
        return;
    }
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    let screen = ctx.content_rect();
    let left = pos.x - screen.min.x < RESIZE_MARGIN;
    let right = screen.max.x - pos.x < RESIZE_MARGIN;
    let top = pos.y - screen.min.y < RESIZE_MARGIN;
    let bottom = screen.max.y - pos.y < RESIZE_MARGIN;
    let direction = match (left, right, top, bottom) {
        (true, _, true, _) => ResizeDirection::NorthWest,
        (_, true, true, _) => ResizeDirection::NorthEast,
        (true, _, _, true) => ResizeDirection::SouthWest,
        (_, true, _, true) => ResizeDirection::SouthEast,
        (true, ..) => ResizeDirection::West,
        (_, true, ..) => ResizeDirection::East,
        (_, _, true, _) => ResizeDirection::North,
        (_, _, _, true) => ResizeDirection::South,
        _ => return,
    };
    let cursor = match direction {
        ResizeDirection::NorthWest | ResizeDirection::SouthEast => CursorIcon::ResizeNwSe,
        ResizeDirection::NorthEast | ResizeDirection::SouthWest => CursorIcon::ResizeNeSw,
        ResizeDirection::West | ResizeDirection::East => CursorIcon::ResizeHorizontal,
        ResizeDirection::North | ResizeDirection::South => CursorIcon::ResizeVertical,
    };
    ctx.set_cursor_icon(cursor);
    if ctx.input(|i| i.pointer.primary_pressed()) {
        ctx.send_viewport_cmd(ViewportCommand::BeginResize(direction));
    }
}
