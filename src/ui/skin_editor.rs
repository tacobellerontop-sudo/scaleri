//! The skin editor: paint on a turnable 3D player or on the flat texture, then save the skin to
//! the library or upload it to the signed-in Microsoft account.

use std::collections::{HashMap, HashSet};

use egui::epaint::Vertex;
use egui::{pos2, Color32, Mesh, Pos2, Rect, RichText, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};

use super::theme::{self, block_button, bold_font, section_title, ButtonKind};
use super::App;
use crate::accounts::AccountKind;
use crate::skins::{self, face_area, Face, Part, Skin, SIZE};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Pencil,
    Eraser,
    Fill,
    Picker,
    Shade,
}

impl Tool {
    const ALL: [Tool; 5] = [Tool::Pencil, Tool::Eraser, Tool::Fill, Tool::Picker, Tool::Shade];

    fn name(self) -> &'static str {
        match self {
            Tool::Pencil => "Pencil",
            Tool::Eraser => "Eraser",
            Tool::Fill => "Fill",
            Tool::Picker => "Pick",
            Tool::Shade => "Shade",
        }
    }

    fn key(self) -> egui::Key {
        match self {
            Tool::Pencil => egui::Key::B,
            Tool::Eraser => egui::Key::E,
            Tool::Fill => egui::Key::G,
            Tool::Picker => egui::Key::I,
            Tool::Shade => egui::Key::S,
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Tool::Pencil => "Paint pixels (B)",
            Tool::Eraser => "Make pixels see-through (E)",
            Tool::Fill => "Fill an area of one colour on one face (G)",
            Tool::Picker => "Pick up a colour from the skin (I)",
            Tool::Shade => "Darken pixels; hold Shift to lighten (S)",
        }
    }
}

/// Colours that suit Minecraft skins, from skin tones to wool.
const PALETTE: [u32; 24] = [
    0xf2c9a8, 0xd9a07a, 0xb07650, 0x7a4a2c, 0x4a2c18, 0x2a1a10, 0xffffff, 0xb0b0b4, 0x6a6a70, 0x303034, 0x101012,
    0x000000, 0xd83a2e, 0xf0812a, 0xf7d037, 0x6cc349, 0x2f8f3a, 0x2a9d8f, 0x3a8fd8, 0x2f4b8a, 0x7a3fb0, 0xd85aa0,
    0x8a5a3c, 0xe8e0c8,
];

/// One brush stroke: everything between pressing and releasing the mouse button.
struct BrushStroke {
    last: Option<(usize, usize)>,
    shaded: HashSet<(usize, usize)>,
}

/// A face of the 3D player as drawn on screen, kept for working out which pixel is under the mouse.
struct DrawnFace {
    origin: Pos2,
    u: Vec2,
    v: Vec2,
    area: skins::Area,
    outer: bool,
}

pub struct SkinEditor {
    skin: Skin,
    pub name: String,
    tool: Tool,
    color: Color32,
    recent: Vec<Color32>,
    paint_outer: bool,
    show_outer: bool,
    mirror: bool,
    grid: bool,
    undo: Vec<Skin>,
    redo: Vec<Skin>,
    stroke: Option<BrushStroke>,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    texture: Option<egui::TextureHandle>,
    texture_dirty: bool,
    library: Vec<String>,
    thumbnails: HashMap<String, egui::TextureHandle>,
    /// What's happening in the background, e.g. "Uploading your skin".
    pub busy: Option<&'static str>,
    pub status: Option<(String, bool)>,
}

impl SkinEditor {
    pub fn new() -> Self {
        Self {
            skin: Skin::template(),
            name: "My skin".into(),
            tool: Tool::Pencil,
            color: Color32::from_rgb(0x2a, 0x9d, 0x8f),
            recent: Vec::new(),
            paint_outer: false,
            show_outer: true,
            mirror: false,
            grid: true,
            undo: Vec::new(),
            redo: Vec::new(),
            stroke: None,
            yaw: -0.5,
            pitch: 0.25,
            zoom: 1.0,
            texture: None,
            texture_dirty: true,
            library: skins::library(),
            thumbnails: HashMap::new(),
            busy: None,
            status: None,
        }
    }

    pub fn skin(&self) -> &Skin {
        &self.skin
    }

    /// Replaces the skin being edited; the old one stays reachable with Undo.
    pub fn open(&mut self, skin: Skin, name: &str) {
        self.checkpoint();
        self.skin = skin;
        self.name = name.to_string();
        self.texture_dirty = true;
    }

    fn checkpoint(&mut self) {
        self.undo.push(self.skin.clone());
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn undo(&mut self) {
        if let Some(previous) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.skin, previous));
            self.texture_dirty = true;
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.skin, next));
            self.texture_dirty = true;
        }
    }

    fn remember_color(&mut self) {
        let color = self.color;
        self.recent.retain(|c| *c != color);
        self.recent.insert(0, color);
        self.recent.truncate(6);
    }

    // -----------------------------------------------------------------------------------------
    // Painting

    fn begin_stroke(&mut self) {
        self.checkpoint();
        self.stroke = Some(BrushStroke {
            last: None,
            shaded: HashSet::new(),
        });
    }

    fn end_stroke(&mut self) {
        if self.stroke.take().is_none() {
            return;
        }
        // Nothing changed (e.g. a click on an empty area): drop the undo step
        if self.undo.last() == Some(&self.skin) {
            self.undo.pop();
        } else if matches!(self.tool, Tool::Pencil | Tool::Fill) {
            self.remember_color();
        }
    }

    /// Applies the tool along the line from the previous point of the stroke to (x, y).
    fn stroke_to(&mut self, x: usize, y: usize, shift: bool, connect: bool) {
        let last = self.stroke.as_ref().and_then(|s| s.last);
        if last == Some((x, y)) {
            return;
        }
        let points = match last {
            Some(from) if connect && matches!(self.tool, Tool::Pencil | Tool::Eraser | Tool::Shade) => {
                line(from, (x, y))
            }
            _ => vec![(x, y)],
        };
        for (px, py) in points {
            self.apply(px, py, shift);
        }
        if let Some(stroke) = &mut self.stroke {
            stroke.last = Some((x, y));
        }
    }

    fn apply(&mut self, x: usize, y: usize, shift: bool) {
        if skins::locate(x, y, self.skin.slim).is_none() {
            return;
        }
        let mut targets = vec![(x, y)];
        if self.mirror {
            if let Some(m) = skins::mirror(x, y, self.skin.slim) {
                if m != (x, y) {
                    targets.push(m);
                }
            }
        }
        let rgba = self.color.to_srgba_unmultiplied();
        match self.tool {
            Tool::Pencil => targets.iter().for_each(|&(x, y)| self.skin.set(x, y, rgba)),
            Tool::Eraser => targets.iter().for_each(|&(x, y)| self.skin.set(x, y, [0; 4])),
            Tool::Fill => targets.iter().for_each(|&(x, y)| self.skin.flood_fill(x, y, rgba)),
            Tool::Picker => {
                let [r, g, b, a] = self.skin.get(x, y);
                if a > 0 {
                    self.color = Color32::from_rgb(r, g, b);
                    self.tool = Tool::Pencil;
                }
                return;
            }
            Tool::Shade => {
                for (x, y) in targets {
                    let fresh = self.stroke.as_mut().is_none_or(|s| s.shaded.insert((x, y)));
                    let [r, g, b, a] = self.skin.get(x, y);
                    if !fresh || a == 0 {
                        continue;
                    }
                    let shade = |c: u8| {
                        if shift {
                            (c as f32 + (255.0 - c as f32) * 0.15) as u8
                        } else {
                            (c as f32 * 0.85) as u8
                        }
                    };
                    self.skin.set(x, y, [shade(r), shade(g), shade(b), a]);
                }
            }
        }
        self.texture_dirty = true;
    }

    fn texture(&mut self, ctx: &egui::Context) -> egui::TextureId {
        let image = || egui::ColorImage::from_rgba_unmultiplied([SIZE, SIZE], &self.skin.pixels);
        match &mut self.texture {
            Some(texture) if self.texture_dirty => texture.set(image(), egui::TextureOptions::NEAREST),
            Some(_) => {}
            None => self.texture = Some(ctx.load_texture("skin-editor", image(), egui::TextureOptions::NEAREST)),
        }
        self.texture_dirty = false;
        self.texture.as_ref().expect("texture was just made").id()
    }

    fn thumbnail(&mut self, ctx: &egui::Context, name: &str) -> Option<egui::TextureId> {
        if !self.thumbnails.contains_key(name) {
            let skin = skins::load_from_library(name).ok()?;
            let image = egui::ColorImage::from_rgba_unmultiplied([16, 32], &skin.front_view());
            let texture = ctx.load_texture(format!("skin-{name}"), image, egui::TextureOptions::NEAREST);
            self.thumbnails.insert(name.to_string(), texture);
        }
        self.thumbnails.get(name).map(|t| t.id())
    }

    pub fn refresh_library(&mut self) {
        self.library = skins::library();
        self.thumbnails.clear();
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let (undo, redo, tool) = ctx.input_mut(|i| {
            let redo = i.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::Z)
                || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y);
            let undo = i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z);
            let tool = Tool::ALL
                .into_iter()
                .find(|t| i.consume_key(egui::Modifiers::NONE, t.key()));
            (undo, redo, tool)
        });
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
        if let Some(tool) = tool {
            self.tool = tool;
        }
    }

    /// Opens skin PNGs dropped onto the window.
    fn dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        let Some(file) = dropped.into_iter().next() else {
            return;
        };
        let name = file
            .path()
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Imported skin".into());
        match file.bytes().and_then(|d| Skin::from_png(&d)) {
            Ok(skin) => {
                self.open(skin, &name);
                self.status = Some((format!("Opened {name}"), false));
            }
            Err(e) => self.status = Some((e, true)),
        }
    }
}

/// The pixels on a straight line between two points, both ends included.
fn line(from: (usize, usize), to: (usize, usize)) -> Vec<(usize, usize)> {
    let (mut x, mut y) = (from.0 as i32, from.1 as i32);
    let (x1, y1) = (to.0 as i32, to.1 as i32);
    let dx = (x1 - x).abs();
    let dy = -(y1 - y).abs();
    let (sx, sy) = (if x < x1 { 1 } else { -1 }, if y < y1 { 1 } else { -1 });
    let mut err = dx + dy;
    let mut points = Vec::new();
    loop {
        points.push((x as usize, y as usize));
        if x == x1 && y == y1 {
            return points;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Page layout

pub fn show(app: &mut App, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    app.skins.shortcuts(&ctx);
    app.skins.dropped_files(&ctx);
    if !ui.input(|i| i.pointer.primary_down()) {
        app.skins.end_stroke();
    }

    header(app, ui);
    ui.add_space(10.0);

    let body = ui.available_rect_before_wrap();
    let tools_width = 176.0;
    let flat_px = ((body.height() - 150.0) / SIZE as f32).floor().clamp(3.0, 6.0);
    let flat_width = flat_px * SIZE as f32;
    let tools = Rect::from_min_size(body.min, Vec2::new(tools_width, body.height()));
    let flat = Rect::from_min_max(pos2(body.max.x - flat_width, body.min.y), body.max);
    let model = Rect::from_min_max(
        pos2(tools.max.x + 14.0, body.min.y),
        pos2(flat.min.x - 14.0, body.max.y),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(tools), |ui| {
        tool_panel(&mut app.skins, ui)
    });
    ui.scope_builder(egui::UiBuilder::new().max_rect(model), |ui| {
        model_view(&mut app.skins, ui)
    });
    ui.scope_builder(egui::UiBuilder::new().max_rect(flat), |ui| {
        flat_view(&mut app.skins, ui, flat_px);
        ui.add_space(10.0);
        library(&mut app.skins, ui);
    });
    ui.allocate_rect(body, Sense::hover());

    if ctx.input(|i| !i.raw.hovered_files.is_empty()) {
        let p = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::unique("skin-drop"),
        ));
        p.rect_filled(body, 0.0, Color32::from_black_alpha(170));
        p.rect_stroke(body, 0.0, Stroke::new(3.0, theme::GREEN_LIGHT), StrokeKind::Inside);
        p.text(
            body.center(),
            egui::Align2::CENTER_CENTER,
            "Drop a skin PNG to open it",
            bold_font(22.0),
            Color32::WHITE,
        );
    }
}

fn header(app: &mut App, ui: &mut Ui) {
    let microsoft = app
        .accounts
        .active()
        .filter(|a| a.kind == AccountKind::Microsoft)
        .cloned();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Skin Editor").font(bold_font(22.0)));
        ui.add_space(10.0);
        ui.add(
            egui::TextEdit::singleline(&mut app.skins.name)
                .hint_text("Skin name")
                .desired_width(170.0)
                .margin(Vec2::new(8.0, 6.0)),
        );
        ui.add_space(6.0);
        for (slim, label) in [(false, "Classic arms"), (true, "Slim arms")] {
            let kind = if app.skins.skin.slim == slim {
                ButtonKind::Green
            } else {
                ButtonKind::Stone
            };
            if block_button(ui, label, kind, Vec2::new(108.0, 32.0), bold_font(13.0)).clicked()
                && app.skins.skin.slim != slim
            {
                app.skins.checkpoint();
                app.skins.skin.slim = slim;
            }
        }
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let size = Vec2::new(96.0, 34.0);
        if block_button(ui, "New", ButtonKind::Stone, size, bold_font(13.0))
            .on_hover_text("Start again from a plain skin")
            .clicked()
        {
            app.skins.open(Skin::template(), "My skin");
        }
        let load = ui.add_enabled_ui(microsoft.is_some() && app.skins.busy.is_none(), |ui| {
            block_button(
                ui,
                "Load my skin",
                ButtonKind::Stone,
                Vec2::new(124.0, 34.0),
                bold_font(13.0),
            )
        });
        if load
            .inner
            .on_hover_text("Open the skin your Microsoft account wears now")
            .on_disabled_hover_text("Sign in with a Microsoft account to load your skin")
            .clicked()
        {
            if let Some(account) = &microsoft {
                app.load_current_skin(ui.ctx(), account);
            }
        }
        if block_button(ui, "Save", ButtonKind::Stone, size, bold_font(13.0))
            .on_hover_text("Save to your skin library")
            .clicked()
        {
            match skins::save_to_library(&app.skins.name, &app.skins.skin) {
                Ok(name) => {
                    app.skins.name = name.clone();
                    app.skins.refresh_library();
                    app.skins.status = Some((format!("Saved {name} to your library"), false));
                }
                Err(e) => app.skins.status = Some((format!("Could not save the skin: {e}"), true)),
            }
        }
        if block_button(ui, "Folder", ButtonKind::Stone, size, bold_font(13.0))
            .on_hover_text("Open the folder with your saved skins")
            .clicked()
        {
            crate::paths::open_in_explorer(&skins::library_dir());
        }
        let upload = ui.add_enabled_ui(microsoft.is_some() && app.skins.busy.is_none(), |ui| {
            block_button(
                ui,
                "Wear in Minecraft",
                ButtonKind::Green,
                Vec2::new(170.0, 34.0),
                bold_font(13.0),
            )
        });
        if upload
            .inner
            .on_hover_text("Upload this skin to your Microsoft account")
            .on_disabled_hover_text("Sign in with a Microsoft account to wear your skin in Minecraft")
            .clicked()
        {
            if let Some(account) = &microsoft {
                app.upload_skin(ui.ctx(), account);
            }
        }
        ui.add_space(8.0);
        if let Some(busy) = app.skins.busy {
            ui.spinner();
            ui.label(RichText::new(format!("{busy}…")).color(theme::MUTED));
        } else if let Some((text, is_error)) = &app.skins.status {
            let color = if *is_error { theme::GOLD } else { theme::GREEN_LIGHT };
            ui.label(RichText::new(text).color(color));
        } else if microsoft.is_none() {
            ui.label(
                RichText::new("Offline accounts can save skins, but only Microsoft accounts can wear them.")
                    .small()
                    .color(theme::MUTED),
            );
        }
    });
}

fn tool_panel(editor: &mut SkinEditor, ui: &mut Ui) {
    egui::ScrollArea::vertical()
        .id_salt("skin-tools")
        .auto_shrink([false, false])
        .show(ui, |ui| tool_list(editor, ui));
}

fn tool_list(editor: &mut SkinEditor, ui: &mut Ui) {
    section_title(ui, "TOOLS");
    egui::Grid::new("skin-tools").spacing(Vec2::splat(6.0)).show(ui, |ui| {
        for (i, tool) in Tool::ALL.into_iter().enumerate() {
            let kind = if editor.tool == tool {
                ButtonKind::Green
            } else {
                ButtonKind::Stone
            };
            if block_button(ui, tool.name(), kind, Vec2::new(82.0, 30.0), bold_font(13.0))
                .on_hover_text(tool.hint())
                .clicked()
            {
                editor.tool = tool;
            }
            if i % 2 == 1 {
                ui.end_row();
            }
        }
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let undo = ui.add_enabled(!editor.undo.is_empty(), egui::Button::new("Undo"));
        if undo.on_hover_text("Ctrl+Z").clicked() {
            editor.undo();
        }
        let redo = ui.add_enabled(!editor.redo.is_empty(), egui::Button::new("Redo"));
        if redo.on_hover_text("Ctrl+Y").clicked() {
            editor.redo();
        }
    });

    ui.add_space(8.0);
    section_title(ui, "COLOUR");
    ui.horizontal(|ui| {
        egui::color_picker::color_edit_button_srgba(ui, &mut editor.color, egui::color_picker::Alpha::Opaque);
        let [r, g, b, _] = editor.color.to_array();
        ui.label(
            RichText::new(format!("#{r:02X}{g:02X}{b:02X}"))
                .monospace()
                .color(theme::MUTED),
        );
    });
    ui.add_space(4.0);
    swatches(ui, &PALETTE.map(rgb), &mut editor.color);
    if !editor.recent.is_empty() {
        ui.add_space(4.0);
        ui.label(RichText::new("Recent").small().color(theme::MUTED));
        let recent = editor.recent.clone();
        swatches(ui, &recent, &mut editor.color);
    }

    ui.add_space(8.0);
    section_title(ui, "LAYER");
    ui.horizontal(|ui| {
        let hint = "The outer layer is the hat, jacket, sleeves and trousers";
        ui.radio_value(&mut editor.paint_outer, false, "Base")
            .on_hover_text(hint);
        ui.radio_value(&mut editor.paint_outer, true, "Outer")
            .on_hover_text(hint);
    });
    if editor.paint_outer {
        editor.show_outer = true;
    }
    ui.add_enabled(
        !editor.paint_outer,
        egui::Checkbox::new(&mut editor.show_outer, "Show outer layer"),
    );
    ui.checkbox(&mut editor.mirror, "Mirror left and right")
        .on_hover_text("Paint both arms, both legs and both sides of the head and body at once");
    ui.checkbox(&mut editor.grid, "Pixel grid");
}

fn swatches(ui: &mut Ui, colors: &[Color32], selected: &mut Color32) {
    let size = 22.0;
    let per_row = 6;
    for row in colors.chunks(per_row) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for &color in row {
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
                ui.painter().rect_filled(rect, 0.0, color);
                let stroke = if *selected == color {
                    Stroke::new(2.0, Color32::WHITE)
                } else if response.hovered() {
                    Stroke::new(2.0, theme::MUTED)
                } else {
                    Stroke::new(1.0, theme::EDGE)
                };
                ui.painter().rect_stroke(rect, 0.0, stroke, StrokeKind::Inside);
                if response.clicked() {
                    *selected = color;
                }
            }
        });
        ui.add_space(-2.0);
    }
}

fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

// ---------------------------------------------------------------------------------------------
// The 3D player

fn model_view(editor: &mut SkinEditor, ui: &mut Ui) {
    let full = ui.available_rect_before_wrap();
    let rect = Rect::from_min_max(full.min, pos2(full.max.x, full.max.y - 40.0));
    let response = ui.allocate_rect(rect, Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(0x16, 0x1a, 0x17));
    // A grass floor for the player to stand on
    let floor = Rect::from_min_max(pos2(rect.min.x, rect.max.y - 34.0), rect.max);
    painter.rect_filled(floor, 0.0, Color32::from_rgb(0x2a, 0x3d, 0x22));
    painter.rect_filled(
        Rect::from_min_size(floor.min, Vec2::new(floor.width(), 4.0)),
        0.0,
        Color32::from_rgb(0x4f, 0x8a, 0x34),
    );
    painter.rect_stroke(rect, 0.0, Stroke::new(2.0, theme::EDGE), StrokeKind::Inside);
    painter.text(
        rect.min + Vec2::new(10.0, 10.0),
        egui::Align2::LEFT_TOP,
        "Click to paint · right-drag to turn · scroll to zoom",
        egui::FontId::proportional(11.0),
        theme::MUTED,
    );

    // Right-drag (or middle-drag) turns the player; the wheel zooms
    if response.dragged_by(egui::PointerButton::Secondary) || response.dragged_by(egui::PointerButton::Middle) {
        let delta = response.drag_delta();
        editor.yaw += delta.x * 0.012;
        editor.pitch = (editor.pitch + delta.y * 0.012).clamp(-1.2, 1.2);
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        editor.zoom = (editor.zoom * (1.0 + scroll * 0.002)).clamp(0.5, 3.0);
    }

    let texture = editor.texture(ui.ctx());
    let scale = (rect.height() / 40.0).min(rect.width() / 22.0) * editor.zoom;
    let center = pos2(rect.center().x, rect.center().y + 6.0);
    let faces = build_model(editor, center, scale);

    let mut mesh = Mesh::with_texture(texture);
    for (face, light) in &faces {
        let uv = |x: usize, y: usize| pos2(x as f32 / SIZE as f32, y as f32 / SIZE as f32);
        let a = face.area;
        let tint = Color32::from_gray((light * 255.0) as u8);
        let base = mesh.vertices.len() as u32;
        for (pos, uv) in [
            (face.origin, uv(a.x, a.y)),
            (face.origin + face.u, uv(a.x + a.w, a.y)),
            (face.origin + face.u + face.v, uv(a.x + a.w, a.y + a.h)),
            (face.origin + face.v, uv(a.x, a.y + a.h)),
        ] {
            mesh.vertices.push(Vertex { pos, uv, color: tint });
        }
        mesh.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    painter.add(Shape::mesh(mesh));

    // Painting
    let hover = response.hover_pos().or(response.interact_pointer_pos());
    let hit = hover.and_then(|p| pick(&faces, p, editor.paint_outer));
    if let Some((face, col, row)) = &hit {
        let corner =
            |c: f32, r: f32| face.origin + face.u * (c / face.area.w as f32) + face.v * (r / face.area.h as f32);
        let (c, r) = (*col as f32, *row as f32);
        painter.add(Shape::closed_line(
            vec![
                corner(c, r),
                corner(c + 1.0, r),
                corner(c + 1.0, r + 1.0),
                corner(c, r + 1.0),
            ],
            Stroke::new(1.5, Color32::WHITE),
        ));
    }
    let primary = ui.input(|i| i.pointer.primary_down());
    if primary && response.is_pointer_button_down_on() {
        if editor.stroke.is_none() {
            editor.begin_stroke();
        }
        if let Some((face, col, row)) = hit {
            let shift = ui.input(|i| i.modifiers.shift);
            editor.stroke_to(face.area.x + col, face.area.y + row, shift, false);
        }
    }

    // View buttons under the player
    let buttons = Rect::from_min_max(pos2(full.min.x, rect.max.y + 6.0), full.max);
    ui.scope_builder(egui::UiBuilder::new().max_rect(buttons), |ui| {
        ui.horizontal(|ui| {
            for (label, yaw) in [
                ("Front", 0.0),
                ("Side", -std::f32::consts::FRAC_PI_2),
                ("Back", std::f32::consts::PI),
            ] {
                if ui.button(label).clicked() {
                    editor.yaw = yaw;
                    editor.pitch = 0.0;
                }
            }
            if ui.button("Reset view").clicked() {
                editor.yaw = -0.5;
                editor.pitch = 0.25;
                editor.zoom = 1.0;
            }
        });
    });
}

/// Projects every visible face of the player onto the screen, back to front, with its lighting.
fn build_model(editor: &SkinEditor, center: Pos2, scale: f32) -> Vec<(DrawnFace, f32)> {
    let slim = editor.skin.slim;
    let (sy, cy) = editor.yaw.sin_cos();
    let (sp, cp) = editor.pitch.sin_cos();
    let rotate = |[x, y, z]: [f32; 3]| {
        let (x1, z1) = (x * cy + z * sy, -x * sy + z * cy);
        [x1, y * cp - z1 * sp, y * sp + z1 * cp]
    };
    let project = |p: [f32; 3]| {
        let [x, y, _] = rotate(p);
        pos2(center.x + x * scale, center.y - (y - 16.0) * scale)
    };

    let mut faces = Vec::new();
    for outer in [false, true] {
        if outer && !editor.show_outer {
            continue;
        }
        for part in Part::ALL {
            let [w, h, d] = part.size(slim).map(|v| v as f32);
            let grow = match (outer, part) {
                (false, _) => 0.0,
                (true, Part::Head) => 0.5,
                (true, _) => 0.25,
            };
            let (hx, hy, hz) = (w / 2.0 + grow, h / 2.0 + grow, d / 2.0 + grow);
            let c = part.center(slim);
            for face in Face::ALL {
                // Texture top-left corner, the texture's x and y directions, and the outward normal
                let (o, u, v, n) = match face {
                    Face::Front => (
                        [-hx, hy, hz],
                        [2.0 * hx, 0.0, 0.0],
                        [0.0, -2.0 * hy, 0.0],
                        [0.0, 0.0, 1.0],
                    ),
                    Face::Back => (
                        [hx, hy, -hz],
                        [-2.0 * hx, 0.0, 0.0],
                        [0.0, -2.0 * hy, 0.0],
                        [0.0, 0.0, -1.0],
                    ),
                    Face::Right => (
                        [-hx, hy, -hz],
                        [0.0, 0.0, 2.0 * hz],
                        [0.0, -2.0 * hy, 0.0],
                        [-1.0, 0.0, 0.0],
                    ),
                    Face::Left => (
                        [hx, hy, hz],
                        [0.0, 0.0, -2.0 * hz],
                        [0.0, -2.0 * hy, 0.0],
                        [1.0, 0.0, 0.0],
                    ),
                    Face::Top => (
                        [-hx, hy, -hz],
                        [2.0 * hx, 0.0, 0.0],
                        [0.0, 0.0, 2.0 * hz],
                        [0.0, 1.0, 0.0],
                    ),
                    Face::Bottom => (
                        [-hx, -hy, hz],
                        [2.0 * hx, 0.0, 0.0],
                        [0.0, 0.0, -2.0 * hz],
                        [0.0, -1.0, 0.0],
                    ),
                };
                let normal = rotate(n);
                if normal[2] <= 0.0 {
                    continue;
                }
                let at = |p: [f32; 3]| [c[0] + p[0], c[1] + p[1], c[2] + p[2]];
                let corner = at(o);
                let origin = project(corner);
                let u_end = project(at(add(o, u)));
                let v_end = project(at(add(o, v)));
                let middle = at(add(o, add(scale3(u, 0.5), scale3(v, 0.5))));
                // Outer faces sit a little in front of the base face they cover
                let depth = rotate(middle)[2] + if outer { 0.01 } else { 0.0 };
                let light = 0.55 + 0.45 * (normal[2] * 0.8 + normal[1] * 0.4).clamp(0.0, 1.0);
                faces.push((
                    depth,
                    DrawnFace {
                        origin,
                        u: u_end - origin,
                        v: v_end - origin,
                        area: face_area(part, face, outer, slim),
                        outer,
                    },
                    light,
                ));
            }
        }
    }
    faces.sort_by(|a, b| a.0.total_cmp(&b.0));
    faces.into_iter().map(|(_, face, light)| (face, light)).collect()
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// The pixel under `pos` on the front-most face of the layer being painted.
fn pick(faces: &[(DrawnFace, f32)], pos: Pos2, outer: bool) -> Option<(&DrawnFace, usize, usize)> {
    for (face, _) in faces.iter().rev() {
        let det = face.u.x * face.v.y - face.u.y * face.v.x;
        if det.abs() < 1e-3 {
            continue;
        }
        let d = pos - face.origin;
        let a = (d.x * face.v.y - d.y * face.v.x) / det;
        let b = (face.u.x * d.y - face.u.y * d.x) / det;
        if !(0.0..1.0).contains(&a) || !(0.0..1.0).contains(&b) {
            continue;
        }
        if face.outer && !outer {
            // Painting the base: reach through the outer layer
            continue;
        }
        if !face.outer && outer {
            // Another part's base is in front of the outer layer here
            return None;
        }
        let col = ((a * face.area.w as f32) as usize).min(face.area.w - 1);
        let row = ((b * face.area.h as f32) as usize).min(face.area.h - 1);
        return Some((face, col, row));
    }
    None
}

// ---------------------------------------------------------------------------------------------
// The flat texture

fn flat_view(editor: &mut SkinEditor, ui: &mut Ui, px: f32) {
    section_title(ui, "TEXTURE");
    let size = Vec2::splat(px * SIZE as f32);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, theme::FIELD);
    let slim = editor.skin.slim;
    let cell = |x: usize, y: usize| Rect::from_min_size(rect.min + Vec2::new(x as f32, y as f32) * px, Vec2::splat(px));

    // A checkerboard behind the faces shows which pixels are see-through
    for (_, _, outer, area) in skins::all_faces(slim) {
        let r = Rect::from_min_max(
            cell(area.x, area.y).min,
            cell(area.x + area.w - 1, area.y + area.h - 1).max,
        );
        painter.rect_filled(
            r,
            0.0,
            if outer {
                Color32::from_gray(52)
            } else {
                Color32::from_gray(64)
            },
        );
        for y in area.y..area.y + area.h {
            for x in area.x..area.x + area.w {
                if (x + y) % 2 == 0 {
                    painter.rect_filled(cell(x, y), 0.0, Color32::from_gray(if outer { 40 } else { 48 }));
                }
            }
        }
    }
    let texture = editor.texture(ui.ctx());
    painter.image(
        texture,
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    if editor.grid && px >= 4.0 {
        let stroke = Stroke::new(1.0, Color32::from_black_alpha(50));
        for i in 1..SIZE {
            let o = i as f32 * px;
            painter.line_segment(
                [pos2(rect.min.x + o, rect.min.y), pos2(rect.min.x + o, rect.max.y)],
                stroke,
            );
            painter.line_segment(
                [pos2(rect.min.x, rect.min.y + o), pos2(rect.max.x, rect.min.y + o)],
                stroke,
            );
        }
    }
    for (_, _, outer, area) in skins::all_faces(slim) {
        if outer == editor.paint_outer {
            let r = Rect::from_min_max(
                cell(area.x, area.y).min,
                cell(area.x + area.w - 1, area.y + area.h - 1).max,
            );
            painter.rect_stroke(
                r,
                0.0,
                Stroke::new(1.0, Color32::from_white_alpha(40)),
                StrokeKind::Inside,
            );
        }
    }
    painter.rect_stroke(rect, 0.0, Stroke::new(2.0, theme::EDGE), StrokeKind::Outside);

    let texel = |pos: Pos2| {
        let local = (pos - rect.min) / px;
        (local.x >= 0.0 && local.y >= 0.0 && local.x < SIZE as f32 && local.y < SIZE as f32)
            .then_some((local.x as usize, local.y as usize))
    };
    let pointer = response.hover_pos().or(response.interact_pointer_pos());
    let hovered = pointer.and_then(texel);
    if let Some((x, y)) = hovered {
        if skins::locate(x, y, slim).is_some() {
            painter.rect_stroke(cell(x, y), 0.0, Stroke::new(1.5, Color32::WHITE), StrokeKind::Outside);
            if editor.mirror {
                if let Some((mx, my)) = skins::mirror(x, y, slim) {
                    painter.rect_stroke(cell(mx, my), 0.0, Stroke::new(1.0, theme::GOLD), StrokeKind::Outside);
                }
            }
        }
    }
    let primary = ui.input(|i| i.pointer.primary_down());
    if primary && response.is_pointer_button_down_on() {
        if editor.stroke.is_none() {
            editor.begin_stroke();
        }
        if let Some((x, y)) = hovered {
            let shift = ui.input(|i| i.modifiers.shift);
            editor.stroke_to(x, y, shift, true);
        }
    }
}

fn library(editor: &mut SkinEditor, ui: &mut Ui) {
    section_title(ui, "YOUR SKINS");
    if editor.library.is_empty() {
        ui.label(
            RichText::new("Skins you save show up here. You can also drop a skin PNG onto this page.")
                .small()
                .color(theme::MUTED),
        );
        return;
    }
    let names = editor.library.clone();
    let mut open = None;
    let mut delete = None;
    egui::ScrollArea::horizontal().id_salt("skin-library").show(ui, |ui| {
        ui.horizontal(|ui| {
            for name in &names {
                let (rect, response) = ui.allocate_exact_size(Vec2::new(64.0, 96.0), Sense::click());
                let p = ui.painter();
                p.rect_filled(rect, 0.0, if response.hovered() { theme::STONE } else { theme::FIELD });
                p.rect_stroke(rect, 0.0, Stroke::new(1.0, theme::EDGE), StrokeKind::Inside);
                if let Some(texture) = editor.thumbnail(ui.ctx(), name) {
                    let figure =
                        Rect::from_center_size(pos2(rect.center().x, rect.min.y + 38.0), Vec2::new(30.0, 60.0));
                    ui.painter().image(
                        texture,
                        figure,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                let label = ui
                    .painter()
                    .layout(name.clone(), egui::FontId::proportional(11.0), theme::TEXT, 60.0);
                let pos = pos2(rect.center().x - label.size().x / 2.0, rect.max.y - 22.0);
                ui.painter().with_clip_rect(rect).galley(pos, label, theme::TEXT);
                let response = response.on_hover_text(format!("Open {name}"));
                if response.clicked() {
                    open = Some(name.clone());
                }
                response.context_menu(|ui| {
                    if ui.button("Open").clicked() {
                        open = Some(name.clone());
                        ui.close();
                    }
                    if ui
                        .button(RichText::new("Delete").color(Color32::from_rgb(0xff, 0x8a, 0x80)))
                        .clicked()
                    {
                        delete = Some(name.clone());
                        ui.close();
                    }
                });
            }
        });
    });
    if let Some(name) = open {
        match skins::load_from_library(&name) {
            Ok(skin) => {
                editor.open(skin, &name);
                editor.status = Some((format!("Opened {name}"), false));
            }
            Err(e) => editor.status = Some((e, true)),
        }
    }
    if let Some(name) = delete {
        if let Err(e) = skins::delete_from_library(&name) {
            editor.status = Some((format!("Could not delete {name}: {e}"), true));
        }
        editor.refresh_library();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_include_both_ends() {
        assert_eq!(line((0, 0), (3, 0)), [(0, 0), (1, 0), (2, 0), (3, 0)]);
        assert_eq!(line((2, 2), (0, 0)), [(2, 2), (1, 1), (0, 0)]);
        assert_eq!(line((5, 5), (5, 5)), [(5, 5)]);
    }

    #[test]
    fn mirror_paints_both_hands() {
        let mut editor = SkinEditor::new();
        editor.mirror = true;
        editor.color = Color32::from_rgb(1, 2, 3);
        editor.begin_stroke();
        editor.stroke_to(44, 20, false, true);
        editor.end_stroke();
        assert_eq!(editor.skin.get(44, 20), [1, 2, 3, 255]);
        assert_eq!(editor.skin.get(39, 52), [1, 2, 3, 255]);
        editor.undo();
        assert_ne!(editor.skin.get(44, 20), [1, 2, 3, 255]);
        assert_eq!(editor.recent.first(), Some(&Color32::from_rgb(1, 2, 3)));
    }

    #[test]
    fn the_front_of_the_head_faces_the_viewer() {
        let mut editor = SkinEditor::new();
        editor.yaw = 0.0;
        editor.pitch = 0.0;
        let faces = build_model(&editor, pos2(0.0, 0.0), 10.0);
        // Straight on, only front faces are visible: 6 parts, two layers
        assert_eq!(faces.len(), 12);
        let (face, col, row) = pick(&faces, pos2(-35.0, -155.0), false).unwrap();
        assert_eq!((face.area.x + col, face.area.y + row), (8, 8));
    }
}
