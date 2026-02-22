pub mod docx;
pub mod epub;
pub mod odt;
pub mod pdf;
pub mod plaintext;
pub mod spreadsheet;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::{Error, Result};

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
        Box::new(odt::OdtExtractor),
        Box::new(epub::EpubExtractor),
        Box::new(spreadsheet::SpreadsheetExtractor),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtractorKind {
    Plaintext,
    Pdf,
    Docx,
    Odt,
    Epub,
    Spreadsheet,
}

fn detect_extractor_kind(path: &Path) -> Option<ExtractorKind> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);

    if has_magic_prefix(path, b"%PDF-") {
        return Some(ExtractorKind::Pdf);
    }

    if has_magic_prefix(path, &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        return Some(ExtractorKind::Spreadsheet);
    }

    match extension.as_deref() {
        Some("pdf") => Some(ExtractorKind::Pdf),
        Some("docx") => Some(ExtractorKind::Docx),
        Some("odt") => Some(ExtractorKind::Odt),
        Some("epub") => Some(ExtractorKind::Epub),
        Some("xlsx" | "xls" | "ods" | "csv") => Some(ExtractorKind::Spreadsheet),
        Some(ext) if plaintext::supports_extension(ext) => Some(ExtractorKind::Plaintext),
        _ => None,
    }
}

fn has_magic_prefix(path: &Path, prefix: &[u8]) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };

    let mut buf = vec![0_u8; prefix.len()];
    let Ok(read) = file.read(&mut buf) else {
        return false;
    };

    read == prefix.len() && buf == prefix
}

/// Extract text from a file using the first matching extractor.
pub fn extract_text(path: &Path) -> Result<String> {
    match detect_extractor_kind(path) {
        Some(ExtractorKind::Plaintext) => plaintext::PlaintextExtractor.extract(path),
        Some(ExtractorKind::Pdf) => pdf::PdfExtractor.extract(path),
        Some(ExtractorKind::Docx) => docx::DocxExtractor.extract(path),
        Some(ExtractorKind::Odt) => odt::OdtExtractor.extract(path),
        Some(ExtractorKind::Epub) => epub::EpubExtractor.extract(path),
        Some(ExtractorKind::Spreadsheet) => spreadsheet::SpreadsheetExtractor.extract(path),
        None => Err(Error::Extraction {
            path: path.to_path_buf(),
            message: "no extractor available for this file type".to_string(),
        }),
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
    fn extract_text_uses_plaintext_for_text_extensions() {
        let base = unique_temp_dir();
        let file = base.join("note.txt");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, "hello from plaintext").expect("write test file");

        let content = extract_text(&file).expect("extract text");
        assert!(content.contains("hello from plaintext"));

        cleanup_temp_dir(&base);
    }

    #[test]
    fn extract_text_detects_pdf_from_magic_without_extension() {
        let base = unique_temp_dir();
        let file = base.join("sample");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, b"%PDF-1.7\ninvalid").expect("write test file");

        let result = extract_text(&file);
        assert!(matches!(result, Err(Error::Extraction { .. })));

        cleanup_temp_dir(&base);
    }

    #[test]
    fn extract_text_returns_unsupported_error_for_unknown_extension() {
        let base = unique_temp_dir();
        let file = base.join("unknown.bin");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, [1_u8, 2, 3, 4]).expect("write test file");

        let result = extract_text(&file);
        assert!(matches!(result, Err(Error::Extraction { .. })));

        cleanup_temp_dir(&base);
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sotis-extract-mod-tests-{}-{}",
            process::id(),
            nanos
        ))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
