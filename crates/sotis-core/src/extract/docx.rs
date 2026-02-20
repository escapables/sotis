use std::io::Read;
use std::path::Path;

use dotext::doc::MsDoc;
use dotext::Docx;

use crate::error::Result;
use crate::extract::TextExtractor;

pub struct DocxExtractor;

impl TextExtractor for DocxExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("docx"))
    }

    fn extract(&self, path: &Path) -> Result<String> {
        let mut docx = Docx::open(path).map_err(|e| crate::error::Error::Extraction {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        let mut text = String::new();
        docx.read_to_string(&mut text)
            .map_err(|e| crate::error::Error::Extraction {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;
        Ok(text)
    }
}
