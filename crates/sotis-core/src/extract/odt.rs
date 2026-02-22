use std::fs::File;
use std::io::Read;
use std::path::Path;

use zip::ZipArchive;

use crate::error::{Error, Result};
use crate::extract::TextExtractor;

pub struct OdtExtractor;

impl TextExtractor for OdtExtractor {
    fn can_extract(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("odt"))
    }

    fn extract(&self, path: &Path) -> Result<String> {
        let file = File::open(path).map_err(|source| Error::Extraction {
            path: path.to_path_buf(),
            message: format!("failed to open ODT file: {source}"),
        })?;

        let mut archive = ZipArchive::new(file).map_err(|source| Error::Extraction {
            path: path.to_path_buf(),
            message: format!("failed to read ODT archive: {source}"),
        })?;

        let mut content_xml =
            archive
                .by_name("content.xml")
                .map_err(|source| Error::Extraction {
                    path: path.to_path_buf(),
                    message: format!("failed to find content.xml in ODT: {source}"),
                })?;

        let mut xml = String::new();
        content_xml
            .read_to_string(&mut xml)
            .map_err(|source| Error::Extraction {
                path: path.to_path_buf(),
                message: format!("failed to read ODT content.xml: {source}"),
            })?;

        Ok(strip_xml_tags(&xml))
    }
}

fn strip_xml_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }

    decode_xml_entities(&out)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_xml_entities(input: &str) -> String {
    input
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zip::write::SimpleFileOptions;

    use super::*;

    #[test]
    fn recognizes_odt_extension() {
        assert!(OdtExtractor.can_extract(Path::new("doc.odt")));
        assert!(OdtExtractor.can_extract(Path::new("doc.ODT")));
        assert!(!OdtExtractor.can_extract(Path::new("doc.docx")));
    }

    #[test]
    fn extracts_text_from_content_xml() {
        let base = unique_temp_dir();
        let file = base.join("sample.odt");
        fs::create_dir_all(&base).expect("create temp dir");
        write_odt(
            &file,
            r#"<office:document-content><text:p>Hello <text:span>ODT</text:span> &amp; world</text:p></office:document-content>"#,
        );

        let text = OdtExtractor.extract(&file).expect("extract odt");
        assert!(text.contains("Hello"));
        assert!(text.contains("ODT"));
        assert!(text.contains("& world"));

        cleanup_temp_dir(&base);
    }

    #[test]
    fn returns_extraction_error_for_invalid_odt() {
        let base = unique_temp_dir();
        let file = base.join("broken.odt");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, b"not-a-zip-archive").expect("write invalid odt");

        let result = OdtExtractor.extract(&file);
        assert!(matches!(result, Err(Error::Extraction { .. })));

        cleanup_temp_dir(&base);
    }

    fn write_odt(path: &Path, content_xml: &str) {
        let file = File::create(path).expect("create odt file");
        let mut writer = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        writer
            .start_file("mimetype", options)
            .expect("start mimetype file");
        writer
            .write_all(b"application/vnd.oasis.opendocument.text")
            .expect("write mimetype");

        writer
            .start_file("content.xml", options)
            .expect("start content.xml");
        writer
            .write_all(content_xml.as_bytes())
            .expect("write content.xml");

        writer.finish().expect("finish odt archive");
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("sotis-odt-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
