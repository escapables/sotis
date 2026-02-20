use std::path::PathBuf;

use eframe::egui;
use egui::text::LayoutJob;
use egui::{Color32, FontId, TextFormat};
use sotis_core::extract;
use sotis_core::index::SearchIndex;
use sotis_core::search::{QueryMode, SearchEngine, SearchMode, SearchResult};

const RESULTS_LIMIT: usize = 100;
const PREVIEW_MAX_CHARS: usize = 10_000;

pub struct SotisApp {
    query: String,
    query_mode: QueryMode,
    results: Vec<SearchResult>,
    selected_path: Option<PathBuf>,
    preview_text: String,
    last_query: String,
    last_query_mode: QueryMode,
    status: String,
    search_index: Option<SearchIndex>,
    search_engine: Option<SearchEngine>,
}

impl Default for SotisApp {
    fn default() -> Self {
        let mut status = String::from("SOTIS — Ready");
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

        Self {
            query: String::new(),
            query_mode: QueryMode::Fuzzy,
            results: Vec::new(),
            selected_path: None,
            preview_text: String::new(),
            last_query: String::new(),
            last_query_mode: QueryMode::Fuzzy,
            status,
            search_index,
            search_engine,
        }
    }
}

impl eframe::App for SotisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.maybe_refresh_results();

        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");
                let response = ui.text_edit_singleline(&mut self.query);
                if response.changed() {
                    self.maybe_refresh_results();
                }

                ui.separator();
                ui.label("Mode:");
                let fuzzy =
                    ui.selectable_label(matches!(self.query_mode, QueryMode::Fuzzy), "Fuzzy");
                if fuzzy.clicked() {
                    self.query_mode = QueryMode::Fuzzy;
                    self.maybe_refresh_results();
                }
                let regex =
                    ui.selectable_label(matches!(self.query_mode, QueryMode::Regex), "Regex");
                if regex.clicked() {
                    self.query_mode = QueryMode::Regex;
                    self.maybe_refresh_results();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                self.render_results_panel(&mut columns[0]);
                self.render_preview_panel(&mut columns[1]);
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            if let Some(index) = &self.search_index {
                ui.label(format!(
                    "{} | index: {} | results: {}",
                    self.status,
                    index.index_path().display(),
                    self.results.len()
                ));
            } else {
                ui.label(format!("{} | results: {}", self.status, self.results.len()));
            }
        });
    }
}

impl SotisApp {
    fn maybe_refresh_results(&mut self) {
        if self.last_query == self.query && self.last_query_mode == self.query_mode {
            return;
        }

        self.last_query = self.query.clone();
        self.last_query_mode = self.query_mode;

        let trimmed = self.query.trim();
        if trimmed.is_empty() {
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

        match engine.search(
            trimmed,
            self.query_mode,
            SearchMode::Combined,
            RESULTS_LIMIT,
        ) {
            Ok(results) => {
                self.status = format!("Search completed for '{trimmed}'");
                self.results = results;
                if self.results.is_empty() {
                    self.selected_path = None;
                    self.preview_text = "No results found".to_string();
                } else if self
                    .selected_path
                    .as_ref()
                    .is_none_or(|path| !self.results.iter().any(|result| &result.path == path))
                {
                    self.select_result(0);
                }
            }
            Err(err) => {
                self.results.clear();
                self.selected_path = None;
                self.preview_text.clear();
                self.status = format!("Search failed: {err}");
            }
        }
    }

    fn render_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Results");
        ui.separator();

        if self.results.is_empty() {
            ui.label("Type to search files...");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for index in 0..self.results.len() {
                let (path, filename, score) = {
                    let result = &self.results[index];
                    (result.path.clone(), result.filename.clone(), result.score)
                };
                let is_selected = self.selected_path.as_ref() == Some(&path);
                let label = format!("{} ({:.2})", filename, score);
                if ui.selectable_label(is_selected, label).clicked() {
                    self.select_result(index);
                }
                ui.label(path.display().to_string());
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

        egui::ScrollArea::vertical().show(ui, |ui| {
            let job = build_highlight_job(&self.preview_text, self.query.trim());
            ui.label(job);
        });
    }

    fn select_result(&mut self, index: usize) {
        let Some(result) = self.results.get(index) else {
            return;
        };

        self.selected_path = Some(result.path.clone());

        match extract::extract_text(&result.path) {
            Ok(text) => {
                let truncated = text.chars().take(PREVIEW_MAX_CHARS).collect::<String>();
                self.preview_text = truncated;
            }
            Err(err) => {
                self.preview_text = format!("Failed to extract preview: {err}");
            }
        }
    }
}

fn build_highlight_job(text: &str, query: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let default_format = TextFormat {
        font_id: FontId::monospace(13.0),
        color: Color32::LIGHT_GRAY,
        ..Default::default()
    };
    let highlight_format = TextFormat {
        font_id: FontId::monospace(13.0),
        color: Color32::BLACK,
        background: Color32::from_rgb(244, 208, 63),
        ..Default::default()
    };

    if query.is_empty() {
        job.append(text, 0.0, default_format);
        return job;
    }

    let mut ranges = Vec::new();

    for token in query.split_whitespace().filter(|part| !part.is_empty()) {
        for (offset, _) in text.match_indices(token) {
            let range_start = offset;
            let range_end = offset + token.len();
            ranges.push((range_start, range_end));
        }
    }

    ranges.sort_unstable_by_key(|range| range.0);
    ranges.dedup();

    if ranges.is_empty() {
        job.append(text, 0.0, default_format);
        return job;
    }

    let merged = merge_ranges(&ranges);
    let mut cursor = 0usize;
    for (start, end) in merged {
        if start > cursor {
            job.append(&text[cursor..start], 0.0, default_format.clone());
        }
        if end > cursor {
            job.append(&text[start..end], 0.0, highlight_format.clone());
            cursor = end;
        }
    }

    if cursor < text.len() {
        job.append(&text[cursor..], 0.0, default_format);
    }

    job
}

fn merge_ranges(ranges: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut merged = Vec::new();
    for &(start, end) in ranges {
        if let Some((_, prev_end)) = merged.last_mut() {
            if start <= *prev_end {
                *prev_end = (*prev_end).max(end);
                continue;
            }
        }
        merged.push((start, end));
    }
    merged
}
