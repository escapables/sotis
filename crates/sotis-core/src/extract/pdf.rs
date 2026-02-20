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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn recognizes_pdf_extension() {
        assert!(PdfExtractor.can_extract(Path::new("report.PDF")));
        assert!(!PdfExtractor.can_extract(Path::new("report.txt")));
    }

    #[test]
    fn returns_extraction_error_for_invalid_pdf() {
        let base = unique_temp_dir();
        let file = base.join("invalid.pdf");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, b"%PDF-1.7\nnot-a-real-pdf").expect("write invalid pdf");

        let result = PdfExtractor.extract(&file);
        assert!(matches!(
            result,
            Err(crate::error::Error::Extraction { .. })
        ));

        cleanup_temp_dir(&base);
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("sotis-pdf-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
