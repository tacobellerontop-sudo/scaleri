//! Modal windows: New Instance, Edit Instance, Settings, Accounts and delete confirmation.

use std::sync::atomic::Ordering;

use egui::{pos2, Align, Color32, Layout, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2};

use super::theme::{self, block_button, bold_font, section_title, ButtonKind};
use super::{art, modal_id, App, ManifestState};
use crate::accounts::{self, Account, AccountKind};
use crate::instances::{Instance, ICONS};
use crate::paths;
use crate::settings::{Settings, BUILT_IN_CLIENT_ID};

pub enum Dialog {
    NewInstance(NewInstanceForm),
    Edit(EditForm),
    Settings(Box<Settings>),
    Accounts(AccountsForm),
    ConfirmDelete(String),
}

impl Dialog {
    pub fn accounts(message: Option<&str>) -> Self {
        Dialog::Accounts(AccountsForm {
            offline_name: String::new(),
            message: message.map(String::from),
        })
    }

    pub fn edit(instance: &Instance) -> Self {
        Dialog::Edit(EditForm {
            id: instance.id.clone(),
            name: instance.name.clone(),
            version: instance.version.clone(),
            icon: instance.icon.clone(),
            custom_memory: instance.memory_mb.is_some(),
            memory_mb: instance.memory_mb.unwrap_or(4096),
            snapshots: false,
        })
    }
}

#[derive(Default)]
pub struct NewInstanceForm {
    name: String,
    name_edited: bool,
    version: Option<String>,
    icon: String,
    snapshots: bool,
    filter: String,
}

pub struct EditForm {
    id: String,
    name: String,
    version: String,
    icon: String,
    custom_memory: bool,
    memory_mb: u32,
    snapshots: bool,
}

pub struct AccountsForm {
    offline_name: String,
    message: Option<String>,
}

enum Outcome {
    Keep,
    Close,
}

/// A titled modal with Scaleri's square, dark frame.
fn modal(ctx: &egui::Context, name: &str, title: &str, width: f32, body: impl FnOnce(&mut Ui) -> Outcome) -> Outcome {
    let frame = egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(2.0, theme::EDGE))
        .inner_margin(egui::Margin::same(0));
    let response = egui::Modal::new(modal_id(name))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(150))
        .show(ctx, |ui| {
            ui.set_width(width);
            // Title bar
            egui::Frame::new()
                .fill(Color32::from_rgb(0x1a, 0x1a, 0x1b))
                .inner_margin(egui::Margin::symmetric(20, 14))
                .show(ui, |ui| {
                    ui.set_width(width - 40.0);
                    ui.label(RichText::new(title).font(bold_font(18.0)).color(Color32::WHITE));
                });
            let mut outcome = Outcome::Keep;
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(20, 16))
                .show(ui, |ui| {
                    ui.set_width(width - 40.0);
                    outcome = body(ui);
                });
            outcome
        });
    if response.should_close() {
        return Outcome::Close;
    }
    response.inner
}

fn footer(ui: &mut Ui, primary: &str, primary_enabled: bool, kind: ButtonKind) -> (bool, bool) {
    let mut ok = false;
    let mut cancel = false;
    ui.add_space(8.0);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        let r = ui.add_enabled_ui(primary_enabled, |ui| {
            block_button(ui, primary, kind, Vec2::new(130.0, 38.0), bold_font(14.0))
        });
        ok = r.inner.clicked();
        cancel = block_button(ui, "Cancel", ButtonKind::Stone, Vec2::new(100.0, 38.0), bold_font(14.0)).clicked();
    });
    (ok, cancel)
}

fn icon_picker(ui: &mut Ui, selected: &mut String) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(6.0);
        for icon in ICONS {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(44.0), Sense::click());
            let p = ui.painter();
            let active = selected == icon;
            p.rect_filled(
                rect,
                0.0,
                if active {
                    Color32::from_rgb(0x2b, 0x3d, 0x23)
                } else {
                    theme::FIELD
                },
            );
            art::paint_block_icon(p, rect.shrink(8.0), icon);
            let stroke = if active {
                Stroke::new(2.0, theme::GREEN_LIGHT)
            } else if response.hovered() {
                Stroke::new(2.0, theme::STONE)
            } else {
                Stroke::new(1.0, theme::EDGE)
            };
            p.rect_stroke(rect, 0.0, stroke, StrokeKind::Inside);
            if response.clicked() {
                *selected = icon.to_string();
            }
        }
    });
}

/// The version list with a search box and a snapshot toggle. Returns the clicked version.
fn version_picker(
    app: &mut App,
    ui: &mut Ui,
    selected: Option<&str>,
    snapshots: &mut bool,
    filter: &mut String,
) -> Option<String> {
    let mut picked = None;
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(filter)
                .hint_text("Search versions")
                .desired_width(220.0),
        );
        ui.checkbox(snapshots, "Show snapshots and old versions");
    });
    ui.add_space(4.0);
    let manifest = match &app.manifest {
        ManifestState::Ready(m) => m,
        ManifestState::Loading => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading versions from Mojang…");
            });
            return None;
        }
        ManifestState::Failed(e) => {
            let e = e.clone();
            ui.label(RichText::new(format!("Couldn't load the version list: {e}")).color(theme::GOLD));
            if ui.button("Try again").clicked() {
                app.reload_manifest(ui.ctx());
            }
            return None;
        }
    };
    let query = filter.trim().to_lowercase();
    theme::field_frame().inner_margin(0).show(ui, |ui| {
        egui::ScrollArea::vertical()
            .max_height(230.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for v in &manifest.versions {
                    if !*snapshots && !v.is_release() {
                        continue;
                    }
                    if !query.is_empty() && !v.id.to_lowercase().contains(&query) {
                        continue;
                    }
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
                    let active = selected == Some(v.id.as_str());
                    let p = ui.painter();
                    if active {
                        p.rect_filled(rect, 0.0, Color32::from_rgb(0x2b, 0x3d, 0x23));
                        p.rect_filled(
                            Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())),
                            0.0,
                            theme::GREEN_LIGHT,
                        );
                    } else if response.hovered() {
                        p.rect_filled(rect, 0.0, theme::PANEL);
                    }
                    p.text(
                        pos2(rect.min.x + 14.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &v.id,
                        bold_font(14.0),
                        theme::TEXT,
                    );
                    let kind = match v.kind.as_str() {
                        "release" => "Release",
                        "snapshot" => "Snapshot",
                        "old_beta" => "Beta",
                        "old_alpha" => "Alpha",
                        other => other,
                    };
                    let date = v.release_time.get(..10).unwrap_or_default();
                    p.text(
                        pos2(rect.max.x - 120.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        kind,
                        egui::FontId::proportional(12.0),
                        theme::MUTED,
                    );
                    p.text(
                        pos2(rect.max.x - 14.0, rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        date,
                        egui::FontId::proportional(12.0),
                        theme::MUTED,
                    );
                    if response.clicked() {
                        picked = Some(v.id.clone());
                    }
                }
            });
    });
    picked
}

pub fn show(app: &mut App, ctx: &egui::Context) {
    let Some(mut dialog) = app.dialog.take() else {
        return;
    };
    let outcome = match &mut dialog {
        Dialog::NewInstance(form) => new_instance(app, ctx, form),
        Dialog::Edit(form) => edit_instance(app, ctx, form),
        Dialog::Settings(draft) => settings(app, ctx, draft),
        Dialog::Accounts(form) => accounts_dialog(app, ctx, form),
        Dialog::ConfirmDelete(id) => confirm_delete(app, ctx, id),
    };
    match outcome {
        // A dialog may have opened another one (e.g. Edit opening Delete); keep the newer one.
        Outcome::Keep if app.dialog.is_none() => app.dialog = Some(dialog),
        Outcome::Keep => {}
        Outcome::Close => {
            if let (Dialog::Accounts(_), Some(login)) = (&dialog, app.login.take()) {
                login.progress.cancel.store(true, Ordering::Relaxed);
            }
        }
    }
}

fn new_instance(app: &mut App, ctx: &egui::Context, form: &mut NewInstanceForm) -> Outcome {
    if form.icon.is_empty() {
        form.icon = "grass".into();
    }
    if form.version.is_none() {
        if let ManifestState::Ready(m) = &app.manifest {
            form.version = Some(m.latest.release.clone());
        }
    }
    if !form.name_edited {
        form.name = form
            .version
            .clone()
            .map(|v| format!("Minecraft {v}"))
            .unwrap_or_default();
    }
    modal(ctx, "new-instance", "New Instance", 620.0, |ui| {
        section_title(ui, "NAME");
        let name = ui.add(
            egui::TextEdit::singleline(&mut form.name)
                .desired_width(f32::INFINITY)
                .margin(Vec2::new(8.0, 6.0)),
        );
        if name.changed() {
            form.name_edited = true;
        }
        ui.add_space(6.0);
        section_title(ui, "ICON");
        icon_picker(ui, &mut form.icon);
        ui.add_space(6.0);
        section_title(ui, "MINECRAFT VERSION");
        if let Some(v) = version_picker(app, ui, form.version.as_deref(), &mut form.snapshots, &mut form.filter) {
            form.version = Some(v);
        }
        let ready = form.version.is_some() && !form.name.trim().is_empty();
        let (ok, cancel) = footer(ui, "Create", ready, ButtonKind::Green);
        if ok {
            match Instance::create(&form.name, form.version.as_deref().unwrap_or_default(), &form.icon) {
                Ok(instance) => app.add_instance(instance),
                Err(e) => app.notice = Some((format!("Could not create the instance: {e}"), true)),
            }
            return Outcome::Close;
        }
        if cancel {
            Outcome::Close
        } else {
            Outcome::Keep
        }
    })
}

fn edit_instance(app: &mut App, ctx: &egui::Context, form: &mut EditForm) -> Outcome {
    modal(ctx, "edit-instance", "Edit Instance", 620.0, |ui| {
        section_title(ui, "NAME");
        ui.add(
            egui::TextEdit::singleline(&mut form.name)
                .desired_width(f32::INFINITY)
                .margin(Vec2::new(8.0, 6.0)),
        );
        ui.add_space(6.0);
        section_title(ui, "ICON");
        icon_picker(ui, &mut form.icon);
        ui.add_space(6.0);
        section_title(ui, &format!("MINECRAFT VERSION · {}", form.version));
        let mut filter = String::new();
        if let Some(v) = version_picker(app, ui, Some(&form.version), &mut form.snapshots, &mut filter) {
            form.version = v;
        }
        ui.add_space(6.0);
        section_title(ui, "MEMORY");
        ui.checkbox(
            &mut form.custom_memory,
            "Use a different amount of memory for this instance",
        );
        if form.custom_memory {
            ui.add(
                egui::Slider::new(&mut form.memory_mb, 1024..=16384)
                    .step_by(512.0)
                    .suffix(" MB"),
            );
        }
        ui.add_space(8.0);
        let mut delete = false;
        ui.horizontal(|ui| {
            delete = block_button(
                ui,
                "Delete",
                ButtonKind::Danger,
                Vec2::new(100.0, 38.0),
                bold_font(14.0),
            )
            .clicked();
            let (ok, cancel) = footer(ui, "Save", !form.name.trim().is_empty(), ButtonKind::Green);
            if ok {
                if let Some(inst) = app.instances.iter_mut().find(|i| i.id == form.id) {
                    inst.name = form.name.trim().to_string();
                    inst.icon = form.icon.clone();
                    inst.version = form.version.clone();
                    inst.memory_mb = form.custom_memory.then_some(form.memory_mb);
                    if let Err(e) = inst.save() {
                        app.notice = Some((format!("Could not save the instance: {e}"), true));
                    }
                }
                delete = false;
                form.id.clear();
            } else if cancel {
                form.id.clear();
            }
        });
        if delete {
            app.dialog = Some(Dialog::ConfirmDelete(form.id.clone()));
            return Outcome::Close;
        }
        if form.id.is_empty() {
            Outcome::Close
        } else {
            Outcome::Keep
        }
    })
}

fn settings(app: &mut App, ctx: &egui::Context, draft: &mut Settings) -> Outcome {
    modal(ctx, "settings", "Settings", 600.0, |ui| {
        egui::ScrollArea::vertical().max_height(470.0).show(ui, |ui| {
            section_title(ui, "WHEN THE GAME STARTS");
            ui.radio_value(
                &mut draft.close_on_launch,
                true,
                "Close Scaleri (uses no memory while you play)",
            );
            ui.radio_value(
                &mut draft.close_on_launch,
                false,
                "Keep Scaleri open (tracks play time)",
            );
            ui.add_space(10.0);

            section_title(ui, "MEMORY");
            ui.add(
                egui::Slider::new(&mut draft.memory_mb, 1024..=16384)
                    .step_by(512.0)
                    .suffix(" MB"),
            );
            ui.label(
                RichText::new("4096 MB suits most worlds; give large modpacks more.")
                    .small()
                    .color(theme::MUTED),
            );
            ui.add_space(10.0);

            section_title(ui, "GAME WINDOW");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut draft.game_width).range(640..=7680));
                ui.label("×");
                ui.add(egui::DragValue::new(&mut draft.game_height).range(480..=4320));
            });
            ui.add_space(10.0);

            section_title(ui, "JAVA");
            ui.label(
                "Scaleri downloads the right Java for each version. To use your own, enter the path to javaw.exe:",
            );
            ui.add(
                egui::TextEdit::singleline(&mut draft.java_path)
                    .hint_text("Automatic")
                    .desired_width(f32::INFINITY),
            );
            ui.label("Extra Java arguments:");
            ui.add(
                egui::TextEdit::singleline(&mut draft.java_args)
                    .hint_text("e.g. -XX:+UseG1GC")
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(10.0);

            section_title(ui, "MICROSOFT SIGN-IN");
            let built_in = BUILT_IN_CLIENT_ID.is_some_and(|id| !id.trim().is_empty());
            ui.label(if built_in {
                "This build has a Microsoft application ID. You can override it here."
            } else {
                "Signing in needs the client ID of an Azure application with Minecraft access. See the README."
            });
            ui.add(
                egui::TextEdit::singleline(&mut draft.msa_client_id)
                    .hint_text("Application (client) ID")
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(10.0);

            section_title(ui, "FILES");
            ui.horizontal(|ui| {
                ui.label(RichText::new(paths::data_dir().display().to_string()).color(theme::MUTED));
                if ui.button("Open").clicked() {
                    paths::open_in_explorer(paths::data_dir());
                }
            });
        });
        let (ok, cancel) = footer(ui, "Save", true, ButtonKind::Green);
        if ok {
            app.settings = draft.clone();
            app.settings.save();
            return Outcome::Close;
        }
        if cancel {
            Outcome::Close
        } else {
            Outcome::Keep
        }
    })
}

fn account_row(ui: &mut Ui, account: &Account, active: bool) -> (bool, bool) {
    let mut use_it = false;
    let mut remove = false;
    let fill = if active {
        Color32::from_rgb(0x2b, 0x3d, 0x23)
    } else {
        theme::FIELD
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(2.0, if active { theme::GREEN_LIGHT } else { theme::EDGE }))
        .inner_margin(10)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (head, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
                art::paint_head(ui.painter(), head, &account.name);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&account.name).font(bold_font(15.0)));
                    let kind = match account.kind {
                        AccountKind::Microsoft => "Microsoft account",
                        AccountKind::Offline => "Offline account",
                    };
                    ui.label(
                        RichText::new(if active {
                            format!("{kind} · Playing as")
                        } else {
                            kind.to_string()
                        })
                        .small()
                        .color(theme::MUTED),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    remove = ui.button("Remove").clicked();
                    if !active {
                        use_it = ui.button("Use").clicked();
                    }
                });
            });
        });
    (use_it, remove)
}

fn accounts_dialog(app: &mut App, ctx: &egui::Context, form: &mut AccountsForm) -> Outcome {
    modal(ctx, "accounts", "Accounts", 560.0, |ui| {
        if let Some(message) = &form.message {
            ui.label(RichText::new(message).color(theme::GOLD));
            ui.add_space(4.0);
        }

        let active = app.accounts.active.clone();
        let mut use_id = None;
        let mut remove_id = None;
        for account in &app.accounts.accounts {
            let (use_it, remove) = account_row(ui, account, active.as_deref() == Some(account.uuid.as_str()));
            if use_it {
                use_id = Some(account.uuid.clone());
            }
            if remove {
                remove_id = Some(account.uuid.clone());
            }
        }
        if let Some(id) = use_id {
            app.accounts.active = Some(id);
            app.accounts.save();
        }
        if let Some(id) = remove_id {
            app.accounts.remove(&id);
            app.accounts.save();
        }
        if app.accounts.accounts.is_empty() {
            ui.label(RichText::new("No accounts yet.").color(theme::MUTED));
        }

        ui.add_space(10.0);
        section_title(ui, "MICROSOFT ACCOUNT");
        match &app.login {
            None => {
                if block_button(
                    ui,
                    "Sign in with Microsoft",
                    ButtonKind::Green,
                    Vec2::new(240.0, 40.0),
                    bold_font(14.0),
                )
                .clicked()
                {
                    app.start_sign_in(ui.ctx());
                }
            }
            Some(login) => {
                if let Some(error) = &login.error {
                    ui.label(RichText::new(error).color(theme::GOLD));
                    if ui.button("Try again").clicked() {
                        app.start_sign_in(ui.ctx());
                    }
                } else if let Some(code) = &login.code {
                    ui.label("Enter this code on the Microsoft page that opened in your browser:");
                    ui.horizontal(|ui| {
                        theme::field_frame()
                            .inner_margin(egui::Margin::symmetric(16, 8))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(&code.user_code)
                                        .font(bold_font(26.0))
                                        .color(theme::GREEN_LIGHT),
                                );
                            });
                        if ui.button("Copy").clicked() {
                            ui.ctx().copy_text(code.user_code.clone());
                        }
                        if ui.button("Open page").clicked() {
                            paths::open_url(&code.verification_uri);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.spinner();
                        let stage = login.progress.label.lock().unwrap().clone();
                        ui.label(if stage.is_empty() {
                            "Waiting for you to sign in…".to_string()
                        } else {
                            format!("{stage}…")
                        });
                        if ui.button("Cancel").clicked() {
                            login.progress.cancel.store(true, Ordering::Relaxed);
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Contacting Microsoft…");
                    });
                }
            }
        }

        ui.add_space(10.0);
        section_title(ui, "OFFLINE ACCOUNT");
        let allowed = app.accounts.has_microsoft();
        ui.label(
            RichText::new(if allowed {
                "Play under another name without signing in. Multiplayer servers in online mode won't let you join."
            } else {
                "Available once you've added a Microsoft account that owns Minecraft."
            })
            .small()
            .color(theme::MUTED),
        );
        ui.add_enabled_ui(allowed, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut form.offline_name)
                        .hint_text("Player name")
                        .desired_width(200.0),
                );
                let valid = accounts::valid_player_name(form.offline_name.trim());
                if ui.add_enabled(valid, egui::Button::new("Add")).clicked() {
                    app.accounts.upsert(Account::offline(form.offline_name.trim()));
                    app.accounts.save();
                    form.offline_name.clear();
                }
            });
        });

        ui.add_space(4.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if block_button(ui, "Done", ButtonKind::Stone, Vec2::new(100.0, 38.0), bold_font(14.0)).clicked() {
                if let Some(login) = &app.login {
                    login.progress.cancel.store(true, Ordering::Relaxed);
                }
                app.login = None;
                return Outcome::Close;
            }
            Outcome::Keep
        })
        .inner
    })
}

fn confirm_delete(app: &mut App, ctx: &egui::Context, id: &str) -> Outcome {
    let name = app
        .instances
        .iter()
        .find(|i| i.id == id)
        .map(|i| i.name.clone())
        .unwrap_or_default();
    modal(ctx, "delete", "Delete instance", 460.0, |ui| {
        ui.label(format!(
            "Delete \"{name}\"? Its worlds, screenshots and settings will be removed for good."
        ));
        let (ok, cancel) = footer(ui, "Delete", true, ButtonKind::Danger);
        if ok {
            app.remove_instance(id);
            return Outcome::Close;
        }
        if cancel {
            Outcome::Close
        } else {
            Outcome::Keep
        }
    })
}
