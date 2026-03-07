mod app;
mod browser;
mod vault;

use app::DevBrowsersApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 45.0])
            .with_min_inner_size([400.0, 45.0])
            .with_max_inner_size([4000.0, 45.0])
            .with_decorations(false)
            .with_always_on_top()
            .with_position([460.0, 10.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DevBrowsers",
        options,
        Box::new(|_cc| Box::new(DevBrowsersApp::default())),
    )
}