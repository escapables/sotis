use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use eframe::egui;
use sotis_core::config::FolderEntry;

use crate::app::SotisApp;
use crate::filters::file_size_text;

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
        ui.checkbox(&mut self.new_folder_recursive, "Recursive");

        if ui
            .add_enabled(!self.is_reindexing, egui::Button::new("Add Folder"))
            .clicked()
        {
            self.pick_and_add_folder();
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

        if ui
            .add_enabled(!self.is_reindexing, egui::Button::new("Clear Index"))
            .clicked()
        {
            self.confirm_clear_index = true;
        }

        if self.confirm_clear_index {
            ui.separator();
            ui.label("Clear all indexed documents and OCR approvals?");
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.is_reindexing, egui::Button::new("Confirm Clear"))
                    .clicked()
                {
                    self.clear_index();
                }
                if ui.button("Cancel").clicked() {
                    self.confirm_clear_index = false;
                }
            });
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
            ui.small("OCR is slower for scanned PDFs and may take minutes on large files.");
            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    let pending_paths = self.pending_pdf_ocr_paths.clone();
                    for path in pending_paths {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(path.display().to_string());
                            ui.small(file_size_text(&path));
                            if ui
                                .add_enabled(!self.is_reindexing, egui::Button::new("Approve"))
                                .clicked()
                            {
                                self.approve_pending_pdf(path.clone());
                            }
                            if ui
                                .add_enabled(!self.is_reindexing, egui::Button::new("Deny"))
                                .clicked()
                            {
                                self.deny_pending_pdf(path.clone());
                            }
                        });
                    }
                });
        }
    }

    fn pick_and_add_folder(&mut self) {
        match pick_folder_path() {
            Ok(Some(path)) => self.add_folder(path),
            Ok(None) => {
                self.status = "Folder selection canceled".to_string();
            }
            Err(err) => {
                self.status = format!("Folder picker unavailable: {err}");
            }
        }
    }

    fn add_folder(&mut self, path: PathBuf) {
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

    fn approve_pending_pdf(&mut self, path: PathBuf) {
        let Some(index) = &mut self.search_index else {
            self.status = "Index unavailable".to_string();
            return;
        };

        if let Err(err) = index.set_pdf_ocr_approved(&path, true) {
            self.status = format!("Failed to persist OCR approval: {err}");
            return;
        }

        match index.update_document_with_config(&path, &self.config.general, true) {
            Ok(changed) => {
                self.pending_pdf_ocr_paths
                    .retain(|pending| pending != &path);
                self.indexed_docs = index.doc_count();
                self.refresh_indexed_extensions();
                self.rerun_last_search();
                self.status = if changed {
                    format!("OCR approved and indexed: {}", path.display())
                } else {
                    format!("OCR approval saved: {}", path.display())
                };
            }
            Err(err) => {
                self.status = format!("OCR approval failed: {err}");
            }
        }
    }

    fn deny_pending_pdf(&mut self, path: PathBuf) {
        let Some(index) = &mut self.search_index else {
            self.status = "Index unavailable".to_string();
            return;
        };

        if let Err(err) = index.set_pdf_ocr_approved(&path, false) {
            self.status = format!("Failed to persist OCR denial: {err}");
            return;
        }

        if let Err(err) = index.remove_document(&path) {
            self.status = format!("Failed to remove denied PDF from index: {err}");
            return;
        }

        self.pending_pdf_ocr_paths
            .retain(|pending| pending != &path);
        self.indexed_docs = index.doc_count();
        self.refresh_indexed_extensions();
        self.rerun_last_search();
        self.status = format!("OCR denied: {}", path.display());
    }

    fn clear_index(&mut self) {
        let index_path = self
            .search_index
            .as_ref()
            .map(|index| index.index_path().to_path_buf())
            .unwrap_or_else(|| sotis_core::config::data_dir().join("index"));

        self.search_index = None;

        if index_path.exists() {
            if let Err(err) = fs::remove_dir_all(&index_path) {
                self.status = format!("Failed to clear index directory: {err}");
                self.confirm_clear_index = false;
                return;
            }
        }

        match sotis_core::index::SearchIndex::open(&index_path) {
            Ok(index) => {
                self.search_index = Some(index);
                self.raw_results.clear();
                self.results.clear();
                self.selected_path = None;
                self.preview_text.clear();
                self.match_positions.clear();
                self.current_match_index = 0;
                self.should_scroll_to_match = false;
                self.pending_pdf_ocr_paths.clear();
                self.indexed_extensions.clear();
                self.indexed_docs = 0;
                self.index_error_count = 0;
                self.last_build_unix_secs = None;
                self.status = "Index cleared".to_string();
            }
            Err(err) => {
                self.status = format!("Failed to reopen cleared index: {err}");
            }
        }

        self.confirm_clear_index = false;
    }
}

fn pick_folder_path() -> Result<Option<PathBuf>, String> {
    let pickers: [(&str, &[&str]); 2] = [
        (
            "zenity",
            &[
                "--file-selection",
                "--directory",
                "--title=Select Folder to Index",
            ],
        ),
        (
            "kdialog",
            &["--getexistingdirectory", ".", "Select Folder to Index"],
        ),
    ];

    for (command, args) in pickers {
        let output = match Command::new(command).args(args).output() {
            Ok(output) => output,
            Err(err) if err.kind() == ErrorKind::NotFound => continue,
            Err(err) => return Err(format!("{command} failed to start: {err}")),
        };

        if output.status.success() {
            let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if selected.is_empty() {
                return Ok(None);
            }
            return Ok(Some(PathBuf::from(selected)));
        }

        if output.status.code() == Some(1) {
            return Ok(None);
        }
    }

    Err("install zenity or kdialog".to_string())
}
