mod folders;
mod watcher;

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use eframe::egui;
use sotis_core::config::Config;
use sotis_core::extract;
use sotis_core::index::SearchIndex;
use sotis_core::search::{QueryMode, SearchEngine, SearchMode, SearchResult};
use sotis_core::watcher::FsWatcher;

use crate::filters::{
    default_file_type_filters, extension_allowed, file_size_text, parse_megabytes_input,
    size_allowed, FileTypeFilter,
};
use crate::preview::build_highlight_job;

const RESULTS_LIMIT: usize = 100;
const PREVIEW_MAX_CHARS: usize = 10_000;

pub struct SotisApp {
    query: String,
    query_mode: QueryMode,
    search_mode: SearchMode,
    raw_results: Vec<SearchResult>,
    results: Vec<SearchResult>,
    selected_path: Option<PathBuf>,
    preview_text: String,
    last_query: String,
    last_query_mode: QueryMode,
    last_search_mode: SearchMode,
    status: String,
    search_index: Option<SearchIndex>,
    search_engine: Option<SearchEngine>,
    config: Config,
    fs_watcher: Option<FsWatcher>,
    new_folder_path: String,
    new_folder_recursive: bool,
    selected_folder_index: Option<usize>,
    file_type_filters: Vec<FileTypeFilter>,
    indexed_extensions: HashSet<String>,
    min_size_mb: String,
    max_size_mb: String,
    last_build_unix_secs: Option<u64>,
    indexed_docs: usize,
    index_error_count: usize,
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

        let search_engine = match SearchEngine::open_default() {
            Ok(engine) => Some(engine),
            Err(err) => {
                status = format!("Search error: {err}");
                None
            }
        };

        let mut app = Self {
            query: String::new(),
            query_mode: QueryMode::Fuzzy,
            search_mode: SearchMode::Combined,
            raw_results: Vec::new(),
            results: Vec::new(),
            selected_path: None,
            preview_text: String::new(),
            last_query: String::new(),
            last_query_mode: QueryMode::Fuzzy,
            last_search_mode: SearchMode::Combined,
            status,
            search_index,
            search_engine,
            config,
            fs_watcher: None,
            new_folder_path: String::new(),
            new_folder_recursive: true,
            selected_folder_index: None,
            file_type_filters: default_file_type_filters(),
            indexed_extensions: HashSet::new(),
            min_size_mb: String::new(),
            max_size_mb: String::new(),
            last_build_unix_secs: None,
            indexed_docs: 0,
            index_error_count: 0,
        };
        app.refresh_indexed_extensions();
        app.restart_watcher();
        app
    }
}

impl eframe::App for SotisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_watcher_events();
        self.maybe_refresh_results();

        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Search:");
                let response = ui.text_edit_singleline(&mut self.query);
                if response.changed() {
                    self.maybe_refresh_results();
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
                    self.maybe_refresh_results();
                }
                if ui
                    .selectable_label(matches!(self.query_mode, QueryMode::Regex), "Regex")
                    .clicked()
                {
                    self.query_mode = QueryMode::Regex;
                    self.maybe_refresh_results();
                }

                ui.separator();
                ui.label("Search In:");
                if ui
                    .selectable_label(matches!(self.search_mode, SearchMode::Combined), "Combined")
                    .clicked()
                {
                    self.search_mode = SearchMode::Combined;
                    self.maybe_refresh_results();
                }
                if ui
                    .selectable_label(
                        matches!(self.search_mode, SearchMode::FilenameOnly),
                        "Filename",
                    )
                    .clicked()
                {
                    self.search_mode = SearchMode::FilenameOnly;
                    self.maybe_refresh_results();
                }
                if ui
                    .selectable_label(
                        matches!(self.search_mode, SearchMode::ContentOnly),
                        "Content",
                    )
                    .clicked()
                {
                    self.search_mode = SearchMode::ContentOnly;
                    self.maybe_refresh_results();
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
        });

        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

impl SotisApp {
    fn maybe_refresh_results(&mut self) {
        if self.last_query == self.query
            && self.last_query_mode == self.query_mode
            && self.last_search_mode == self.search_mode
        {
            return;
        }

        self.last_query = self.query.clone();
        self.last_query_mode = self.query_mode;
        self.last_search_mode = self.search_mode;

        let trimmed = self.query.trim();
        if trimmed.is_empty() {
            self.raw_results.clear();
            self.results.clear();
            self.selected_path = None;
            self.preview_text.clear();
            self.status = "SOTIS — Ready".to_string();
            return;
        }

        let Some(engine) = &self.search_engine else {
            self.status = "Search engine unavailable".to_string();
            return;
        };

        match engine.search(trimmed, self.query_mode, self.search_mode, RESULTS_LIMIT) {
            Ok(results) => {
                self.status = format!("Search completed for '{trimmed}'");
                self.raw_results = results;
                self.apply_client_filters();
            }
            Err(err) => {
                self.raw_results.clear();
                self.results.clear();
                self.selected_path = None;
                self.preview_text.clear();
                self.status = format!("Search failed: {err}");
            }
        }
    }

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
            ui.label("Type to search files...");
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

    fn render_preview_panel(&self, ui: &mut egui::Ui) {
        ui.heading("Preview");
        ui.separator();

        if self.preview_text.is_empty() {
            ui.label("Select a result to preview extracted text.");
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("preview")
            .show(ui, |ui| {
                let job = build_highlight_job(&self.preview_text, self.query.trim());
                ui.label(job);
            });
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
            self.preview_text = "No results found".to_string();
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

        match extract::extract_text(&result.path) {
            Ok(text) => {
                self.preview_text = text.chars().take(PREVIEW_MAX_CHARS).collect::<String>();
            }
            Err(err) => {
                self.preview_text = format!("Failed to extract preview: {err}");
            }
        }
    }
}
