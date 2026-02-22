use std::path::Path;

use tesseract::Tesseract;

use crate::error::{Error, Result};
use crate::extract::TextExtractor;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "tiff", "tif", "bmp"];

pub struct ImageExtractor;

impl ImageExtractor {
    pub fn extract_with_tessdata(
        &self,
        path: &Path,
        tessdata_path: Option<&str>,
    ) -> Result<String> {
        let image_path = path.to_str().ok_or_else(|| Error::Extraction {
            path: path.to_path_buf(),
            message: "image path is not valid UTF-8".to_string(),
        })?;

        let mut tess =
            Tesseract::new(tessdata_path, Some("eng")).map_err(|source| Error::Extraction {
                path: path.to_path_buf(),
                message: format!("failed to initialize tesseract: {source}"),
            })?;

        tess = tess
            .set_image(image_path)
            .map_err(|source| Error::Extraction {
                path: path.to_path_buf(),
                message: format!("failed to load image into tesseract: {source}"),
            })?;

        tess = tess.recognize().map_err(|source| Error::Extraction {
            path: path.to_path_buf(),
            message: format!("failed to run OCR: {source}"),
        })?;

        tess.get_text().map_err(|source| Error::Extraction {
            path: path.to_path_buf(),
            message: format!("failed to read OCR text: {source}"),
        })
    }
}

impl TextExtractor for ImageExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
            .is_some_and(|ext| IMAGE_EXTENSIONS.contains(&ext))
    }

    fn extract(&self, path: &Path) -> Result<String> {
        self.extract_with_tessdata(path, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_image_extensions() {
        assert!(ImageExtractor.can_extract(Path::new("scan.PNG")));
        assert!(ImageExtractor.can_extract(Path::new("photo.jpg")));
        assert!(ImageExtractor.can_extract(Path::new("photo.jpeg")));
        assert!(ImageExtractor.can_extract(Path::new("scan.tiff")));
        assert!(ImageExtractor.can_extract(Path::new("scan.tif")));
        assert!(ImageExtractor.can_extract(Path::new("scan.bmp")));
        assert!(!ImageExtractor.can_extract(Path::new("notes.txt")));
    }
}
