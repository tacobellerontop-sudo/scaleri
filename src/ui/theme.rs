//! Scaleri's look: deepslate greys, square corners, bevelled block buttons and grass-green accents.

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Rect, Response, Sense, Stroke, StrokeKind,
    TextStyle, Ui, Vec2,
};

pub const BG: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x1f);
pub const PANEL: Color32 = Color32::from_rgb(0x26, 0x26, 0x27);
pub const SIDEBAR: Color32 = Color32::from_rgb(0x18, 0x18, 0x19);
pub const EDGE: Color32 = Color32::from_rgb(0x10, 0x10, 0x11);
pub const FIELD: Color32 = Color32::from_rgb(0x14, 0x14, 0x15);
pub const TEXT: Color32 = Color32::from_rgb(0xec, 0xec, 0xec);
pub const MUTED: Color32 = Color32::from_rgb(0xa0, 0xa0, 0xa4);
pub const GREEN: Color32 = Color32::from_rgb(0x3c, 0x85, 0x27);
pub const GREEN_LIGHT: Color32 = Color32::from_rgb(0x6c, 0xc3, 0x49);
pub const GREEN_DARK: Color32 = Color32::from_rgb(0x1d, 0x4f, 0x12);
pub const STONE: Color32 = Color32::from_rgb(0x48, 0x48, 0x4b);
pub const RED: Color32 = Color32::from_rgb(0xb3, 0x26, 0x1e);
pub const GOLD: Color32 = Color32::from_rgb(0xfc, 0xd3, 0x4c);

pub fn bold() -> FontFamily {
    FontFamily::Name("bold".into())
}

pub fn bold_font(size: f32) -> FontId {
    FontId::new(size, bold())
}

/// Uses Segoe UI on Windows; elsewhere egui's built-in font stands in for it.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut bold_family = fonts.families[&FontFamily::Proportional].clone();
    let dir = std::path::Path::new("C:\\Windows\\Fonts");
    if let Ok(regular) = std::fs::read(dir.join("segoeui.ttf")) {
        fonts
            .font_data
            .insert("segoe".into(), FontData::from_owned(regular).into());
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "segoe".into());
        bold_family.insert(0, "segoe".into());
    }
    if let Ok(bold) = std::fs::read(dir.join("segoeuib.ttf")) {
        fonts
            .font_data
            .insert("segoe-bold".into(), FontData::from_owned(bold).into());
        bold_family.insert(0, "segoe-bold".into());
    }
    fonts.families.insert(bold(), bold_family);
    ctx.set_fonts(fonts);
}

pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.all_styles_mut(|style| {
        style.text_styles.insert(TextStyle::Body, FontId::proportional(14.0));
        style.text_styles.insert(TextStyle::Button, FontId::proportional(14.0));
        style.text_styles.insert(TextStyle::Small, FontId::proportional(12.0));
        style.text_styles.insert(TextStyle::Heading, bold_font(22.0));
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        style.spacing.interact_size.y = 30.0;
        style.spacing.slider_width = 300.0;

        let v = &mut style.visuals;
        v.dark_mode = true;
        v.override_text_color = Some(TEXT);
        v.panel_fill = BG;
        v.window_fill = PANEL;
        v.window_stroke = Stroke::new(2.0, EDGE);
        v.window_corner_radius = CornerRadius::ZERO;
        v.menu_corner_radius = CornerRadius::ZERO;
        v.extreme_bg_color = FIELD;
        v.faint_bg_color = PANEL;
        v.hyperlink_color = GREEN_LIGHT;
        v.selection.bg_fill = GREEN;
        v.selection.stroke = Stroke::new(1.0, TEXT);
        v.slider_trailing_fill = true;
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = CornerRadius::ZERO;
            w.expansion = 0.0;
        }
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, EDGE);
        v.widgets.noninteractive.bg_fill = PANEL;
        v.widgets.inactive.bg_fill = STONE;
        v.widgets.inactive.weak_bg_fill = STONE;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, EDGE);
        v.widgets.hovered.bg_fill = Color32::from_rgb(0x58, 0x58, 0x5c);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x58, 0x58, 0x5c);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, GREEN_LIGHT);
        v.widgets.active.bg_fill = GREEN;
        v.widgets.active.weak_bg_fill = GREEN;
        v.widgets.active.bg_stroke = Stroke::new(1.0, GREEN_LIGHT);
        v.widgets.open.weak_bg_fill = STONE;
    });
}

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonKind {
    Stone,
    Green,
    Danger,
}

/// A Minecraft-style block button: flat face, light top edge, dark bottom edge.
pub fn block_button(ui: &mut Ui, text: &str, kind: ButtonKind, size: Vec2, font: FontId) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    paint_block_button(ui, rect, &response, text, kind, font);
    response
}

pub fn paint_block_button(ui: &Ui, rect: Rect, response: &Response, text: &str, kind: ButtonKind, font: FontId) {
    let enabled = response.enabled();
    let (face, light, dark) = match kind {
        ButtonKind::Stone => (
            STONE,
            Color32::from_rgb(0x6a, 0x6a, 0x6e),
            Color32::from_rgb(0x2c, 0x2c, 0x2e),
        ),
        ButtonKind::Green => (GREEN, Color32::from_rgb(0x5f, 0xae, 0x34), GREEN_DARK),
        ButtonKind::Danger => (
            RED,
            Color32::from_rgb(0xd9, 0x4a, 0x3f),
            Color32::from_rgb(0x6a, 0x14, 0x10),
        ),
    };
    let hovered = enabled && response.hovered();
    let pressed = enabled && response.is_pointer_button_down_on();
    let face = if !enabled {
        Color32::from_rgb(0x33, 0x33, 0x35)
    } else if pressed {
        dark.lerp_to_gamma(face, 0.6)
    } else if hovered {
        face.lerp_to_gamma(light, 0.35)
    } else {
        face
    };
    let p = ui.painter();
    p.rect_filled(rect, 0.0, face);
    if enabled && !pressed {
        p.rect_filled(Rect::from_min_size(rect.min, Vec2::new(rect.width(), 3.0)), 0.0, light);
        p.rect_filled(
            Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - 4.0), rect.max),
            0.0,
            dark,
        );
    }
    let outline = if hovered { TEXT } else { EDGE };
    p.rect_stroke(rect, 0.0, Stroke::new(2.0, outline), StrokeKind::Inside);
    let color = if enabled { Color32::WHITE } else { MUTED };
    let offset = if pressed {
        Vec2::new(0.0, 1.0)
    } else {
        Vec2::new(0.0, -1.0)
    };
    p.text(rect.center() + offset, egui::Align2::CENTER_CENTER, text, font, color);
}

/// A dark sunken text field frame, like an anvil's name box.
pub fn field_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(FIELD)
        .stroke(Stroke::new(2.0, Color32::from_rgb(0x5a, 0x5a, 0x5e)))
        .inner_margin(6)
}

pub fn section_title(ui: &mut Ui, text: &str) {
    ui.label(egui::RichText::new(text).font(bold_font(13.0)).color(GREEN_LIGHT));
}
