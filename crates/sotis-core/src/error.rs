use std::path::PathBuf;

/// Unified error type for sotis-core.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("config I/O error at {path}: {source}")]
    ConfigIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("config parse error at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("config serialize error: {0}")]
    ConfigSerialize(#[source] toml::ser::Error),

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
