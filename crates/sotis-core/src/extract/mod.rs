pub mod docx;
pub mod epub;
pub mod pdf;
pub mod plaintext;
pub mod spreadsheet;

use std::path::Path;

use crate::error::Result;

/// Trait for extracting text content from files.
pub trait TextExtractor {
    /// Returns true if this extractor can handle the given file.
    fn can_extract(&self, path: &Path) -> bool;

    /// Extract text content from the file at the given path.
    fn extract(&self, path: &Path) -> Result<String>;
}

/// Returns all available extractors.
pub fn extractors() -> Vec<Box<dyn TextExtractor>> {
    vec![
        Box::new(plaintext::PlaintextExtractor),
        Box::new(pdf::PdfExtractor),
        Box::new(docx::DocxExtractor),
        Box::new(epub::EpubExtractor),
        Box::new(spreadsheet::SpreadsheetExtractor),
    ]
}

/// Extract text from a file using the first matching extractor.
pub fn extract_text(path: &Path) -> Result<String> {
    for extractor in extractors() {
        if extractor.can_extract(path) {
            return extractor.extract(path);
        }
    }
    Err(crate::error::Error::Extraction {
        path: path.to_path_buf(),
        message: "no extractor available for this file type".to_string(),
    })
}
