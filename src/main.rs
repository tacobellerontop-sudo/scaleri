//! Scaleri: a lightweight Minecraft launcher for Windows.

// No console window behind the launcher in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod accounts;
mod instances;
mod java;
mod launch;
mod net;
mod paths;
mod settings;
mod ui;
mod versions;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Scaleri")
            .with_app_id("scaleri")
            .with_inner_size([1180.0, 740.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(ui::window_icon()),
        ..Default::default()
    };
    eframe::run_native("Scaleri", options, Box::new(|cc| Ok(Box::new(ui::App::new(cc)))))
}
