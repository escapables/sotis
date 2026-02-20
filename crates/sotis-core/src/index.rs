use std::path::Path;

use crate::error::Result;

/// Manages the tantivy search index.
pub struct SearchIndex {
    _index_path: std::path::PathBuf,
}

impl SearchIndex {
    /// Open or create an index at the given path.
    pub fn open(_path: &Path) -> Result<Self> {
        Ok(Self {
            _index_path: _path.to_path_buf(),
        })
    }
}
