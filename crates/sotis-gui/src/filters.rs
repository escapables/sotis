use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct FileTypeFilter {
    pub label: &'static str,
    pub extensions: &'static [&'static str],
    pub enabled: bool,
}

pub fn default_file_type_filters() -> Vec<FileTypeFilter> {
    vec![
        FileTypeFilter {
            label: "Text/Code",
            extensions: &[
                "txt", "md", "rs", "py", "json", "toml", "yaml", "yml", "csv",
            ],
            enabled: true,
        },
        FileTypeFilter {
            label: "PDF",
            extensions: &["pdf"],
            enabled: true,
        },
        FileTypeFilter {
            label: "DOCX",
            extensions: &["docx"],
            enabled: true,
        },
        FileTypeFilter {
            label: "ODT",
            extensions: &["odt"],
            enabled: true,
        },
        FileTypeFilter {
            label: "EPUB",
            extensions: &["epub"],
            enabled: true,
        },
        FileTypeFilter {
            label: "Spreadsheet",
            extensions: &["xlsx", "xls", "ods", "csv"],
            enabled: true,
        },
    ]
}

pub fn extension_allowed(path: &Path, allowed_extensions: &[&str]) -> bool {
    if allowed_extensions.is_empty() {
        return false;
    }

    path.extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|ext| allowed_extensions.contains(&ext.as_str()))
}

pub fn size_allowed(path: &Path, min_size: Option<u64>, max_size: Option<u64>) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return true;
    };

    let size = metadata.len();
    if min_size.is_some_and(|min| size < min) {
        return false;
    }
    if max_size.is_some_and(|max| size > max) {
        return false;
    }

    true
}

pub fn parse_megabytes_input(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let megabytes = trimmed.parse::<f64>().ok()?;
    if !megabytes.is_finite() || megabytes < 0.0 {
        return None;
    }

    Some((megabytes * 1_048_576.0) as u64)
}

pub fn file_size_text(path: &Path) -> String {
    match fs::metadata(path) {
        Ok(metadata) => format!("{} bytes", metadata.len()),
        Err(_) => "size unavailable".to_string(),
    }
}

pub fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{extension_allowed, parse_megabytes_input};

    #[test]
    fn empty_or_invalid_megabytes_input_returns_none() {
        assert_eq!(parse_megabytes_input(""), None);
        assert_eq!(parse_megabytes_input("abc"), None);
    }

    #[test]
    fn parses_valid_megabytes_input() {
        assert_eq!(parse_megabytes_input("42"), Some(44_040_192));
    }

    #[test]
    fn parses_decimal_megabytes_input() {
        assert_eq!(parse_megabytes_input("0.001"), Some(1_048));
    }

    #[test]
    fn extension_filter_matches_lowercase_extensions() {
        let allowed = vec!["rs", "md"];
        assert!(extension_allowed(std::path::Path::new("test.rs"), &allowed));
        assert!(extension_allowed(
            std::path::Path::new("README.MD"),
            &allowed
        ));
        assert!(!extension_allowed(
            std::path::Path::new("book.pdf"),
            &allowed
        ));
    }
}
