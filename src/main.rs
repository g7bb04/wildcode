#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod arduino;
mod boards;
mod config;
mod editor;
mod events;
mod highlight;
mod proc;
mod runner;
mod serial;
mod theme;
mod toolchain;
mod util;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Wild IDE · 野IDE")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([1024.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Wild IDE",
        options,
        Box::new(|cc| Ok(Box::new(app::WildApp::new(cc)))),
    )
}
