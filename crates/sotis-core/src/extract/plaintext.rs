use std::path::Path;

use crate::error::Result;
use crate::extract::TextExtractor;

const PLAINTEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rs", "py", "js", "ts", "go", "c", "cpp", "h", "hpp", "java", "rb", "sh", "bash",
    "zsh", "fish", "toml", "yaml", "yml", "json", "xml", "html", "css", "sql", "lua", "vim",
    "conf", "cfg", "ini", "env", "log", "csv", "tsv", "org", "rst", "tex", "bib",
];

pub struct PlaintextExtractor;

pub(crate) fn supports_extension(extension: &str) -> bool {
    PLAINTEXT_EXTENSIONS.contains(&extension)
}

impl TextExtractor for PlaintextExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
            .is_some_and(supports_extension)
    }

    fn extract(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
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
    fn recognizes_supported_extensions_case_insensitively() {
        let extractor = PlaintextExtractor;
        assert!(extractor.can_extract(Path::new("notes.TXT")));
        assert!(extractor.can_extract(Path::new("config.Toml")));
        assert!(!extractor.can_extract(Path::new("image.png")));
    }

    #[test]
    fn extracts_plaintext_contents() {
        let base = unique_temp_dir();
        let file = base.join("file.md");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, "# Title\nbody").expect("write text");

        let text = PlaintextExtractor
            .extract(&file)
            .expect("extract should succeed");
        assert!(text.contains("Title"));

        cleanup_temp_dir(&base);
    }

    #[test]
    fn returns_io_error_for_unreadable_path() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create temp dir");

        let result = PlaintextExtractor.extract(&base);
        assert!(matches!(result, Err(crate::error::Error::Io(_))));

        cleanup_temp_dir(&base);
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("sotis-plaintext-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
