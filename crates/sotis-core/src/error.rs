use std::path::PathBuf;

/// Unified error type for sotis-core.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("config error: {0}")]
    Config(String),

    #[error("index error: {0}")]
    Index(String),

    #[error("search error: {0}")]
    Search(String),

    #[error("extraction error for {path}: {message}")]
    Extraction { path: PathBuf, message: String },

    #[error("scanner error: {0}")]
    Scanner(String),

    #[error("watcher error: {0}")]
    Watcher(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tantivy(#[from] tantivy::TantivyError),
}

pub type Result<T> = std::result::Result<T, Error>;
