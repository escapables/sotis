use std::path::PathBuf;

use eframe::egui;
use sotis_core::config::FolderEntry;
use sotis_core::scanner;

use crate::app::SotisApp;
use crate::filters::current_unix_secs;

impl SotisApp {
    pub(super) fn render_folder_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Folders");

        if self.config.folders.is_empty() {
            ui.label("No indexed folders yet.");
        } else {
            egui::ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    for (index, folder) in self.config.folders.iter().enumerate() {
                        let is_selected = self.selected_folder_index == Some(index);
                        let label = format!(
                            "{} ({})",
                            folder.path.display(),
                            if folder.recursive {
                                "recursive"
                            } else {
                                "top-level"
                            }
                        );
                        if ui.selectable_label(is_selected, label).clicked() {
                            self.selected_folder_index = Some(index);
                        }
                    }
                });
        }

        ui.separator();
        ui.label("Add Folder Path:");
        ui.text_edit_singleline(&mut self.new_folder_path);
        ui.checkbox(&mut self.new_folder_recursive, "Recursive");

        if ui.button("Add Folder").clicked() {
            self.add_folder();
        }

        if ui.button("Remove Selected Folder").clicked() {
            self.remove_selected_folder();
        }

        if ui.button("Reindex Now").clicked() {
            self.rebuild_index();
        }
    }

    fn add_folder(&mut self) {
        let trimmed = self.new_folder_path.trim();
        if trimmed.is_empty() {
            self.status = "Folder path is empty".to_string();
            return;
        }

        let path = PathBuf::from(trimmed);
        if !path.is_dir() {
            self.status = format!("Folder does not exist: {}", path.display());
            return;
        }

        if self.config.folders.iter().any(|folder| folder.path == path) {
            self.status = "Folder already indexed".to_string();
            return;
        }

        let previous = self.config.clone();
        self.config.folders.push(FolderEntry {
            path,
            recursive: self.new_folder_recursive,
            extensions: Vec::new(),
        });

        if let Err(err) = self.config.save() {
            self.config = previous;
            self.status = format!("Failed to save config: {err}");
            return;
        }

        self.new_folder_path.clear();
        self.restart_watcher();
        self.rebuild_index();
    }

    fn remove_selected_folder(&mut self) {
        let Some(index) = self.selected_folder_index else {
            self.status = "No folder selected".to_string();
            return;
        };

        if index >= self.config.folders.len() {
            self.selected_folder_index = None;
            self.status = "Selected folder no longer exists".to_string();
            return;
        }

        let previous = self.config.clone();
        self.config.folders.remove(index);
        self.selected_folder_index = None;

        if let Err(err) = self.config.save() {
            self.config = previous;
            self.status = format!("Failed to save config: {err}");
            return;
        }

        self.restart_watcher();
        self.rebuild_index();
    }

    fn rebuild_index(&mut self) {
        let Some(index) = &mut self.search_index else {
            self.status = "Index unavailable".to_string();
            return;
        };

        let scan_result = scanner::scan(&self.config.folders);
        let result = index
            .build_from_scan(&scan_result)
            .map(|stats| (stats, index.doc_count()));

        match result {
            Ok((stats, doc_count)) => {
                self.index_error_count = stats.errors.len();
                self.indexed_docs = doc_count;
                self.refresh_indexed_extensions();
                self.last_build_unix_secs = Some(current_unix_secs());
                self.status = format!(
                    "Reindex complete: added {}, skipped {}, errors {}",
                    stats.added,
                    stats.skipped,
                    stats.errors.len()
                );
                self.last_query.clear();
                self.maybe_refresh_results();
            }
            Err(err) => {
                self.status = format!("Reindex failed: {err}");
            }
        }
    }
}
