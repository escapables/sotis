mod folders;
mod jobs;
mod watcher;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use eframe::egui;
use sotis_core::config::Config;
use sotis_core::extract;
use sotis_core::index::SearchIndex;
use sotis_core::search::{QueryMode, SearchEngine, SearchMode, SearchResult};
use sotis_core::watcher::FsWatcher;

use self::jobs::{ReindexJobResult, SearchJobResult};
use crate::filters::{
    default_file_type_filters, extension_allowed, file_size_text, parse_megabytes_input,
    size_allowed, FileTypeFilter,
};
use crate::preview::{build_highlight_job, find_all_match_positions};

const RESULTS_LIMIT: usize = 100;

pub struct SotisApp {
    query: String,
    query_mode: QueryMode,
    search_mode: SearchMode,
    raw_results: Vec<SearchResult>,
    results: Vec<SearchResult>,
    selected_path: Option<PathBuf>,
    preview_text: String,
    match_positions: Vec<usize>,
    current_match_index: usize,
    should_scroll_to_match: bool,
    last_query: String,
    last_query_mode: QueryMode,
    last_search_mode: SearchMode,
    status: String,
    search_index: Option<SearchIndex>,
    config: Config,
    fs_watcher: Option<FsWatcher>,
    new_folder_recursive: bool,
    selected_folder_index: Option<usize>,
    file_type_filters: Vec<FileTypeFilter>,
    indexed_extensions: HashSet<String>,
    min_size_mb: String,
    max_size_mb: String,
    last_build_unix_secs: Option<u64>,
    indexed_docs: usize,
    index_error_count: usize,
    pending_pdf_ocr_paths: Vec<PathBuf>,
    is_searching: bool,
    is_reindexing: bool,
    search_job_rx: Option<Receiver<SearchJobResult>>,
    reindex_job_rx: Option<Receiver<ReindexJobResult>>,
}

impl Default for SotisApp {
    fn default() -> Self {
        let mut status = String::from("SOTIS — Ready");

        let config = match Config::load() {
            Ok(config) => config,
            Err(err) => {
                status = format!("Config load error: {err}");
                Config::default()
            }
        };

        let search_index = match SearchIndex::open_default() {
            Ok(index) => Some(index),
            Err(err) => {
                status = format!("Index error: {err}");
                None
            }
        };

        if let Err(err) = SearchEngine::open_default() {
            status = format!("Search error: {err}");
        }

        let mut app = Self {
            query: String::new(),
            query_mode: QueryMode::Fuzzy,
            search_mode: SearchMode::Combined,
            raw_results: Vec::new(),
            results: Vec::new(),
            selected_path: None,
            preview_text: String::new(),
            match_positions: Vec::new(),
            current_match_index: 0,
            should_scroll_to_match: false,
            last_query: String::new(),
            last_query_mode: QueryMode::Fuzzy,
            last_search_mode: SearchMode::Combined,
            status,
            search_index,
            config,
            fs_watcher: None,
            new_folder_recursive: true,
            selected_folder_index: None,
            file_type_filters: default_file_type_filters(),
            indexed_extensions: HashSet::new(),
            min_size_mb: String::new(),
            max_size_mb: String::new(),
            last_build_unix_secs: None,
            indexed_docs: 0,
            index_error_count: 0,
            pending_pdf_ocr_paths: Vec::new(),
            is_searching: false,
            is_reindexing: false,
            search_job_rx: None,
            reindex_job_rx: None,
        };
        app.refresh_indexed_extensions();
        app.restart_watcher();
        app
    }
}

impl eframe::App for SotisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_background_jobs();
        self.process_watcher_events();

        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Search:");
                let response = ui.text_edit_singleline(&mut self.query);
                let trigger_with_enter =
                    response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                if trigger_with_enter {
                    self.submit_search();
                }

                if ui
                    .add_enabled(
                        !self.is_searching && !self.is_reindexing,
                        egui::Button::new("Search"),
                    )
                    .clicked()
                {
                    self.submit_search();
                }
                if self.is_searching {
                    ui.add(egui::Spinner::new());
                }

                ui.separator();
                ui.label("Query:");
                let disable_fuzzy_query_mode = matches!(
                    (self.query_mode, self.search_mode),
                    (QueryMode::Regex, SearchMode::FilenameOnly)
                );
                let fuzzy_response = ui
                    .add_enabled_ui(!disable_fuzzy_query_mode, |ui| {
                        ui.selectable_label(matches!(self.query_mode, QueryMode::Fuzzy), "Fuzzy")
                    })
                    .inner;
                if fuzzy_response.clicked() {
                    self.query_mode = QueryMode::Fuzzy;
                }
                if ui
                    .selectable_label(matches!(self.query_mode, QueryMode::Regex), "Regex")
                    .clicked()
                {
                    self.query_mode = QueryMode::Regex;
                }

                ui.separator();
                ui.label("Search In:");
                if ui
                    .selectable_label(matches!(self.search_mode, SearchMode::Combined), "Combined")
                    .clicked()
                {
                    self.search_mode = SearchMode::Combined;
                }
                if ui
                    .selectable_label(
                        matches!(self.search_mode, SearchMode::FilenameOnly),
                        "Filename",
                    )
                    .clicked()
                {
                    self.search_mode = SearchMode::FilenameOnly;
                }
                if ui
                    .selectable_label(
                        matches!(self.search_mode, SearchMode::ContentOnly),
                        "Content",
                    )
                    .clicked()
                {
                    self.search_mode = SearchMode::ContentOnly;
                }
            });
        });

        egui::SidePanel::left("filters_and_folders")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.render_folder_panel(ui);
                ui.separator();
                self.render_filters_panel(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                self.render_results_panel(&mut columns[0]);
                self.render_preview_panel(&mut columns[1]);
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            let last_build = self
                .last_build_unix_secs
                .map(|secs| secs.to_string())
                .unwrap_or_else(|| "never".to_string());
            ui.label(format!(
                "{} | indexed docs: {} | last build: {} | index errors: {} | results: {}",
                self.status,
                self.indexed_docs,
                last_build,
                self.index_error_count,
                self.results.len()
            ));
            if self.is_reindexing {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    ui.label("indexing in progress...");
                });
            }
            if !self.pending_pdf_ocr_paths.is_empty() {
                ui.label(format!(
                    "pending PDF OCR approval: {} file(s)",
                    self.pending_pdf_ocr_paths.len()
                ));
            }
        });

        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

impl SotisApp {
    fn render_filters_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Filters");
        ui.label("File types:");

        let mut changed = false;
        let indexed_extensions = &self.indexed_extensions;
        for filter in &mut self.file_type_filters {
            if !filter
                .extensions
                .iter()
                .any(|ext| indexed_extensions.contains(*ext))
            {
                continue;
            }
            changed |= ui.checkbox(&mut filter.enabled, filter.label).changed();
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Min MB:");
            changed |= ui.text_edit_singleline(&mut self.min_size_mb).changed();
        });
        ui.horizontal(|ui| {
            ui.label("Max MB:");
            changed |= ui.text_edit_singleline(&mut self.max_size_mb).changed();
        });

        if changed {
            self.apply_client_filters();
        }
    }

    fn render_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Results");
        ui.separator();

        if self.results.is_empty() {
            ui.label("Press Enter or click Search");
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("results")
            .show(ui, |ui| {
                for index in 0..self.results.len() {
                    let (path, filename, score, size) = {
                        let result = &self.results[index];
                        (
                            result.path.clone(),
                            result.filename.clone(),
                            result.score,
                            file_size_text(&result.path),
                        )
                    };

                    let is_selected = self.selected_path.as_ref() == Some(&path);
                    let label = format!("{} ({:.2})", filename, score);
                    if ui.selectable_label(is_selected, label).clicked() {
                        self.select_result(index);
                    }
                    ui.label(path.display().to_string());
                    ui.label(size);
                    ui.separator();
                }
            });
    }

    fn render_preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Preview");
        ui.separator();

        if self.preview_text.is_empty() {
            ui.label("Select a result to preview extracted text.");
            return;
        }

        if self.match_positions.is_empty() {
            ui.horizontal(|ui| {
                ui.label("No matches");
                ui.add_enabled(false, egui::Button::new("Prev"));
                ui.add_enabled(false, egui::Button::new("Next"));
            });
        } else {
            let current = self.current_match_index + 1;
            let total = self.match_positions.len();
            ui.horizontal(|ui| {
                ui.label(format!("Match {current} of {total}"));

                if ui.button("Prev").clicked() {
                    self.current_match_index = self.current_match_index.saturating_sub(1);
                    self.should_scroll_to_match = true;
                }

                if ui.button("Next").clicked() {
                    self.current_match_index = (self.current_match_index + 1).min(total - 1);
                    self.should_scroll_to_match = true;
                }
            });
        }
        ui.separator();

        let selected_line = self.selected_match_line();
        let query = self.last_query.trim().to_string();
        let mut should_scroll = self.should_scroll_to_match;
        egui::ScrollArea::vertical()
            .id_salt("preview")
            .show(ui, |ui| {
                for (line_idx, line) in self.preview_text.lines().enumerate() {
                    let response = ui.label(build_highlight_job(line, &query));
                    if should_scroll && selected_line == Some(line_idx) {
                        ui.scroll_to_rect(response.rect, Some(egui::Align::Center));
                        should_scroll = false;
                    }
                }
            });
        self.should_scroll_to_match = should_scroll;
    }

    fn apply_client_filters(&mut self) {
        let allowed_extensions = self.enabled_extensions();
        let min_size_bytes = parse_megabytes_input(&self.min_size_mb);
        let max_size_bytes = parse_megabytes_input(&self.max_size_mb);

        self.results = self
            .raw_results
            .iter()
            .filter(|result| {
                extension_allowed(&result.path, &allowed_extensions)
                    && size_allowed(&result.path, min_size_bytes, max_size_bytes)
            })
            .cloned()
            .collect();

        if self.results.is_empty() {
            self.selected_path = None;
            self.preview_text.clear();
            self.match_positions.clear();
            self.current_match_index = 0;
            self.should_scroll_to_match = false;
            return;
        }

        if self
            .selected_path
            .as_ref()
            .is_none_or(|path| !self.results.iter().any(|result| &result.path == path))
        {
            self.select_result(0);
        }
    }

    fn enabled_extensions(&self) -> Vec<&'static str> {
        self.file_type_filters
            .iter()
            .filter(|filter| {
                filter.enabled
                    && filter
                        .extensions
                        .iter()
                        .any(|ext| self.indexed_extensions.contains(*ext))
            })
            .flat_map(|filter| filter.extensions.iter().copied())
            .collect()
    }

    fn refresh_indexed_extensions(&mut self) {
        let Some(index) = &self.search_index else {
            self.indexed_extensions.clear();
            return;
        };

        match index.indexed_extensions() {
            Ok(extensions) => {
                self.indexed_extensions = extensions;
            }
            Err(err) => {
                self.status = format!("Failed to read indexed file types: {err}");
                self.indexed_extensions.clear();
            }
        }
    }

    fn select_result(&mut self, index: usize) {
        let Some(result) = self.results.get(index) else {
            return;
        };

        self.selected_path = Some(result.path.clone());

        match extract::extract_text_with_pdf_ocr_approval(&result.path, &self.config.general, false)
        {
            Ok(text) => {
                self.preview_text = text;
                self.match_positions =
                    find_all_match_positions(&self.preview_text, self.last_query.trim());
                self.current_match_index = 0;
                self.should_scroll_to_match = !self.match_positions.is_empty();
            }
            Err(err) => {
                self.preview_text = format!("Failed to extract preview: {err}");
                self.match_positions.clear();
                self.current_match_index = 0;
                self.should_scroll_to_match = false;
            }
        }
    }

    fn selected_match_line(&self) -> Option<usize> {
        let offset = *self.match_positions.get(self.current_match_index)?;
        Some(
            self.preview_text[..offset]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count(),
        )
    }
}
