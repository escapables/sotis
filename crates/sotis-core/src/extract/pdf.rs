use std::path::Path;

use crate::error::Result;
use crate::extract::TextExtractor;

pub struct PdfExtractor;
#[cfg(feature = "ocr")]
pub const PDF_OCR_APPROVAL_REQUIRED_MESSAGE: &str =
    "PDF appears image-only; manual OCR approval required";
#[cfg(feature = "ocr")]
const SCANNED_PDF_TEXT_THRESHOLD: usize = 50;
#[cfg(feature = "ocr")]
const READABLE_TEXT_RATIO_THRESHOLD: f32 = 0.5;

#[cfg_attr(not(feature = "ocr"), allow(unused_variables))]
pub fn extract_with_ocr_fallback(
    path: &Path,
    pdf_ocr_approved: bool,
    tessdata_path: Option<&str>,
) -> Result<String> {
    let extracted =
        pdf_extract::extract_text(path).map_err(|e| crate::error::Error::Extraction {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

    #[cfg(feature = "ocr")]
    {
        if !should_run_ocr_fallback(&extracted) {
            return Ok(extracted);
        }

        if let Ok(pdfium_text) = crate::extract::pdf_ocr::pdfium_extract_text(path) {
            if !should_run_ocr_fallback(&pdfium_text) {
                return Ok(pdfium_text);
            }
        }

        if pdf_ocr_approved {
            let ocr_text = crate::extract::pdf_ocr::ocr_scanned_pdf(path, tessdata_path)?;
            if !ocr_text.trim().is_empty() {
                return Ok(ocr_text);
            }
            eprintln!(
                "warning: scanned PDF OCR returned empty text for {}",
                path.display()
            );
            return Ok(extracted);
        }

        Err(crate::error::Error::Extraction {
            path: path.to_path_buf(),
            message: PDF_OCR_APPROVAL_REQUIRED_MESSAGE.to_string(),
        })
    }

    #[cfg(not(feature = "ocr"))]
    Ok(extracted)
}

#[cfg(feature = "ocr")]
fn is_near_empty_extracted_text(text: &str) -> bool {
    text.trim().len() < SCANNED_PDF_TEXT_THRESHOLD
}

#[cfg(feature = "ocr")]
fn should_run_ocr_fallback(text: &str) -> bool {
    is_near_empty_extracted_text(text) || has_low_readable_text_ratio(text)
}

#[cfg(feature = "ocr")]
fn has_low_readable_text_ratio(text: &str) -> bool {
    let total = text.chars().count();
    if total == 0 {
        return true;
    }

    let readable = text
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_ascii_whitespace())
        .count();
    let ratio = readable as f32 / total as f32;

    ratio < READABLE_TEXT_RATIO_THRESHOLD
}

impl TextExtractor for PdfExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
    }

    fn extract(&self, path: &Path) -> Result<String> {
        extract_with_ocr_fallback(path, false, None)
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

    #[cfg(feature = "ocr")]
    #[test]
    fn near_empty_text_detection_uses_threshold() {
        assert!(is_near_empty_extracted_text("   "));
        assert!(is_near_empty_extracted_text(&"a".repeat(49)));
        assert!(!is_near_empty_extracted_text(&"a".repeat(50)));
    }

    #[cfg(feature = "ocr")]
    #[test]
    fn readable_text_ratio_detection_catches_garbled_text() {
        assert!(has_low_readable_text_ratio("@@@###$$$***"));
        assert!(has_low_readable_text_ratio("abc!!!@@@"));
        assert!(!has_low_readable_text_ratio("readable plain text 123"));
    }

    #[cfg(feature = "ocr")]
    #[test]
    fn ocr_fallback_runs_for_near_empty_or_low_quality_text() {
        assert!(should_run_ocr_fallback(""));
        assert!(should_run_ocr_fallback("   "));
        assert!(should_run_ocr_fallback("abc!!!@@@"));
        assert!(!should_run_ocr_fallback(
            "This is a long readable line that should keep pdf_extract output."
        ));
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
