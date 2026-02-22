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
    fs::write(base.join("alpha.txt"), "distributed architecture notes").expect("write alpha file");
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
fn regex_mode_matches_filenames_in_filename_only_search() {
    let base = unique_temp_dir();
    let index_dir = base.join("index");
    fs::create_dir_all(&base).expect("create temp dir");

    let report_file = base.join("report-2025.md");
    let notes_file = base.join("notes.txt");
    fs::write(&report_file, "alpha content").expect("write report file");
    fs::write(&notes_file, "report-2025 in content only").expect("write notes file");

    build_index(&index_dir, &[report_file, notes_file]);

    let engine = SearchEngine::open(&index_dir).expect("open search engine");
    let results = engine
        .search(
            r"report-\d{4}\.md",
            QueryMode::Regex,
            SearchMode::FilenameOnly,
            10,
        )
        .expect("run filename regex search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].filename, "report-2025.md");

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
