//! The Scaleri window: sidebar, hero banner with the Play button, and the instance library.

mod art;
mod dialogs;
pub mod theme;

use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{
    pos2, Align, Color32, Id, Layout, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, Ui, UiBuilder, Vec2,
};

use crate::accounts::{self, Account, AccountKind, Accounts, DeviceCode};
use crate::instances::{self, Instance};
use crate::net::{Progress, Result};
use crate::settings::Settings;
use crate::versions::{self, Manifest};
use crate::{launch, paths};
use dialogs::Dialog;
use theme::{block_button, bold_font, ButtonKind};

/// Results that background threads send back to the window.
pub enum Msg {
    Manifest(Result<Manifest>),
    DeviceCode(Result<DeviceCode>),
    SignedIn(Result<Account>),
    AccountRefreshed(Account),
    Launched { instance: String, result: Result<()> },
    GameExited { instance: String, seconds: u64 },
}

pub enum ManifestState {
    Loading,
    Ready(Manifest),
    Failed(String),
}

/// A game launch in progress.
pub struct Task {
    pub instance: String,
    pub progress: Arc<Progress>,
}

/// Microsoft sign-in in progress.
pub struct Login {
    pub code: Option<DeviceCode>,
    pub progress: Arc<Progress>,
    pub error: Option<String>,
}

pub struct App {
    pub settings: Settings,
    pub accounts: Accounts,
    pub instances: Vec<Instance>,
    pub selected: Option<String>,
    pub search: String,
    pub manifest: ManifestState,
    pub task: Option<Task>,
    pub login: Option<Login>,
    pub dialog: Option<Dialog>,
    pub notice: Option<(String, bool)>,
    pub running: HashSet<String>,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    logo: TextureHandle,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let (tx, rx) = channel();
        let logo = load_logo(&cc.egui_ctx);
        let instances = instances::load_all();
        let app = Self {
            settings: Settings::load(),
            accounts: Accounts::load(),
            selected: instances.first().map(|i| i.id.clone()),
            instances,
            search: String::new(),
            manifest: ManifestState::Loading,
            task: None,
            login: None,
            dialog: None,
            notice: None,
            running: HashSet::new(),
            tx,
            rx,
            logo,
        };
        app.spawn(&cc.egui_ctx, || Msg::Manifest(versions::load_manifest()));
        app
    }

    /// Runs `job` on a background thread and delivers its result to the window.
    pub fn spawn(&self, ctx: &egui::Context, job: impl FnOnce() -> Msg + Send + 'static) {
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(job());
            ctx.request_repaint();
        });
    }

    pub fn reload_manifest(&mut self, ctx: &egui::Context) {
        self.manifest = ManifestState::Loading;
        self.spawn(ctx, || Msg::Manifest(versions::load_manifest()));
    }

    fn handle_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Manifest(Ok(manifest)) => self.manifest = ManifestState::Ready(manifest),
                Msg::Manifest(Err(e)) => self.manifest = ManifestState::Failed(e),
                Msg::DeviceCode(result) => self.on_device_code(ctx, result),
                Msg::SignedIn(result) => match result {
                    Ok(account) => {
                        self.notice = Some((format!("Signed in as {}", account.name), false));
                        self.accounts.upsert(account);
                        self.accounts.save();
                        self.login = None;
                    }
                    Err(e) => {
                        if let Some(login) = &mut self.login {
                            if login.progress.cancel.load(Ordering::Relaxed) {
                                self.login = None;
                            } else {
                                login.error = Some(e);
                                login.code = None;
                            }
                        }
                    }
                },
                Msg::AccountRefreshed(account) => {
                    self.accounts.upsert(account);
                    self.accounts.save();
                }
                Msg::Launched { instance, result } => {
                    self.task = None;
                    match result {
                        Ok(()) => {
                            if let Some(inst) = self.instances.iter_mut().find(|i| i.id == instance) {
                                inst.last_played = Some(instances::now());
                                let _ = inst.save();
                            }
                            if self.settings.close_on_launch {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            } else {
                                self.running.insert(instance);
                            }
                        }
                        Err(e) if e == "Cancelled" => {}
                        Err(e) => self.notice = Some((e, true)),
                    }
                }
                Msg::GameExited { instance, seconds } => {
                    self.running.remove(&instance);
                    if let Some(inst) = self.instances.iter_mut().find(|i| i.id == instance) {
                        inst.play_seconds += seconds;
                        let _ = inst.save();
                    }
                }
            }
        }
    }

    pub fn start_sign_in(&mut self, ctx: &egui::Context) {
        let Some(client_id) = self.settings.client_id() else {
            self.login = Some(Login {
                code: None,
                progress: Arc::default(),
                error: Some(
                    "Microsoft sign-in needs an Azure application ID. Add one under Settings → Microsoft sign-in."
                        .into(),
                ),
            });
            return;
        };
        self.login = Some(Login {
            code: None,
            progress: Arc::default(),
            error: None,
        });
        self.spawn(ctx, move || Msg::DeviceCode(accounts::start_device_code(&client_id)));
    }

    fn on_device_code(&mut self, ctx: &egui::Context, result: Result<DeviceCode>) {
        let Some(login) = &mut self.login else { return };
        match result {
            Ok(code) => {
                paths::open_url(&code.verification_uri);
                login.code = Some(code.clone());
                let progress = login.progress.clone();
                let client_id = self.settings.client_id().unwrap_or_default();
                self.spawn(ctx, move || {
                    Msg::SignedIn(accounts::finish_device_code(&client_id, &code, &progress))
                });
            }
            Err(e) => login.error = Some(e),
        }
    }

    pub fn play(&mut self, ctx: &egui::Context, id: &str) {
        if self.task.is_some() || self.running.contains(id) {
            return;
        }
        let Some(instance) = self.instances.iter().find(|i| i.id == id).cloned() else {
            return;
        };
        let Some(account) = self.accounts.active().cloned() else {
            self.dialog = Some(Dialog::accounts(Some("Add an account to play.")));
            return;
        };
        if account.kind == AccountKind::Offline && !self.accounts.has_microsoft() {
            self.dialog = Some(Dialog::accounts(Some(
                "Offline play needs a Microsoft account that owns Minecraft. Add one first.",
            )));
            return;
        }
        let client_id = self.settings.client_id();
        if account.needs_refresh() && client_id.is_none() {
            self.notice = Some((
                "Your sign-in expired and no Microsoft application ID is set (Settings).".into(),
                true,
            ));
            return;
        }

        let progress = Arc::new(Progress::default());
        self.task = Some(Task {
            instance: id.to_string(),
            progress: progress.clone(),
        });
        self.notice = None;
        let settings = self.settings.clone();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let id = id.to_string();
        std::thread::spawn(move || {
            let send = |msg| {
                let _ = tx.send(msg);
                ctx.request_repaint();
            };
            let mut account = account;
            if account.needs_refresh() {
                progress.set_stage("Refreshing your Microsoft sign-in", 0);
                match accounts::refresh(&client_id.unwrap_or_default(), &account) {
                    Ok(fresh) => {
                        account = fresh.clone();
                        send(Msg::AccountRefreshed(fresh));
                    }
                    Err(e) => {
                        send(Msg::Launched {
                            instance: id,
                            result: Err(e),
                        });
                        return;
                    }
                }
            }
            match launch::launch(&instance, &account, &settings, &progress) {
                Ok(mut child) => {
                    let started = Instant::now();
                    send(Msg::Launched {
                        instance: id.clone(),
                        result: Ok(()),
                    });
                    if !settings.close_on_launch {
                        let _ = child.wait();
                        send(Msg::GameExited {
                            instance: id,
                            seconds: started.elapsed().as_secs(),
                        });
                    }
                }
                Err(e) => send(Msg::Launched {
                    instance: id,
                    result: Err(e),
                }),
            }
        });
    }

    pub fn selected_instance(&self) -> Option<&Instance> {
        let id = self.selected.as_deref()?;
        self.instances.iter().find(|i| i.id == id)
    }

    pub fn add_instance(&mut self, instance: Instance) {
        self.selected = Some(instance.id.clone());
        self.instances.push(instance);
        instances::sort(&mut self.instances);
    }

    pub fn remove_instance(&mut self, id: &str) {
        if let Some(inst) = self.instances.iter().find(|i| i.id == id) {
            if let Err(e) = inst.delete() {
                self.notice = Some((format!("Could not delete {}: {e}", inst.name), true));
                return;
            }
        }
        self.instances.retain(|i| i.id != id);
        if self.selected.as_deref() == Some(id) {
            self.selected = self.instances.first().map(|i| i.id.clone());
        }
    }

    // -----------------------------------------------------------------------------------------
    // Layout

    fn sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.add(egui::Image::new(&self.logo).fit_to_exact_size(Vec2::splat(36.0)));
            ui.add_space(4.0);
            ui.label(RichText::new("SCALERI").font(bold_font(21.0)).color(theme::GREEN_LIGHT));
        });
        ui.add_space(18.0);

        let nav = [
            (Some(art::Glyph::Plus), "New Instance"),
            (Some(art::Glyph::Gear), "Settings"),
            (None, "Accounts"),
            (Some(art::Glyph::Chest), "Open Folder"),
        ];
        for (icon, label) in nav {
            if sidebar_button(ui, icon, label).clicked() {
                match label {
                    "New Instance" => self.open_new_instance(ui.ctx()),
                    "Settings" => self.dialog = Some(Dialog::Settings(Box::new(self.settings.clone()))),
                    "Accounts" => self.dialog = Some(Dialog::accounts(None)),
                    _ => paths::open_in_explorer(paths::data_dir()),
                }
            }
        }

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new(concat!("Scaleri ", env!("CARGO_PKG_VERSION")))
                    .small()
                    .color(theme::MUTED),
            );
            ui.add_space(4.0);
            if self.account_chip(ui).clicked() {
                self.dialog = Some(Dialog::accounts(None));
            }
        });
    }

    fn account_chip(&self, ui: &mut Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 52.0), Sense::click());
        let p = ui.painter();
        let fill = if response.hovered() { theme::STONE } else { theme::PANEL };
        p.rect_filled(rect, 0.0, fill);
        p.rect_stroke(rect, 0.0, Stroke::new(2.0, theme::EDGE), StrokeKind::Inside);
        let head = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::splat(32.0));
        match self.accounts.active() {
            Some(account) => {
                art::paint_head(p, head, &account.name);
                p.text(
                    pos2(head.max.x + 10.0, rect.min.y + 17.0),
                    egui::Align2::LEFT_CENTER,
                    &account.name,
                    bold_font(14.0),
                    theme::TEXT,
                );
                let kind = if account.kind == AccountKind::Microsoft {
                    "Microsoft account"
                } else {
                    "Offline"
                };
                p.text(
                    pos2(head.max.x + 10.0, rect.min.y + 35.0),
                    egui::Align2::LEFT_CENTER,
                    kind,
                    egui::FontId::proportional(12.0),
                    theme::MUTED,
                );
            }
            None => {
                p.rect_filled(head, 0.0, theme::FIELD);
                p.text(
                    head.center(),
                    egui::Align2::CENTER_CENTER,
                    "?",
                    bold_font(18.0),
                    theme::MUTED,
                );
                p.text(
                    pos2(head.max.x + 10.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Add an account",
                    bold_font(14.0),
                    theme::TEXT,
                );
            }
        }
        response
    }

    pub fn open_new_instance(&mut self, ctx: &egui::Context) {
        if matches!(self.manifest, ManifestState::Failed(_)) {
            self.reload_manifest(ctx);
        }
        self.dialog = Some(Dialog::NewInstance(Default::default()));
    }

    fn hero(&mut self, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 270.0), Sense::hover());
        art::paint_landscape(ui.painter(), rect);
        let inner = rect.shrink2(Vec2::new(32.0, 26.0));
        let mut content = ui.new_child(UiBuilder::new().max_rect(inner).layout(Layout::top_down(Align::Min)));
        let ui = &mut content;

        let Some(instance) = self.selected_instance().cloned() else {
            ui.add_space(30.0);
            ui.label(
                RichText::new("WELCOME TO SCALERI")
                    .font(bold_font(13.0))
                    .color(theme::GREEN_LIGHT),
            );
            ui.label(
                RichText::new("Let's get you into a world")
                    .font(bold_font(30.0))
                    .color(Color32::WHITE),
            );
            ui.label(
                RichText::new("Create an instance to pick a Minecraft version, then press Play.").color(theme::MUTED),
            );
            ui.add_space(18.0);
            if block_button(
                ui,
                "Create an instance",
                ButtonKind::Green,
                Vec2::new(230.0, 48.0),
                bold_font(17.0),
            )
            .clicked()
            {
                self.open_new_instance(ui.ctx());
            }
            return;
        };

        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(104.0), Sense::hover());
            ui.painter().rect_filled(icon_rect, 0.0, Color32::from_black_alpha(120));
            ui.painter()
                .rect_stroke(icon_rect, 0.0, Stroke::new(2.0, theme::EDGE), StrokeKind::Inside);
            art::paint_block_icon(ui.painter(), icon_rect.shrink(12.0), &instance.icon);
            ui.add_space(14.0);
            ui.vertical(|ui| {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("SELECTED INSTANCE")
                        .font(bold_font(12.0))
                        .color(theme::GREEN_LIGHT),
                );
                ui.label(
                    RichText::new(&instance.name)
                        .font(bold_font(30.0))
                        .color(Color32::WHITE),
                );
                ui.label(RichText::new(format!("Minecraft {}", instance.version)).color(theme::TEXT));
                let status = if self.running.contains(&instance.id) {
                    "Playing now".to_string()
                } else {
                    instance.playtime_text()
                };
                ui.label(RichText::new(status).color(theme::MUTED));
            });
        });

        ui.add_space(ui.available_height() - 52.0);
        ui.horizontal(|ui| {
            let busy = self.task.as_ref().map(|t| t.instance == instance.id).unwrap_or(false);
            if busy {
                self.progress_bar(ui);
            } else {
                let playing = self.running.contains(&instance.id);
                let label = if playing { "Playing" } else { "Play" };
                let enabled = self.task.is_none() && !playing;
                let play = ui.add_enabled_ui(enabled, |ui| {
                    block_button(ui, label, ButtonKind::Green, Vec2::new(210.0, 50.0), bold_font(20.0))
                });
                if play.inner.clicked() {
                    self.play(ui.ctx(), &instance.id);
                }
                ui.add_space(6.0);
                if block_button(ui, "Edit", ButtonKind::Stone, Vec2::new(90.0, 50.0), bold_font(14.0)).clicked() {
                    self.dialog = Some(Dialog::edit(&instance));
                }
                if block_button(ui, "Folder", ButtonKind::Stone, Vec2::new(90.0, 50.0), bold_font(14.0)).clicked() {
                    paths::open_in_explorer(&instance.game_dir());
                }
            }
        });
    }

    /// An XP-bar style progress bar with the current step and a Cancel button.
    fn progress_bar(&mut self, ui: &mut Ui) {
        let Some(task) = &self.task else { return };
        let progress = task.progress.clone();
        ui.vertical(|ui| {
            let label = progress.label.lock().unwrap().clone();
            let total = progress.total.load(Ordering::Relaxed);
            let text = if total > 0 {
                format!("{label}  {} / {total}", progress.done.load(Ordering::Relaxed))
            } else {
                format!("{label}…")
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(text).font(bold_font(14.0)).color(Color32::WHITE));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.set_max_width(120.0);
                    if block_button(ui, "Cancel", ButtonKind::Stone, Vec2::new(90.0, 26.0), bold_font(13.0)).clicked() {
                        progress.cancel.store(true, Ordering::Relaxed);
                    }
                });
            });
            let width = (ui.available_width() - 130.0).max(200.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 14.0), Sense::hover());
            let p = ui.painter();
            p.rect_filled(rect, 0.0, theme::FIELD);
            let fraction = progress.fraction().unwrap_or_else(|| {
                // Indeterminate: a short bar sweeping back and forth
                let t = ui.input(|i| i.time) as f32;
                (t * 0.8).sin().abs()
            });
            let fill = Rect::from_min_size(rect.min, Vec2::new(rect.width() * fraction, rect.height()));
            p.rect_filled(fill, 0.0, Color32::from_rgb(0x80, 0xff, 0x20));
            p.rect_filled(
                Rect::from_min_size(fill.min, Vec2::new(fill.width(), 4.0)),
                0.0,
                Color32::from_rgb(0xc0, 0xff, 0x80),
            );
            // Segment marks, like the experience bar
            let segments = 18;
            for s in 1..segments {
                let x = rect.min.x + rect.width() * s as f32 / segments as f32;
                p.line_segment(
                    [pos2(x, rect.min.y), pos2(x, rect.max.y)],
                    Stroke::new(2.0, theme::EDGE),
                );
            }
            p.rect_stroke(rect, 0.0, Stroke::new(2.0, theme::EDGE), StrokeKind::Outside);
        });
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }

    fn library(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Your Instances").font(bold_font(20.0)));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text("Search instances")
                        .desired_width(220.0)
                        .margin(Vec2::new(8.0, 6.0)),
                );
            });
        });
        ui.add_space(4.0);
        let (sep, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 2.0), Sense::hover());
        ui.painter().rect_filled(sep, 0.0, theme::PANEL);
        ui.add_space(10.0);

        let query = self.search.trim().to_lowercase();
        let shown: Vec<Instance> = self
            .instances
            .iter()
            .filter(|i| query.is_empty() || i.name.to_lowercase().contains(&query) || i.version.contains(&query))
            .cloned()
            .collect();

        if self.instances.is_empty() {
            ui.label(
                RichText::new("No instances yet. Use New Instance in the sidebar to make one.").color(theme::MUTED),
            );
            return;
        }
        if shown.is_empty() {
            ui.label(RichText::new("No instances match your search.").color(theme::MUTED));
            return;
        }

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let card = Vec2::new(132.0, 128.0);
            let gap = 10.0;
            let columns = ((ui.available_width() + gap) / (card.x + gap)).floor().max(1.0) as usize;
            for row in shown.chunks(columns) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for instance in row {
                        self.instance_card(ui, instance, card);
                    }
                });
                ui.add_space(gap - ui.spacing().item_spacing.y);
            }
        });
    }

    fn instance_card(&mut self, ui: &mut Ui, instance: &Instance, size: Vec2) {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let selected = self.selected.as_deref() == Some(instance.id.as_str());
        let p = ui.painter();
        let fill = if selected {
            Color32::from_rgb(0x2b, 0x3d, 0x23)
        } else if response.hovered() {
            theme::PANEL
        } else {
            Color32::TRANSPARENT
        };
        p.rect_filled(rect, 0.0, fill);
        if selected {
            p.rect_stroke(rect, 0.0, Stroke::new(2.0, theme::GREEN_LIGHT), StrokeKind::Inside);
        } else if response.hovered() {
            p.rect_stroke(rect, 0.0, Stroke::new(2.0, theme::STONE), StrokeKind::Inside);
        }
        let icon = Rect::from_center_size(pos2(rect.center().x, rect.min.y + 42.0), Vec2::splat(56.0));
        art::paint_block_icon(p, icon, &instance.icon);
        if self.running.contains(&instance.id) {
            let dot = Rect::from_min_size(pos2(icon.max.x - 6.0, icon.min.y - 4.0), Vec2::splat(12.0));
            p.rect_filled(dot, 0.0, theme::GREEN_LIGHT);
            p.rect_stroke(dot, 0.0, Stroke::new(2.0, theme::EDGE), StrokeKind::Inside);
        }
        let name = ui
            .painter()
            .layout(instance.name.clone(), bold_font(13.0), theme::TEXT, size.x - 12.0);
        let name_pos = pos2(rect.center().x - name.size().x / 2.0, icon.max.y + 8.0);
        let name_height = name.size().y;
        ui.painter().with_clip_rect(rect).galley(name_pos, name, theme::TEXT);
        ui.painter().text(
            pos2(rect.center().x, (name_pos.y + name_height + 9.0).min(rect.max.y - 9.0)),
            egui::Align2::CENTER_CENTER,
            &instance.version,
            egui::FontId::proportional(12.0),
            theme::MUTED,
        );

        if response.clicked() {
            self.selected = Some(instance.id.clone());
        }
        if response.double_clicked() {
            self.play(ui.ctx(), &instance.id);
        }
        response.context_menu(|ui| {
            if ui.button("Play").clicked() {
                self.play(ui.ctx(), &instance.id);
                ui.close();
            }
            if ui.button("Edit").clicked() {
                self.dialog = Some(Dialog::edit(instance));
                ui.close();
            }
            if ui.button("Open folder").clicked() {
                paths::open_in_explorer(&instance.game_dir());
                ui.close();
            }
            ui.separator();
            if ui.button("Delete").clicked() {
                self.dialog = Some(Dialog::ConfirmDelete(instance.id.clone()));
                ui.close();
            }
        });
    }

    fn notice_bar(&mut self, ui: &mut Ui) {
        let Some((text, is_error)) = self.notice.clone() else {
            return;
        };
        let color = if is_error {
            Color32::from_rgb(0x5a, 0x1c, 0x18)
        } else {
            Color32::from_rgb(0x24, 0x3a, 0x1c)
        };
        egui::Frame::new()
            .fill(color)
            .inner_margin(egui::Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(text).color(Color32::WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Dismiss").clicked() {
                            self.notice = None;
                        }
                    });
                });
            });
    }
}

/// A sidebar entry; with no glyph it shows the player's head.
fn sidebar_button(ui: &mut Ui, icon: Option<art::Glyph>, label: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), Sense::click());
    let p = ui.painter();
    if response.hovered() {
        p.rect_filled(rect, 0.0, theme::PANEL);
        p.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())),
            0.0,
            theme::GREEN_LIGHT,
        );
    }
    let badge = Rect::from_min_size(rect.min + Vec2::new(10.0, 8.0), Vec2::splat(24.0));
    p.rect_filled(badge, 0.0, theme::STONE);
    p.rect_filled(
        Rect::from_min_size(badge.min, Vec2::new(badge.width(), 2.0)),
        0.0,
        Color32::from_rgb(0x6a, 0x6a, 0x6e),
    );
    match icon {
        Some(glyph) => art::paint_glyph(p, badge.shrink(4.0), glyph),
        None => art::paint_head(p, badge.shrink(4.0), "steve"),
    }
    p.text(
        pos2(badge.max.x + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        bold_font(14.0),
        theme::TEXT,
    );
    response
}

fn load_logo(ctx: &egui::Context) -> TextureHandle {
    let image = image::load_from_memory(include_bytes!("../../assets/logo.png"))
        .expect("bundled logo")
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    ctx.load_texture("logo", color, egui::TextureOptions::LINEAR)
}

pub fn window_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../../assets/logo.png"))
        .expect("bundled logo")
        .to_rgba8();
    egui::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_messages(&ctx);

        egui::Panel::left("sidebar")
            .resizable(false)
            .exact_size(220.0)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::SIDEBAR)
                    .inner_margin(12)
                    .stroke(Stroke::new(2.0, theme::EDGE)),
            )
            .show(ui, |ui| self.sidebar(ui));

        if self.notice.is_some() {
            egui::Panel::bottom("notice")
                .show_separator_line(false)
                .frame(egui::Frame::new())
                .show(ui, |ui| self.notice_bar(ui));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                self.hero(ui);
                ui.spacing_mut().item_spacing.y = 8.0;
                egui::Frame::new()
                    .inner_margin(egui::Margin {
                        left: 24,
                        right: 24,
                        top: 18,
                        bottom: 12,
                    })
                    .show(ui, |ui| {
                        self.library(ui);
                    });
            });

        dialogs::show(self, &ctx);

        if self.login.as_ref().is_some_and(|l| l.code.is_some()) {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
    }
}

/// Used by the dialogs to give each modal a stable id.
pub fn modal_id(name: &str) -> Id {
    Id::unique(("scaleri-modal", name))
}
