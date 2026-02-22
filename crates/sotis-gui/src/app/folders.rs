use std::path::PathBuf;

use eframe::egui;
use sotis_core::config::FolderEntry;

use crate::app::SotisApp;

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

        if ui
            .add_enabled(!self.is_reindexing, egui::Button::new("Add Folder"))
            .clicked()
        {
            self.add_folder();
        }

        if ui
            .add_enabled(
                !self.is_reindexing,
                egui::Button::new("Remove Selected Folder"),
            )
            .clicked()
        {
            self.remove_selected_folder();
        }

        if ui
            .add_enabled(!self.is_reindexing, egui::Button::new("Reindex Now"))
            .clicked()
        {
            self.start_rebuild_index(false);
        }

        if self.is_reindexing {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new());
                ui.label("Indexing...");
            });
        }

        if !self.pending_pdf_ocr_paths.is_empty() {
            ui.separator();
            ui.label(format!(
                "{} PDF file(s) need OCR approval:",
                self.pending_pdf_ocr_paths.len()
            ));
            let preview_list = self
                .pending_pdf_ocr_paths
                .iter()
                .take(3)
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            if !preview_list.is_empty() {
                ui.small(preview_list);
            }
            ui.small("OCR is slower for scanned PDFs and may take minutes on large files.");
            if ui
                .add_enabled(
                    !self.is_reindexing,
                    egui::Button::new("Approve OCR for Pending PDFs"),
                )
                .clicked()
            {
                self.start_rebuild_index(true);
            }
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
        self.start_rebuild_index(false);
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
        self.start_rebuild_index(false);
    }
}
