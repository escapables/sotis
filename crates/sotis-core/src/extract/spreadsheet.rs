use std::path::Path;

use calamine::{open_workbook_auto, DataType, Reader};

use crate::error::Result;
use crate::extract::TextExtractor;

const SPREADSHEET_EXTENSIONS: &[&str] = &["xlsx", "xls", "ods", "csv", "tsv"];

pub struct SpreadsheetExtractor;

impl TextExtractor for SpreadsheetExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| SPREADSHEET_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
    }

    fn extract(&self, path: &Path) -> Result<String> {
        if is_delimited_text(path, "csv") {
            return read_delimited_file(path, ',');
        }

        if is_delimited_text(path, "tsv") {
            return read_delimited_file(path, '\t');
        }

        let mut workbook =
            open_workbook_auto(path).map_err(|e| crate::error::Error::Extraction {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;

        let mut text = String::new();
        for sheet_name in workbook.sheet_names().to_vec() {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                for row in range.rows() {
                    let cells: Vec<String> = row
                        .iter()
                        .filter_map(|cell| {
                            if cell.is_empty() {
                                None
                            } else {
                                Some(cell.to_string())
                            }
                        })
                        .collect();
                    if !cells.is_empty() {
                        text.push_str(&cells.join("\t"));
                        text.push('\n');
                    }
                }
            }
        }
        Ok(text)
    }
}

fn is_delimited_text(path: &Path, expected_ext: &str) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
        .is_some_and(|ext| ext == expected_ext)
}

fn read_delimited_file(path: &Path, delimiter: char) -> Result<String> {
    let raw = std::fs::read_to_string(path)?;
    let mut text = String::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let normalized = if delimiter == '\t' {
            trimmed.to_string()
        } else {
            trimmed
                .split(delimiter)
                .map(str::trim)
                .collect::<Vec<_>>()
                .join("\t")
        };
        text.push_str(&normalized);
        text.push('\n');
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn recognizes_supported_spreadsheet_extensions() {
        assert!(SpreadsheetExtractor.can_extract(Path::new("book.xlsx")));
        assert!(SpreadsheetExtractor.can_extract(Path::new("table.CSV")));
        assert!(!SpreadsheetExtractor.can_extract(Path::new("notes.txt")));
    }

    #[test]
    fn extracts_text_from_csv_fixture() {
        let base = unique_temp_dir();
        let file = base.join("sheet.csv");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, "name,score\nalice,10\nbob,8\n").expect("write csv");

        let text = SpreadsheetExtractor
            .extract(&file)
            .expect("csv extract should succeed");
        assert!(text.contains("alice"));
        assert!(text.contains("10"));

        cleanup_temp_dir(&base);
    }

    #[test]
    fn returns_extraction_error_for_invalid_spreadsheet() {
        let base = unique_temp_dir();
        let file = base.join("bad.xlsx");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, [1_u8, 2, 3]).expect("write invalid workbook");

        let result = SpreadsheetExtractor.extract(&file);
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
        std::env::temp_dir().join(format!("sotis-sheet-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
