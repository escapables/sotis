pub mod docx;
pub mod epub;
#[cfg(feature = "ocr")]
pub mod image;
pub mod odt;
pub mod pdf;
#[cfg(feature = "ocr")]
pub mod pdf_ocr;
pub mod plaintext;
pub mod spreadsheet;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

use crate::config::{Config, GeneralConfig};
use crate::error::{Error, Result};

/// Trait for extracting text content from files.
pub trait TextExtractor {
    /// Returns true if this extractor can handle the given file.
    fn can_extract(&self, path: &Path) -> bool;

    /// Extract text content from the file at the given path.
    fn extract(&self, path: &Path) -> Result<String>;
}

/// Returns all available extractors.
#[cfg_attr(not(feature = "ocr"), allow(unused_mut))]
pub fn extractors() -> Vec<Box<dyn TextExtractor>> {
    let mut extractors: Vec<Box<dyn TextExtractor>> = vec![
        Box::new(plaintext::PlaintextExtractor),
        Box::new(pdf::PdfExtractor),
        Box::new(docx::DocxExtractor),
        Box::new(odt::OdtExtractor),
        Box::new(epub::EpubExtractor),
        Box::new(spreadsheet::SpreadsheetExtractor),
    ];

    #[cfg(feature = "ocr")]
    {
        extractors.push(Box::new(image::ImageExtractor));
    }

    extractors
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtractorKind {
    Plaintext,
    Pdf,
    Docx,
    Odt,
    Epub,
    Spreadsheet,
    #[cfg(feature = "ocr")]
    Image,
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

    #[cfg(feature = "ocr")]
    if has_any_magic_prefix(
        path,
        &[
            &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'],
            &[0xFF, 0xD8, 0xFF],
            &[b'I', b'I', 0x2A, 0x00],
            &[b'M', b'M', 0x00, 0x2A],
        ],
    ) {
        return Some(ExtractorKind::Image);
    }

    match extension.as_deref() {
        Some("pdf") => Some(ExtractorKind::Pdf),
        Some("docx") => Some(ExtractorKind::Docx),
        Some("odt") => Some(ExtractorKind::Odt),
        Some("epub") => Some(ExtractorKind::Epub),
        Some("xlsx" | "xls" | "ods" | "csv") => Some(ExtractorKind::Spreadsheet),
        #[cfg(feature = "ocr")]
        Some("png" | "jpg" | "jpeg" | "tiff" | "tif" | "bmp") => Some(ExtractorKind::Image),
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

#[cfg(feature = "ocr")]
fn has_any_magic_prefix(path: &Path, prefixes: &[&[u8]]) -> bool {
    prefixes.iter().any(|prefix| has_magic_prefix(path, prefix))
}

fn runtime_general_config() -> &'static GeneralConfig {
    static GENERAL_CONFIG: OnceLock<GeneralConfig> = OnceLock::new();
    GENERAL_CONFIG.get_or_init(|| {
        Config::load()
            .map(|config| config.general)
            .unwrap_or_default()
    })
}

pub fn extract_text_with_config(path: &Path, config: &GeneralConfig) -> Result<String> {
    extract_text_with_ocr_settings(path, config.ocr_enabled, config.tessdata_path.as_deref())
}

#[cfg_attr(not(feature = "ocr"), allow(unused_variables))]
fn extract_text_with_ocr_settings(
    path: &Path,
    ocr_enabled: bool,
    tessdata_path: Option<&str>,
) -> Result<String> {
    match detect_extractor_kind(path) {
        Some(ExtractorKind::Plaintext) => plaintext::PlaintextExtractor.extract(path),
        Some(ExtractorKind::Pdf) => {
            pdf::extract_with_ocr_fallback(path, ocr_enabled, tessdata_path)
        }
        Some(ExtractorKind::Docx) => docx::DocxExtractor.extract(path),
        Some(ExtractorKind::Odt) => odt::OdtExtractor.extract(path),
        Some(ExtractorKind::Epub) => epub::EpubExtractor.extract(path),
        Some(ExtractorKind::Spreadsheet) => spreadsheet::SpreadsheetExtractor.extract(path),
        #[cfg(feature = "ocr")]
        Some(ExtractorKind::Image) => {
            if !ocr_enabled {
                return Err(Error::Extraction {
                    path: path.to_path_buf(),
                    message: "no extractor available for this file type".to_string(),
                });
            }
            image::ImageExtractor.extract_with_tessdata(path, tessdata_path)
        }
        None => Err(Error::Extraction {
            path: path.to_path_buf(),
            message: "no extractor available for this file type".to_string(),
        }),
    }
}

/// Extract text from a file using the first matching extractor.
pub fn extract_text(path: &Path) -> Result<String> {
    extract_text_with_config(path, runtime_general_config())
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

    #[cfg(feature = "ocr")]
    #[test]
    fn detect_extractor_kind_detects_image_extensions() {
        assert_eq!(
            detect_extractor_kind(Path::new("scan.png")),
            Some(ExtractorKind::Image)
        );
        assert_eq!(
            detect_extractor_kind(Path::new("scan.JPG")),
            Some(ExtractorKind::Image)
        );
        assert_eq!(
            detect_extractor_kind(Path::new("scan.tiff")),
            Some(ExtractorKind::Image)
        );
    }

    #[cfg(feature = "ocr")]
    #[test]
    fn extract_text_with_ocr_disabled_preserves_no_extractor_behavior_for_images() {
        let base = unique_temp_dir();
        let file = base.join("image.png");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(
            &file,
            &[
                0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n', 0, 0, 0, 0,
            ],
        )
        .expect("write test file");

        let result = extract_text_with_ocr_settings(&file, false, None);
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
