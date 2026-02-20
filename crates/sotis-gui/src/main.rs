mod app;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "SOTIS — File Search",
        options,
        Box::new(|_cc| Ok(Box::new(app::SotisApp::default()))),
    )
}
