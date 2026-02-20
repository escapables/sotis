use std::path::PathBuf;

/// A single search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub filename: String,
    pub score: f32,
    pub snippet: Option<String>,
}

/// Search mode selector.
#[derive(Debug, Clone, Copy)]
pub enum SearchMode {
    /// Search both content and filenames (default).
    Combined,
    /// Search filenames only.
    FilenameOnly,
    /// Search file content only.
    ContentOnly,
}
