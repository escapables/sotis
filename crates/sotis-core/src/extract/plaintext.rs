use std::path::Path;

use crate::error::Result;
use crate::extract::TextExtractor;

const PLAINTEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rs", "py", "js", "ts", "go", "c", "cpp", "h", "hpp", "java", "rb", "sh", "bash",
    "zsh", "fish", "toml", "yaml", "yml", "json", "xml", "html", "css", "sql", "lua", "vim",
    "conf", "cfg", "ini", "env", "log", "csv", "tsv", "org", "rst", "tex", "bib",
];

pub struct PlaintextExtractor;

impl TextExtractor for PlaintextExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| PLAINTEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
    }

    fn extract(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }
}
