use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use tantivy::collector::TopDocs;
use tantivy::query::{AllQuery, BooleanQuery, FuzzyTermQuery, Occur, Query, RegexQuery};
use tantivy::schema::{Field, Schema, Value, INDEXED, STORED, STRING, TEXT};
use tantivy::{DocAddress, Index, IndexReader, TantivyDocument, Term};

use crate::config;
use crate::error::{Error, Result};

/// A single search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub filename: String,
    pub score: f32,
    pub snippet: Option<String>,
}

/// Search mode selector.
#[derive(Debug, Clone, Copy)]
pub enum SearchMode {
    /// Search both content and filenames (default).
    Combined,
    /// Search filenames only.
    FilenameOnly,
    /// Search file content only.
    ContentOnly,
}

/// Query interpretation mode for content search.
#[derive(Debug, Clone, Copy)]
pub enum QueryMode {
    /// Fuzzy content matching with Tantivy `FuzzyTermQuery`.
    Fuzzy,
    /// Regex content matching with Tantivy `RegexQuery`.
    Regex,
}

#[derive(Clone, Copy)]
struct Fields {
    path: Field,
    filename: Field,
    content: Field,
}

/// Search service over the Tantivy index.
pub struct SearchEngine {
    _index: Index,
    reader: IndexReader,
    fields: Fields,
}

impl SearchEngine {
    /// Open or create search index in the default XDG data location.
    pub fn open_default() -> Result<Self> {
        let index_path = config::data_dir().join("index");
        Self::open(&index_path)
    }

    /// Open or create search index at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path).map_err(|source| {
            Error::Search(format!(
                "failed to create search index directory {}: {source}",
                path.display()
            ))
        })?;

        let directory = tantivy::directory::MmapDirectory::open(path)
            .map_err(|err| Error::Search(format!("failed to open index directory: {err}")))?;
        let index = Index::open_or_create(directory, schema())?;
        let reader = index.reader()?;

        let schema = index.schema();
        let fields = Fields {
            path: schema
                .get_field("path")
                .map_err(|err| Error::Search(format!("missing field 'path': {err}")))?,
            filename: schema
                .get_field("filename")
                .map_err(|err| Error::Search(format!("missing field 'filename': {err}")))?,
            content: schema
                .get_field("content")
                .map_err(|err| Error::Search(format!("missing field 'content': {err}")))?,
        };

        Ok(Self {
            _index: index,
            reader,
            fields,
        })
    }

    /// Run a query and return ranked results.
    pub fn search(
        &self,
        query_text: &str,
        query_mode: QueryMode,
        search_mode: SearchMode,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if query_text.trim().is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        self.reader.reload()?;
        let searcher = self.reader.searcher();

        let mut docs: HashMap<PathBuf, Accumulator> = HashMap::new();

        if matches!(search_mode, SearchMode::Combined | SearchMode::ContentOnly) {
            let content_scores = self.content_scores(&searcher, query_text, query_mode, limit)?;
            apply_normalized_scores(&content_scores, &mut docs, ScoreChannel::Content);
        }

        if matches!(search_mode, SearchMode::Combined | SearchMode::FilenameOnly) {
            let filename_scores = self.filename_scores(&searcher, query_text)?;
            apply_normalized_scores(&filename_scores, &mut docs, ScoreChannel::Filename);
        }

        let mut results: Vec<SearchResult> = docs
            .into_values()
            .filter_map(|acc| {
                let score = match search_mode {
                    SearchMode::Combined => 0.7 * acc.content_score + 0.3 * acc.filename_score,
                    SearchMode::FilenameOnly => acc.filename_score,
                    SearchMode::ContentOnly => acc.content_score,
                };

                if score <= 0.0 {
                    return None;
                }

                Some(SearchResult {
                    path: acc.path,
                    filename: acc.filename,
                    score,
                    snippet: None,
                })
            })
            .collect();

        results.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.path.cmp(&right.path))
        });
        results.truncate(limit);

        Ok(results)
    }

    fn content_scores(
        &self,
        searcher: &tantivy::Searcher,
        query_text: &str,
        query_mode: QueryMode,
        limit: usize,
    ) -> Result<Vec<(DocData, f32)>> {
        let content_docs = match query_mode {
            QueryMode::Fuzzy => {
                let terms: Vec<String> = query_text
                    .split_whitespace()
                    .map(|term| term.to_ascii_lowercase())
                    .filter(|term| !term.is_empty())
                    .collect();

                if terms.is_empty() {
                    return Ok(Vec::new());
                }

                let clauses: Vec<(Occur, Box<dyn Query>)> = terms
                    .iter()
                    .map(|term| {
                        let fuzzy = FuzzyTermQuery::new_prefix(
                            Term::from_field_text(self.fields.content, term),
                            1,
                            true,
                        );
                        (Occur::Should, Box::new(fuzzy) as Box<dyn Query>)
                    })
                    .collect();
                let query = BooleanQuery::new(clauses);
                searcher.search(&query, &TopDocs::with_limit(limit))?
            }
            QueryMode::Regex => {
                let query = RegexQuery::from_pattern(query_text, self.fields.content)?;
                searcher.search(&query, &TopDocs::with_limit(limit))?
            }
        };

        content_docs
            .into_iter()
            .map(|(score, address)| self.load_doc(searcher, address).map(|doc| (doc, score)))
            .collect()
    }

    fn filename_scores(
        &self,
        searcher: &tantivy::Searcher,
        query_text: &str,
    ) -> Result<Vec<(DocData, f32)>> {
        let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
        let pattern = Pattern::parse(query_text, CaseMatching::Ignore, Normalization::Smart);
        let mut scratch = Vec::new();

        let mut scored = Vec::new();
        for address in all_doc_addresses(searcher)? {
            let doc = self.load_doc(searcher, address)?;
            let haystack = Utf32Str::new(&doc.filename, &mut scratch);
            if let Some(score) = pattern.score(haystack, &mut matcher) {
                scored.push((doc, score as f32));
            }
        }

        Ok(scored)
    }

    fn load_doc(&self, searcher: &tantivy::Searcher, address: DocAddress) -> Result<DocData> {
        let document = searcher.doc::<TantivyDocument>(address)?;

        let path = document
            .get_first(self.fields.path)
            .and_then(|value| value.as_str())
            .ok_or_else(|| Error::Search("indexed document missing string path".to_string()))?;
        let filename = document
            .get_first(self.fields.filename)
            .and_then(|value| value.as_str())
            .ok_or_else(|| Error::Search("indexed document missing string filename".to_string()))?;

        Ok(DocData {
            path: PathBuf::from(path),
            filename: filename.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
struct DocData {
    path: PathBuf,
    filename: String,
}

#[derive(Debug, Clone)]
struct Accumulator {
    path: PathBuf,
    filename: String,
    content_score: f32,
    filename_score: f32,
}

#[derive(Clone, Copy)]
enum ScoreChannel {
    Content,
    Filename,
}

fn apply_normalized_scores(
    source: &[(DocData, f32)],
    accumulators: &mut HashMap<PathBuf, Accumulator>,
    channel: ScoreChannel,
) {
    if source.is_empty() {
        return;
    }

    let max = source
        .iter()
        .map(|(_, score)| *score)
        .fold(0.0_f32, f32::max)
        .max(1.0);

    for (doc, raw_score) in source {
        let normalized = (*raw_score / max).clamp(0.0, 1.0);

        let entry = accumulators
            .entry(doc.path.clone())
            .or_insert_with(|| Accumulator {
                path: doc.path.clone(),
                filename: doc.filename.clone(),
                content_score: 0.0,
                filename_score: 0.0,
            });

        match channel {
            ScoreChannel::Content => entry.content_score = normalized,
            ScoreChannel::Filename => entry.filename_score = normalized,
        }
    }
}

fn all_doc_addresses(searcher: &tantivy::Searcher) -> Result<Vec<DocAddress>> {
    let limit = searcher.num_docs() as usize;
    if limit == 0 {
        return Ok(Vec::new());
    }

    let all = AllQuery;
    let hits = searcher.search(&all, &TopDocs::with_limit(limit))?;
    Ok(hits.into_iter().map(|(_, address)| address).collect())
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::index::SearchIndex;

    use super::*;

    #[test]
    fn fuzzy_search_is_typo_tolerant_for_content() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(base.join("alpha.txt"), "distributed architecture notes")
            .expect("write alpha file");
        fs::write(base.join("beta.txt"), "nothing related").expect("write beta file");

        build_index(&index_dir, &[base.join("alpha.txt"), base.join("beta.txt")]);

        let engine = SearchEngine::open(&index_dir).expect("open search engine");
        let results = engine
            .search("archtecture", QueryMode::Fuzzy, SearchMode::ContentOnly, 10)
            .expect("run fuzzy content search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "alpha.txt");

        cleanup_temp_dir(&base);
    }

    #[test]
    fn regex_search_matches_content() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");
        fs::create_dir_all(&base).expect("create temp dir");
        fs::write(base.join("one.txt"), "order id beta123").expect("write one file");
        fs::write(base.join("two.txt"), "order id gamma999").expect("write two file");

        build_index(&index_dir, &[base.join("one.txt"), base.join("two.txt")]);

        let engine = SearchEngine::open(&index_dir).expect("open search engine");
        let results = engine
            .search(
                "beta[0-9]{3}",
                QueryMode::Regex,
                SearchMode::ContentOnly,
                10,
            )
            .expect("run regex content search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "one.txt");

        cleanup_temp_dir(&base);
    }

    #[test]
    fn filename_and_content_modes_filter_sources() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");
        fs::create_dir_all(&base).expect("create temp dir");

        let filename_match = base.join("targetterm-file.txt");
        let content_match = base.join("plain.txt");
        fs::write(&filename_match, "unrelated content").expect("write filename match file");
        fs::write(&content_match, "contains targetterm in body").expect("write content match file");

        build_index(&index_dir, &[filename_match.clone(), content_match.clone()]);

        let engine = SearchEngine::open(&index_dir).expect("open search engine");

        let filename_only = engine
            .search("targetterm", QueryMode::Fuzzy, SearchMode::FilenameOnly, 10)
            .expect("run filename-only search");
        assert_eq!(filename_only.len(), 1);
        assert_eq!(filename_only[0].filename, "targetterm-file.txt");

        let content_only = engine
            .search("targetterm", QueryMode::Fuzzy, SearchMode::ContentOnly, 10)
            .expect("run content-only search");
        assert_eq!(content_only.len(), 1);
        assert_eq!(content_only[0].filename, "plain.txt");

        cleanup_temp_dir(&base);
    }

    #[test]
    fn combined_mode_merges_content_and_filename_scores() {
        let base = unique_temp_dir();
        let index_dir = base.join("index");
        fs::create_dir_all(&base).expect("create temp dir");

        let both = base.join("target-term.txt");
        let filename_only = base.join("target-name.txt");
        let content_only = base.join("plain.txt");

        fs::write(&both, "target appears in content").expect("write both file");
        fs::write(&filename_only, "no body match").expect("write filename file");
        fs::write(&content_only, "target appears in content too").expect("write content file");

        build_index(
            &index_dir,
            &[both.clone(), filename_only.clone(), content_only.clone()],
        );

        let engine = SearchEngine::open(&index_dir).expect("open search engine");
        let combined = engine
            .search("target", QueryMode::Fuzzy, SearchMode::Combined, 10)
            .expect("run combined search");

        assert!(!combined.is_empty());
        assert_eq!(combined[0].filename, "target-term.txt");

        cleanup_temp_dir(&base);
    }

    fn build_index(index_dir: &Path, files: &[PathBuf]) {
        let mut index = SearchIndex::open(index_dir).expect("open index");
        for file in files {
            index.add_document(file).expect("index file");
        }
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("sotis-search-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
