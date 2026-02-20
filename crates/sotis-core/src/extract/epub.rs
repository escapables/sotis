use std::path::Path;

use crate::error::Result;
use crate::extract::TextExtractor;

pub struct EpubExtractor;

impl TextExtractor for EpubExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("epub"))
    }

    fn extract(&self, path: &Path) -> Result<String> {
        let mut doc =
            epub::doc::EpubDoc::new(path).map_err(|e| crate::error::Error::Extraction {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;

        let mut text = String::new();
        while doc.go_next() {
            if let Some((content, _mime)) = doc.get_current_str() {
                // Strip HTML tags for plain text
                let plain = strip_html_tags(&content);
                if !plain.trim().is_empty() {
                    text.push_str(plain.trim());
                    text.push('\n');
                }
            }
        }
        Ok(text)
    }
}

/// Naive HTML tag stripper for epub content.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn recognizes_epub_extension() {
        assert!(EpubExtractor.can_extract(Path::new("book.epub")));
        assert!(!EpubExtractor.can_extract(Path::new("book.txt")));
    }

    #[test]
    fn strips_html_tags() {
        let plain = super::strip_html_tags("<h1>Title</h1><p>Hello <b>world</b></p>");
        assert!(plain.contains("Title"));
        assert!(plain.contains("Hello world"));
        assert!(!plain.contains('<'));
    }

    #[test]
    fn returns_extraction_error_for_invalid_epub() {
        let base = unique_temp_dir();
        let file = base.join("bad.epub");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, b"invalid epub payload").expect("write test file");

        let result = EpubExtractor.extract(&file);
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
        std::env::temp_dir().join(format!("sotis-epub-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
