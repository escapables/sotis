use sotis_core::watcher::{FsWatcher, WatchEvent};

use crate::app::SotisApp;

impl SotisApp {
    pub(super) fn restart_watcher(&mut self) {
        if self.config.folders.is_empty() {
            self.fs_watcher = None;
            return;
        }

        match FsWatcher::watch_folders(&self.config.folders) {
            Ok(watcher) => {
                self.fs_watcher = Some(watcher);
            }
            Err(err) => {
                self.fs_watcher = None;
                self.status = format!("Watcher unavailable: {err}");
            }
        }
    }

    pub(super) fn process_watcher_events(&mut self) {
        if self.is_reindexing {
            return;
        }

        loop {
            let next_event = self.fs_watcher.as_ref().and_then(FsWatcher::try_recv);
            let Some(event) = next_event else {
                break;
            };
            self.apply_watcher_event(event);
        }
    }

    fn apply_watcher_event(&mut self, event: WatchEvent) {
        let Some(index) = &mut self.search_index else {
            return;
        };

        let mut should_refresh = false;
        let mut index_changed = false;

        match event {
            WatchEvent::Upsert(path) => {
                let event_path = path.clone();
                let result = if path.is_file() {
                    index
                        .update_document_with_config(&path, &self.config.general, false)
                        .map(|changed| changed.then_some(path))
                } else {
                    index.remove_document(&path).map(|_| Some(path))
                };

                match result {
                    Ok(Some(path)) => {
                        self.indexed_docs = index.doc_count();
                        self.status = format!("Index updated: {}", path.display());
                        index_changed = true;
                        should_refresh = true;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        if sotis_core::extract::is_pdf_ocr_approval_required_error(&err) {
                            if let Err(remove_err) = index.remove_document(&event_path) {
                                self.index_error_count += 1;
                                self.status =
                                    format!("Watcher OCR pending cleanup failed: {remove_err}");
                                return;
                            }
                            self.pending_pdf_ocr_paths.push(event_path.clone());
                            self.pending_pdf_ocr_paths.sort();
                            self.pending_pdf_ocr_paths.dedup();
                            self.indexed_docs = index.doc_count();
                            index_changed = true;
                            should_refresh = true;
                            self.status = format!(
                                "Watcher found image-only PDF pending OCR approval: {}",
                                event_path.display()
                            );
                        } else {
                            self.index_error_count += 1;
                            self.status = format!("Watcher update failed: {err}");
                        }
                    }
                }
            }
            WatchEvent::Remove(path) => match index.remove_document(&path) {
                Ok(()) => {
                    self.pending_pdf_ocr_paths
                        .retain(|pending| pending != &path);
                    self.indexed_docs = index.doc_count();
                    self.status = format!("Index removed: {}", path.display());
                    index_changed = true;
                    should_refresh = true;
                }
                Err(err) => {
                    self.index_error_count += 1;
                    self.status = format!("Watcher remove failed: {err}");
                }
            },
            WatchEvent::Error(message) => {
                self.index_error_count += 1;
                self.status = format!("Watcher error: {message}");
            }
        }

        if index_changed {
            self.refresh_indexed_extensions();
        }

        if should_refresh {
            self.rerun_last_search();
        }
    }
}
