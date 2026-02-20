use std::path::PathBuf;

use crate::config::FolderEntry;

/// Result of scanning configured folders.
#[derive(Debug)]
pub struct ScanResult {
    pub files: Vec<PathBuf>,
    pub errors: Vec<(PathBuf, String)>,
}

/// Scan configured folders for indexable files.
pub fn scan(_folders: &[FolderEntry]) -> ScanResult {
    ScanResult {
        files: Vec::new(),
        errors: Vec::new(),
    }
}
