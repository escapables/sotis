use std::path::Path;

use calamine::{open_workbook_auto, DataType, Reader};

use crate::error::Result;
use crate::extract::TextExtractor;

const SPREADSHEET_EXTENSIONS: &[&str] = &["xlsx", "xls", "ods", "csv"];

pub struct SpreadsheetExtractor;

impl TextExtractor for SpreadsheetExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| SPREADSHEET_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
    }

    fn extract(&self, path: &Path) -> Result<String> {
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
