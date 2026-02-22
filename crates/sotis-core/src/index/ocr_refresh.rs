use std::path::Path;

pub(super) fn should_force_ocr_sensitive_refresh(path: &Path) -> bool {
    #[cfg(feature = "ocr")]
    {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .is_some_and(|ext| {
                matches!(
                    ext.as_str(),
                    "pdf" | "png" | "jpg" | "jpeg" | "tiff" | "tif" | "bmp"
                )
            })
    }

    #[cfg(not(feature = "ocr"))]
    {
        let _ = path;
        false
    }
}

#[cfg(all(test, feature = "ocr"))]
mod tests {
    use std::path::Path;

    use super::should_force_ocr_sensitive_refresh;

    #[test]
    fn force_ocr_sensitive_refresh_detects_pdf_and_images() {
        assert!(should_force_ocr_sensitive_refresh(Path::new("scan.pdf")));
        assert!(should_force_ocr_sensitive_refresh(Path::new("scan.png")));
        assert!(should_force_ocr_sensitive_refresh(Path::new("scan.JPG")));
        assert!(!should_force_ocr_sensitive_refresh(Path::new("notes.txt")));
    }
}
