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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn recognizes_docx_extension() {
        assert!(DocxExtractor.can_extract(Path::new("file.docx")));
        assert!(!DocxExtractor.can_extract(Path::new("file.doc")));
    }

    #[test]
    fn returns_extraction_error_for_invalid_docx() {
        let base = unique_temp_dir();
        let file = base.join("bad.docx");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, b"not-a-zip-docx").expect("write test file");

        let result = DocxExtractor.extract(&file);
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
        std::env::temp_dir().join(format!("sotis-docx-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
