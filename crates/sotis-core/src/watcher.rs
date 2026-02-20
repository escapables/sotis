/// File system watcher for incremental re-indexing.
///
/// Uses the `notify` crate to watch configured folders and
/// trigger index updates when files change.
pub struct FsWatcher {
    _placeholder: (),
}

impl FsWatcher {
    /// Create a new file system watcher.
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for FsWatcher {
    fn default() -> Self {
        Self::new()
    }
}
