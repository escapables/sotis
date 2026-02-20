use std::path::Path;

use crate::error::Result;
use crate::extract::TextExtractor;

pub struct PdfExtractor;

impl TextExtractor for PdfExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
    }

    fn extract(&self, path: &Path) -> Result<String> {
        let bytes = std::fs::read(path)?;
        pdf_extract::extract_text_from_mem(&bytes).map_err(|e| crate::error::Error::Extraction {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    }
}
