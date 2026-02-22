use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use tantivy::collector::TopDocs;
use tantivy::query::{AllQuery, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, Schema, Value, INDEXED, STORED, STRING, TEXT};
use tantivy::{doc, Index, IndexReader, IndexWriter, TantivyDocument, Term};

use crate::config::{self, GeneralConfig};
use crate::error::{Error, Result};
use crate::extract;
use crate::scanner::ScanResult;

mod ocr_refresh;
use ocr_refresh::should_force_ocr_sensitive_refresh;

/// Stats emitted by [`SearchIndex::build_from_scan`].
#[derive(Debug, Default)]
pub struct BuildStats {
    pub added: usize,
    pub skipped: usize,
    pub errors: Vec<(PathBuf, String)>,
    pub ocr_pending: Vec<PathBuf>,
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

    /// Returns the number of currently indexed documents.
    pub fn doc_count(&self) -> usize {
        self.reader.searcher().num_docs() as usize
    }

    /// Returns all non-empty file extensions currently present in the index.
    pub fn indexed_extensions(&self) -> Result<HashSet<String>> {
        self.reader.reload()?;
        let searcher = self.reader.searcher();
        let limit = searcher.num_docs() as usize;
        if limit == 0 {
            return Ok(HashSet::new());
        }

        let hits = searcher.search(&AllQuery, &TopDocs::with_limit(limit))?;
        let mut extensions = HashSet::new();
        for (_, address) in hits {
            let document = searcher.doc::<TantivyDocument>(address)?;
            if let Some(ext) = document
                .get_first(self.fields.ext)
                .and_then(|value| value.as_str())
                .filter(|ext| !ext.is_empty())
            {
                extensions.insert(ext.to_string());
            }
        }

        Ok(extensions)
    }

    /// Add a document to the index by extracting content from the given file path.
    pub fn add_document(&mut self, path: &Path) -> Result<()> {
        let index_doc = IndexedDoc::from_path(path)?;
        self.add_indexed_doc(index_doc)
    }

    /// Add a document to the index using explicit OCR settings.
    pub fn add_document_with_config(
        &mut self,
        path: &Path,
        config: &GeneralConfig,
        pdf_ocr_approved: bool,
    ) -> Result<()> {
        let index_doc = IndexedDoc::from_path_with_config(path, config, pdf_ocr_approved)?;
        self.add_indexed_doc(index_doc)
    }

    fn add_indexed_doc(&mut self, index_doc: IndexedDoc) -> Result<()> {
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
        let config = config::Config::load()
            .map(|loaded| loaded.general)
            .unwrap_or_default();
        self.update_document_with_config(path, &config, config.ocr_enabled)
    }

    /// Update a document with explicit OCR settings.
    pub fn update_document_with_config(
        &mut self,
        path: &Path,
        config: &GeneralConfig,
        pdf_ocr_approved: bool,
    ) -> Result<bool> {
        if should_force_ocr_sensitive_refresh(path) {
            self.remove_document(path)?;
            self.add_document_with_config(path, config, pdf_ocr_approved)?;
            return Ok(true);
        }

        let fs_modified = modified_secs(path)?;

        if let Some(indexed_modified) = self.indexed_modified(path)? {
            if indexed_modified >= fs_modified {
                return Ok(false);
            }
        }

        self.remove_document(path)?;
        self.add_document_with_config(path, config, pdf_ocr_approved)?;
        Ok(true)
    }

    /// Build or incrementally update the index from a scanner result.
    pub fn build_from_scan(&mut self, scan_result: &ScanResult) -> Result<BuildStats> {
        let config = config::Config::load()
            .map(|loaded| loaded.general)
            .unwrap_or_default();
        self.build_from_scan_with_config(scan_result, &config, config.ocr_enabled)
    }

    /// Build or incrementally update the index from a scanner result with explicit OCR settings.
    pub fn build_from_scan_with_config(
        &mut self,
        scan_result: &ScanResult,
        config: &GeneralConfig,
        pdf_ocr_approved: bool,
    ) -> Result<BuildStats> {
        let mut stats = BuildStats {
            errors: scan_result.errors.clone(),
            ..BuildStats::default()
        };

        for file in &scan_result.files {
            match self.update_document_with_config(file, config, pdf_ocr_approved) {
                Ok(true) => stats.added += 1,
                Ok(false) => stats.skipped += 1,
                Err(err) => {
                    if extract::is_pdf_ocr_approval_required_error(&err) {
                        stats.ocr_pending.push(file.clone());
                    } else {
                        stats.errors.push((file.clone(), err.to_string()));
                    }
                }
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
        let config = config::Config::load()
            .map(|loaded| loaded.general)
            .unwrap_or_default();
        Self::from_path_with_config(path, &config, config.ocr_enabled)
    }

    fn from_path_with_config(
        path: &Path,
        config: &GeneralConfig,
        pdf_ocr_approved: bool,
    ) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        let content = extract::extract_text_with_pdf_ocr_approval(path, config, pdf_ocr_approved)?;
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
mod tests;
