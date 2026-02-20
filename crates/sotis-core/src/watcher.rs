use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};

use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::config::FolderEntry;
use crate::error::{Error, Result};

/// File system watcher event consumed by callers for incremental index updates.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WatchEvent {
    Upsert(PathBuf),
    Remove(PathBuf),
    Error(String),
}

#[derive(Clone)]
struct FolderRule {
    path: PathBuf,
    extensions: HashSet<String>,
}

impl FolderRule {
    fn allows(&self, path: &Path) -> bool {
        if !path.starts_with(&self.path) {
            return false;
        }

        if is_hidden(path) || matches_common_ignored_path(path) {
            return false;
        }

        if self.extensions.is_empty() {
            return true;
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .is_some_and(|ext| self.extensions.contains(&ext))
    }
}

/// File system watcher for incremental re-indexing.
///
/// Uses the `notify` crate to watch configured folders and emit normalized
/// create/modify/delete events for indexable files.
pub struct FsWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<WatchEvent>,
}

impl FsWatcher {
    /// Start watching configured folders and return a receiver-backed watcher.
    pub fn watch_folders(folders: &[FolderEntry]) -> Result<Self> {
        let rules = build_rules(folders);
        let (tx, rx) = mpsc::channel();
        let tx_events = tx.clone();
        let callback_rules = rules.clone();

        let mut watcher =
            notify::recommended_watcher(move |result: notify::Result<Event>| match result {
                Ok(event) => {
                    for watch_event in map_notify_event(&event, &callback_rules) {
                        let _ = tx_events.send(watch_event);
                    }
                }
                Err(err) => {
                    let _ = tx.send(WatchEvent::Error(err.to_string()));
                }
            })
            .map_err(|err| Error::Watcher(format!("failed to initialize watcher: {err}")))?;

        watcher
            .configure(Config::default())
            .map_err(|err| Error::Watcher(format!("failed to configure watcher: {err}")))?;

        for folder in folders {
            let mode = if folder.recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            watcher.watch(&folder.path, mode).map_err(|err| {
                Error::Watcher(format!(
                    "failed to watch folder {}: {err}",
                    folder.path.display()
                ))
            })?;
        }

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Poll the next pending event, if available.
    pub fn try_recv(&self) -> Option<WatchEvent> {
        self.rx.try_recv().ok()
    }
}

fn build_rules(folders: &[FolderEntry]) -> Vec<FolderRule> {
    folders
        .iter()
        .map(|folder| FolderRule {
            path: folder.path.clone(),
            extensions: folder
                .extensions
                .iter()
                .filter_map(|ext| {
                    let normalized = ext.trim().trim_start_matches('.').to_ascii_lowercase();
                    if normalized.is_empty() {
                        None
                    } else {
                        Some(normalized)
                    }
                })
                .collect(),
        })
        .collect()
}

fn map_notify_event(event: &Event, rules: &[FolderRule]) -> Vec<WatchEvent> {
    let Some(kind) = classify_kind(&event.kind) else {
        return Vec::new();
    };

    let mut emitted = HashSet::new();
    let mut output = Vec::new();

    for path in &event.paths {
        if !rules.iter().any(|rule| rule.allows(path)) {
            continue;
        }

        let watch_event = match kind {
            EventAction::Upsert => WatchEvent::Upsert(path.clone()),
            EventAction::Remove => WatchEvent::Remove(path.clone()),
        };

        if emitted.insert(watch_event.clone()) {
            output.push(watch_event);
        }
    }

    output
}

#[derive(Clone, Copy)]
enum EventAction {
    Upsert,
    Remove,
}

fn classify_kind(kind: &EventKind) -> Option<EventAction> {
    match kind {
        EventKind::Create(CreateKind::Any)
        | EventKind::Create(CreateKind::File)
        | EventKind::Create(CreateKind::Folder)
        | EventKind::Create(CreateKind::Other) => Some(EventAction::Upsert),
        EventKind::Modify(ModifyKind::Any)
        | EventKind::Modify(ModifyKind::Data(_))
        | EventKind::Modify(ModifyKind::Metadata(_))
        | EventKind::Modify(ModifyKind::Name(_))
        | EventKind::Modify(ModifyKind::Other) => Some(EventAction::Upsert),
        EventKind::Remove(RemoveKind::Any)
        | EventKind::Remove(RemoveKind::File)
        | EventKind::Remove(RemoveKind::Folder)
        | EventKind::Remove(RemoveKind::Other) => Some(EventAction::Remove),
        _ => None,
    }
}

fn matches_common_ignored_path(path: &Path) -> bool {
    const IGNORED_NAMES: &[&str] = &[".git", "node_modules", "target", ".cache", "__pycache__"];
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|segment| IGNORED_NAMES.contains(&segment))
    })
}

fn is_hidden(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|segment| segment.starts_with('.') && segment != "." && segment != "..")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_notify_event_filters_by_extension() {
        let rules = vec![FolderRule {
            path: PathBuf::from("/tmp/docs"),
            extensions: HashSet::from([String::from("txt")]),
        }];

        let event = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![
                PathBuf::from("/tmp/docs/notes.txt"),
                PathBuf::from("/tmp/docs/notes.md"),
            ],
            attrs: Default::default(),
        };

        let mapped = map_notify_event(&event, &rules);
        assert_eq!(
            mapped,
            vec![WatchEvent::Upsert(PathBuf::from("/tmp/docs/notes.txt"))]
        );
    }

    #[test]
    fn map_notify_event_maps_remove() {
        let rules = vec![FolderRule {
            path: PathBuf::from("/tmp/docs"),
            extensions: HashSet::new(),
        }];
        let event = Event {
            kind: EventKind::Remove(RemoveKind::File),
            paths: vec![PathBuf::from("/tmp/docs/deleted.txt")],
            attrs: Default::default(),
        };

        let mapped = map_notify_event(&event, &rules);
        assert_eq!(
            mapped,
            vec![WatchEvent::Remove(PathBuf::from("/tmp/docs/deleted.txt"))]
        );
    }

    #[test]
    fn map_notify_event_skips_hidden_and_ignored_paths() {
        let rules = vec![FolderRule {
            path: PathBuf::from("/tmp/project"),
            extensions: HashSet::new(),
        }];
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Any),
            paths: vec![
                PathBuf::from("/tmp/project/.env"),
                PathBuf::from("/tmp/project/node_modules/lib.js"),
                PathBuf::from("/tmp/project/readme.md"),
            ],
            attrs: Default::default(),
        };

        let mapped = map_notify_event(&event, &rules);
        assert_eq!(
            mapped,
            vec![WatchEvent::Upsert(PathBuf::from("/tmp/project/readme.md"))]
        );
    }
}
