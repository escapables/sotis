use std::collections::HashSet;
use std::sync::mpsc;
use std::thread;

use sotis_core::index::BuildStats;
use sotis_core::search::{QueryMode, SearchEngine, SearchMode, SearchResult};

use crate::app::{SotisApp, RESULTS_LIMIT};
use crate::filters::current_unix_secs;

pub(super) struct SearchJobResult {
    pub(super) query: String,
    pub(super) query_mode: QueryMode,
    pub(super) search_mode: SearchMode,
    pub(super) result: std::result::Result<Vec<SearchResult>, String>,
}

pub(super) struct ReindexJobSuccess {
    pub(super) stats: BuildStats,
    pub(super) doc_count: usize,
    pub(super) indexed_extensions: HashSet<String>,
}

pub(super) struct ReindexJobResult {
    pub(super) result: std::result::Result<ReindexJobSuccess, String>,
}

impl SotisApp {
    pub(super) fn submit_search(&mut self) {
        if self.is_searching {
            return;
        }

        let trimmed = self.query.trim();
        if trimmed.is_empty() {
            self.last_query.clear();
            self.raw_results.clear();
            self.results.clear();
            self.selected_path = None;
            self.preview_text.clear();
            self.status = "SOTIS — Ready".to_string();
            return;
        }

        if self.is_reindexing {
            self.status = "Indexing in progress; search when indexing completes".to_string();
            return;
        }

        let query = trimmed.to_string();
        let query_mode = self.query_mode;
        let search_mode = self.search_mode;

        if self.last_query == query
            && self.last_query_mode == query_mode
            && self.last_search_mode == search_mode
        {
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.search_job_rx = Some(rx);
        self.is_searching = true;
        self.status = format!("Searching for '{query}'...");

        thread::spawn(move || {
            let result = SearchEngine::open_default()
                .and_then(|engine| engine.search(&query, query_mode, search_mode, RESULTS_LIMIT))
                .map_err(|err| err.to_string());
            let _ = tx.send(SearchJobResult {
                query,
                query_mode,
                search_mode,
                result,
            });
        });
    }

    pub(super) fn rerun_last_search(&mut self) {
        if self.last_query.is_empty() || self.is_searching || self.is_reindexing {
            return;
        }

        let query = self.last_query.clone();
        let query_mode = self.last_query_mode;
        let search_mode = self.last_search_mode;

        let (tx, rx) = mpsc::channel();
        self.search_job_rx = Some(rx);
        self.is_searching = true;
        self.status = format!("Refreshing search for '{query}'...");

        thread::spawn(move || {
            let result = SearchEngine::open_default()
                .and_then(|engine| engine.search(&query, query_mode, search_mode, RESULTS_LIMIT))
                .map_err(|err| err.to_string());
            let _ = tx.send(SearchJobResult {
                query,
                query_mode,
                search_mode,
                result,
            });
        });
    }

    pub(super) fn poll_background_jobs(&mut self) {
        self.poll_search_job();
        self.poll_reindex_job();
    }

    fn poll_search_job(&mut self) {
        let Some(receiver) = &self.search_job_rx else {
            return;
        };
        let Ok(job) = receiver.try_recv() else {
            return;
        };

        self.search_job_rx = None;
        self.is_searching = false;

        match job.result {
            Ok(results) => {
                self.last_query = job.query;
                self.last_query_mode = job.query_mode;
                self.last_search_mode = job.search_mode;
                self.status = format!("Search completed for '{}'", self.last_query);
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

    fn poll_reindex_job(&mut self) {
        let Some(receiver) = &self.reindex_job_rx else {
            return;
        };
        let Ok(job) = receiver.try_recv() else {
            return;
        };

        self.reindex_job_rx = None;
        self.is_reindexing = false;

        match job.result {
            Ok(success) => {
                self.index_error_count = success.stats.errors.len();
                self.indexed_docs = success.doc_count;
                self.pending_pdf_ocr_paths = success.stats.ocr_pending.clone();
                self.indexed_extensions = success.indexed_extensions;
                self.last_build_unix_secs = Some(current_unix_secs());
                self.status = if success.stats.ocr_pending.is_empty() {
                    format!(
                        "Reindex complete: added {}, already added {}, errors {}",
                        success.stats.added,
                        success.stats.skipped,
                        success.stats.errors.len()
                    )
                } else {
                    format!(
                        "Reindex complete: added {}, already added {}, errors {}, OCR pending {}",
                        success.stats.added,
                        success.stats.skipped,
                        success.stats.errors.len(),
                        success.stats.ocr_pending.len()
                    )
                };
                self.rerun_last_search();
            }
            Err(err) => {
                self.status = format!("Reindex failed: {err}");
            }
        }
    }

    pub(super) fn start_rebuild_index(&mut self, pdf_ocr_approved: bool) {
        if self.is_reindexing {
            self.status = "Indexing already in progress".to_string();
            return;
        }

        let folders = self.config.folders.clone();
        let general = self.config.general.clone();
        let (tx, rx) = mpsc::channel();
        self.reindex_job_rx = Some(rx);
        self.is_reindexing = true;
        self.status = "Indexing started...".to_string();

        thread::spawn(move || {
            let scan_result = sotis_core::scanner::scan(&folders);
            let result = sotis_core::index::SearchIndex::open_default()
                .and_then(|mut index| {
                    index
                        .build_from_scan_with_config(&scan_result, &general, pdf_ocr_approved)
                        .and_then(|stats| {
                            let doc_count = index.doc_count();
                            let indexed_extensions = index.indexed_extensions()?;
                            Ok(ReindexJobSuccess {
                                stats,
                                doc_count,
                                indexed_extensions,
                            })
                        })
                })
                .map_err(|err| err.to_string());
            let _ = tx.send(ReindexJobResult { result });
        });
    }
}
