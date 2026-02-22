use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use pdfium_render::prelude::{PdfRenderConfig, Pdfium};
use rayon::prelude::*;

use crate::config;
use crate::error::{Error, Result};

const TARGET_WIDTH_PX: i32 = 1_200;
const OCR_CACHE_DIR: &str = "ocr-cache";
const OCR_CACHE_VERSION: &str = "v1";
const OCR_CACHE_METADATA_LINES: usize = 3;

pub fn pdfium_extract_text(path: &Path) -> Result<String> {
    let pdfium = bind_pdfium(path)?;
    let document = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|source| Error::Extraction {
            path: path.to_path_buf(),
            message: format!("failed to open PDF with pdfium: {source}"),
        })?;

    let mut all_text = Vec::new();
    for (index, page) in document.pages().iter().enumerate() {
        let page_text = page
            .text()
            .map_err(|source| Error::Extraction {
                path: path.to_path_buf(),
                message: format!("failed to read PDF text layer on page {index}: {source}"),
            })?
            .all();
        if !page_text.trim().is_empty() {
            all_text.push(page_text);
        }
    }

    let text = all_text.join("\n");
    eprintln!(
        "pdf-tier: pdfium text read complete path={} pages={} trimmed_len={}",
        path.display(),
        document.pages().len(),
        text.trim().len()
    );
    Ok(text)
}

pub fn ocr_scanned_pdf(path: &Path, tessdata_path: Option<&str>) -> Result<String> {
    if let Some(cached) = read_ocr_cache(path)? {
        eprintln!(
            "pdf-tier: tier3(ocr) cache hit path={} trimmed_len={}",
            path.display(),
            cached.trim().len()
        );
        return Ok(cached);
    }

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
    let mut all_text = Vec::new();
    let mut ocr_jobs = Vec::new();

    for (index, page) in document.pages().iter().enumerate() {
        let page_text = page
            .text()
            .map_err(|source| Error::Extraction {
                path: path.to_path_buf(),
                message: format!("failed to read PDF text layer on page {index}: {source}"),
            })?
            .all();
        if !page_text.trim().is_empty() {
            all_text.push(page_text);
            continue;
        }

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

        ocr_jobs.push((index, page_image_path));
    }

    let mut ocr_pages = ocr_jobs
        .into_par_iter()
        .map(|(index, page_image_path)| {
            let page_text = crate::extract::image::ImageExtractor
                .extract_with_tessdata(&page_image_path, tessdata_path)?;
            Ok::<(usize, String), Error>((index, page_text))
        })
        .collect::<Vec<_>>();
    ocr_pages.sort_by_key(|result| {
        result
            .as_ref()
            .map(|(index, _)| *index)
            .unwrap_or(usize::MAX)
    });

    for page_result in ocr_pages {
        let (_, page_text) = page_result?;
        if !page_text.trim().is_empty() {
            all_text.push(page_text);
        }
    }

    let text = all_text.join("\n");
    write_ocr_cache(path, &text)?;
    Ok(text)
}

fn bind_pdfium(path: &Path) -> Result<Pdfium> {
    let mut attempts = Vec::new();

    if let Ok(custom_path) = std::env::var("SOTIS_PDFIUM_LIB_PATH") {
        if !custom_path.trim().is_empty() {
            attempts.push(PathBuf::from(custom_path));
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            attempts.push(exe_dir.join("lib").join("libpdfium.so"));
            attempts.push(exe_dir.join("libpdfium.so"));
        }
    }

    attempts.push(PathBuf::from("./libpdfium.so"));

    for candidate in attempts {
        if !candidate.exists() {
            continue;
        }
        match Pdfium::bind_to_library(candidate.to_string_lossy().as_ref()) {
            Ok(bindings) => {
                eprintln!(
                    "pdf-tier: loaded pdfium from explicit path path={} lib={}",
                    path.display(),
                    candidate.display()
                );
                return Ok(Pdfium::new(bindings));
            }
            Err(source) => {
                eprintln!(
                    "pdf-tier: failed loading pdfium candidate path={} lib={} err={}",
                    path.display(),
                    candidate.display(),
                    source
                );
            }
        }
    }

    match Pdfium::bind_to_system_library() {
        Ok(bindings) => {
            eprintln!(
                "pdf-tier: loaded pdfium from system library path={}",
                path.display()
            );
            Ok(Pdfium::new(bindings))
        }
        Err(source) => Err(Error::Extraction {
            path: path.to_path_buf(),
            message: format!("failed to load pdfium library: {source}"),
        }),
    }
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

fn read_ocr_cache(path: &Path) -> Result<Option<String>> {
    let cache_path = cache_file_path(path)?;
    if !cache_path.exists() {
        return Ok(None);
    }

    let Some(modified) = modified_secs(path)? else {
        return Ok(None);
    };
    let content = fs::read_to_string(&cache_path).map_err(|source| Error::Extraction {
        path: path.to_path_buf(),
        message: format!(
            "failed to read OCR cache {}: {source}",
            cache_path.display()
        ),
    })?;
    let mut lines = content.lines();
    let Some(version) = lines.next() else {
        return Ok(None);
    };
    if version.trim() != OCR_CACHE_VERSION {
        return Ok(None);
    }
    let Some(cached_modified) = lines.next().and_then(|raw| raw.trim().parse::<u64>().ok()) else {
        return Ok(None);
    };
    if cached_modified != modified {
        return Ok(None);
    }
    let Some(cached_len) = lines
        .next()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
    else {
        return Ok(None);
    };
    let text = content
        .lines()
        .skip(OCR_CACHE_METADATA_LINES)
        .collect::<Vec<_>>()
        .join("\n");
    if text.len() != cached_len {
        return Ok(None);
    }
    Ok(Some(text))
}

fn write_ocr_cache(path: &Path, text: &str) -> Result<()> {
    let Some(modified) = modified_secs(path)? else {
        return Ok(());
    };
    let cache_path = cache_file_path(path)?;
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Extraction {
            path: path.to_path_buf(),
            message: format!(
                "failed to create OCR cache directory {}: {source}",
                parent.display()
            ),
        })?;
    }

    let payload = format!("{OCR_CACHE_VERSION}\n{modified}\n{}\n{text}", text.len());
    fs::write(&cache_path, payload).map_err(|source| Error::Extraction {
        path: path.to_path_buf(),
        message: format!(
            "failed to write OCR cache {}: {source}",
            cache_path.display()
        ),
    })?;
    Ok(())
}

fn cache_file_path(path: &Path) -> Result<PathBuf> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    let cache_key = format!("{:016x}.txt", hasher.finish());
    Ok(config::data_dir().join(OCR_CACHE_DIR).join(cache_key))
}

fn modified_secs(path: &Path) -> Result<Option<u64>> {
    let Ok(metadata) = fs::metadata(path) else {
        return Ok(None);
    };
    let Ok(modified) = metadata.modified() else {
        return Ok(None);
    };
    let Ok(duration) = modified.duration_since(UNIX_EPOCH) else {
        return Ok(None);
    };
    Ok(Some(duration.as_secs()))
}
