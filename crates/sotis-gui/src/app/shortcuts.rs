use eframe::egui;

use crate::app::SotisApp;

impl SotisApp {
    pub(super) fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::F)) {
            self.focus_search_bar = true;
        }

        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.handle_escape_shortcut();
        }
    }

    fn handle_escape_shortcut(&mut self) {
        if self.selected_path.is_some() || !self.preview_text.is_empty() {
            self.selected_path = None;
            self.preview_text.clear();
            self.match_positions.clear();
            self.current_match_index = 0;
            self.should_scroll_to_match = false;
            self.status = "Selection cleared".to_string();
            return;
        }

        if !self.query.is_empty() {
            self.query.clear();
            self.last_query.clear();
            self.raw_results.clear();
            self.results.clear();
            self.status = "Search cleared".to_string();
        }
    }
}
