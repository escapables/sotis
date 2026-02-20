use eframe::egui;

#[derive(Default)]
pub struct SotisApp {
    query: String,
    results: Vec<sotis_core::search::SearchResult>,
}

impl eframe::App for SotisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.query);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.results.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("Type to search files...");
                });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for result in &self.results {
                        ui.label(format!(
                            "{} (score: {:.2})",
                            result.path.display(),
                            result.score
                        ));
                    }
                });
            }
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.label("SOTIS — Ready");
        });
    }
}
