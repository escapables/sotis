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
                let result = if path.is_file() {
                    index
                        .update_document(&path)
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
                        self.index_error_count += 1;
                        self.status = format!("Watcher update failed: {err}");
                    }
                }
            }
            WatchEvent::Remove(path) => match index.remove_document(&path) {
                Ok(()) => {
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
            self.last_query.clear();
            self.maybe_refresh_results();
        }
    }
}
