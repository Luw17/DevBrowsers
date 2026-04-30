mod app;
mod bidi;
mod browser;
mod cdp;
mod project;
mod ui_main;
mod ui_projects;
mod ui_vault;
mod vault;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DevBrowsers",
        options,
        Box::new(|_cc| Box::new(app::DevBrowsersApp::default())),
    )
}