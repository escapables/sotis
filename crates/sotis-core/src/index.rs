use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use tantivy::collector::TopDocs;
use tantivy::query::TermQuery;
use tantivy::schema::{Field, IndexRecordOption, Schema, Value, INDEXED, STORED, STRING, TEXT};
use tantivy::{doc, Index, IndexReader, IndexWriter, TantivyDocument, Term};

use crate::config;
use crate::error::{Error, Result};
use crate::extract;
use crate::scanner::ScanResult;

/// Stats emitted by [`SearchIndex::build_from_scan`].
#[derive(Debug, Default)]
pub struct BuildStats {
    pub added: usize,
    pub skipped: usize,
    pub errors: Vec<(PathBuf, String)>,
}

#[derive(Clone, Copy)]
struct Fields {
    path: Field,
    filename: Field,
    content: Field,
    modified: Field,
    size: Field,
    ext: Field,
}

/// Manages the tantivy search index.
pub struct SearchIndex {
    index_path: PathBuf,
    index: Index,
    reader: IndexReader,
    fields: Fields,
}

impl SearchIndex {
    /// Open or create an index in the default XDG data location.
    pub fn open_default() -> Result<Self> {
        let index_path = config::data_dir().join("index");
        Self::open(&index_path)
    }

    /// Open or create an index at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path).map_err(|source| {
            Error::Index(format!(
                "failed to create index directory {}: {source}",
                path.display()
            ))
        })?;

        let schema = Self::schema();
        let directory = tantivy::directory::MmapDirectory::open(path)
            .map_err(|err| Error::Index(format!("failed to open index directory: {err}")))?;
        let index = Index::open_or_create(directory, schema)?;
        let reader = index.reader()?;
        let fields = Self::fields(index.schema())?;

        Ok(Self {
            index_path: path.to_path_buf(),
            index,
            reader,
            fields,
        })
    }

    /// Returns the on-disk index path.
    pub fn index_path(&self) -> &Path {
        &self.index_path
    }

    /// Add a document to the index by extracting content from the given file path.
    pub fn add_document(&mut self, path: &Path) -> Result<()> {
        let index_doc = IndexedDoc::from_path(path)?;
        let mut writer: IndexWriter<TantivyDocument> = self.index.writer(50_000_000)?;
        writer.add_document(doc!(
            self.fields.path => index_doc.path,
            self.fields.filename => index_doc.filename,
            self.fields.content => index_doc.content,
            self.fields.modified => index_doc.modified,
            self.fields.size => index_doc.size,
            self.fields.ext => index_doc.ext,
        ))?;
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    /// Remove a document from the index by full file path.
    pub fn remove_document(&mut self, path: &Path) -> Result<()> {
        let path_text = path.to_string_lossy().into_owned();
        let mut writer: IndexWriter<TantivyDocument> = self.index.writer(50_000_000)?;
        writer.delete_term(Term::from_field_text(self.fields.path, &path_text));
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    /// Update a document if it is missing or stale compared to filesystem mtime.
    /// Returns true if the index was modified.
    pub fn update_document(&mut self, path: &Path) -> Result<bool> {
        let fs_modified = modified_secs(path)?;

        if let Some(indexed_modified) = self.indexed_modified(path)? {
            if indexed_modified >= fs_modified {
                return Ok(false);
            }
        }

        self.remove_document(path)?;
        self.add_document(path)?;
        Ok(true)
    }

    /// Build or incrementally update the index from a scanner result.
    pub fn build_from_scan(&mut self, scan_result: &ScanResult) -> Result<BuildStats> {
        let mut stats = BuildStats {
            errors: scan_result.errors.clone(),
            ..BuildStats::default()
        };

        for file in &scan_result.files {
            match self.update_document(file) {
                Ok(true) => stats.added += 1,
                Ok(false) => stats.skipped += 1,
                Err(err) => stats.errors.push((file.clone(), err.to_string())),
            }
        }

        Ok(stats)
    }

    fn schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("path", STRING | STORED);
        schema_builder.add_text_field("filename", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT);
        schema_builder.add_u64_field("modified", INDEXED | STORED);
        schema_builder.add_u64_field("size", STORED);
        schema_builder.add_text_field("ext", STRING | STORED);
        schema_builder.build()
    }

    fn fields(schema: Schema) -> Result<Fields> {
        let get = |name| {
            schema.get_field(name).map_err(|err| {
                Error::Index(format!("missing field '{name}' in index schema: {err}"))
            })
        };

        Ok(Fields {
            path: get("path")?,
            filename: get("filename")?,
            content: get("content")?,
            modified: get("modified")?,
            size: get("size")?,
            ext: get("ext")?,
        })
    }

    fn indexed_modified(&self, path: &Path) -> Result<Option<u64>> {
        let searcher = self.reader.searcher();
        let path_text = path.to_string_lossy().into_owned();
        let query = TermQuery::new(
            Term::from_field_text(self.fields.path, &path_text),
            IndexRecordOption::Basic,
        );

        let top_docs = searcher.search(&query, &TopDocs::with_limit(1))?;
        let Some((_, doc_address)) = top_docs.into_iter().next() else {
            return Ok(None);
        };

        let document = searcher.doc::<TantivyDocument>(doc_address)?;
        let modified_value = document
            .get_first(self.fields.modified)
            .and_then(|value| value.as_u64())
            .ok_or_else(|| {
                Error::Index(format!(
                    "indexed document at {} is missing 'modified' field",
                    path.display()
                ))
            })?;

        Ok(Some(modified_value))
    }
}

struct IndexedDoc {
    path: String,
    filename: String,
    content: String,
    modified: u64,
    size: u64,
    ext: String,
}

impl IndexedDoc {
    fn from_path(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        let content = extract::extract_text(path)?;
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                Error::Index(format!("file has no valid UTF-8 name: {}", path.display()))
            })?;

        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();

        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            filename,
            content,
            modified: modified_secs(path)?,
            size: metadata.len(),
            ext,
        })
    }
}

fn modified_secs(path: &Path) -> Result<u64> {
    let modified = fs::metadata(path)?.modified().map_err(|source| {
        Error::Index(format!(
            "failed to read modified time for {}: {source}",
            path.display()
        ))
    })?;

    modified
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|err| {
            Error::Index(format!(
                "modified time before unix epoch for {}: {err}",
                path.display()
            ))
        })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::process;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use tantivy::collector::TopDocs;
    use tantivy::query::QueryParser;

    use crate::scanner::ScanResult;

    use super::*;

    #[test]
    fn open_creates_index_directory() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");

        let index = SearchIndex::open(&index_dir).expect("open index");

        assert!(index.index_path().exists());
        assert!(index.index_path().is_dir());

        cleanup_temp_dir(&base);
    }

    #[test]
    fn add_document_supports_content_search_round_trip() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");
        let file = base.join("note.txt");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, "alpha beta gamma").expect("write source file");

        let mut index = SearchIndex::open(&index_dir).expect("open index");
        index.add_document(&file).expect("add document");

        let searcher = index.reader.searcher();
        let query_parser = QueryParser::for_index(&index.index, vec![index.fields.content]);
        let query = query_parser.parse_query("beta").expect("parse query");
        let docs = searcher
            .search(&query, &TopDocs::with_limit(5))
            .expect("search index");

        assert_eq!(docs.len(), 1);

        cleanup_temp_dir(&base);
    }

    #[test]
    fn update_document_skips_when_file_is_not_stale() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");
        let file = base.join("same.txt");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, "first version").expect("write source file");

        let mut index = SearchIndex::open(&index_dir).expect("open index");
        index.add_document(&file).expect("add document");

        let updated = index.update_document(&file).expect("update document");
        assert!(!updated);

        cleanup_temp_dir(&base);
    }

    #[test]
    fn update_document_reindexes_when_file_mtime_changes() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");
        let file = base.join("stale.txt");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, "first version").expect("write source file");

        let mut index = SearchIndex::open(&index_dir).expect("open index");
        index.add_document(&file).expect("add document");

        thread::sleep(Duration::from_secs(1));
        fs::write(&file, "second version updated").expect("write updated file");

        let updated = index.update_document(&file).expect("update document");
        assert!(updated);

        let searcher = index.reader.searcher();
        let query_parser = QueryParser::for_index(&index.index, vec![index.fields.content]);
        let query = query_parser
            .parse_query("updated")
            .expect("parse query for new content");
        let docs = searcher
            .search(&query, &TopDocs::with_limit(5))
            .expect("search index");
        assert_eq!(docs.len(), 1);

        cleanup_temp_dir(&base);
    }

    #[test]
    fn remove_document_deletes_by_path() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");
        let file = base.join("remove.txt");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, "to be removed").expect("write source file");

        let mut index = SearchIndex::open(&index_dir).expect("open index");
        index.add_document(&file).expect("add document");
        index.remove_document(&file).expect("remove document");

        let searcher = index.reader.searcher();
        let query_parser = QueryParser::for_index(&index.index, vec![index.fields.content]);
        let query = query_parser.parse_query("removed").expect("parse query");
        let docs = searcher
            .search(&query, &TopDocs::with_limit(5))
            .expect("search index");
        assert!(docs.is_empty());

        cleanup_temp_dir(&base);
    }

    #[test]
    fn build_from_scan_counts_added_skipped_and_errors() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");
        let file = base.join("one.txt");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(&file, "scan me").expect("write source file");

        let mut index = SearchIndex::open(&index_dir).expect("open index");
        let scan = ScanResult {
            files: vec![file.clone(), base.join("missing.txt")],
            errors: vec![(base.join("walker-error"), "walk failed".to_string())],
        };

        let first = index.build_from_scan(&scan).expect("build from scan");
        assert_eq!(first.added, 1);
        assert_eq!(first.skipped, 0);
        assert_eq!(first.errors.len(), 2);

        let second_scan = ScanResult {
            files: vec![file],
            errors: Vec::new(),
        };
        let second = index
            .build_from_scan(&second_scan)
            .expect("incremental build from scan");
        assert_eq!(second.added, 0);
        assert_eq!(second.skipped, 1);

        cleanup_temp_dir(&base);
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("sotis-index-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
