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
