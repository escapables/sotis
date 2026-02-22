use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use pdfium_render::prelude::{PdfRenderConfig, Pdfium};

use crate::error::{Error, Result};
use crate::extract::image::ImageExtractor;

const TARGET_WIDTH_PX: i32 = 2_500;

pub fn ocr_scanned_pdf(path: &Path, tessdata_path: Option<&str>) -> Result<String> {
    let pdfium = bind_pdfium(path)?;
    let document = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|source| Error::Extraction {
            path: path.to_path_buf(),
            message: format!("failed to open PDF with pdfium: {source}"),
        })?;

    let render_config = PdfRenderConfig::new()
        .set_target_width(TARGET_WIDTH_PX)
        .render_annotations(false)
        .use_grayscale_rendering(true);

    let temp_dir = TempDir::new();
    let image_extractor = ImageExtractor;
    let mut all_text = Vec::new();

    for (index, page) in document.pages().iter().enumerate() {
        let rendered =
            page.render_with_config(&render_config)
                .map_err(|source| Error::Extraction {
                    path: path.to_path_buf(),
                    message: format!("failed to render PDF page {index} for OCR: {source}"),
                })?;

        let page_image_path = temp_dir.path().join(format!("page-{index}.png"));
        rendered
            .as_image()
            .save(&page_image_path)
            .map_err(|source| Error::Extraction {
                path: path.to_path_buf(),
                message: format!("failed to write OCR image for page {index}: {source}"),
            })?;

        let page_text = image_extractor.extract_with_tessdata(&page_image_path, tessdata_path)?;
        if !page_text.trim().is_empty() {
            all_text.push(page_text);
        }
    }

    Ok(all_text.join("\n"))
}

fn bind_pdfium(path: &Path) -> Result<Pdfium> {
    let bindings = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
        .or_else(|_| Pdfium::bind_to_system_library())
        .map_err(|source| Error::Extraction {
            path: path.to_path_buf(),
            message: format!("failed to load pdfium library: {source}"),
        })?;

    Ok(Pdfium::new(bindings))
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!("sotis-pdf-ocr-{}-{nanos}", process::id()));
        let _ = fs::create_dir_all(&path);
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
