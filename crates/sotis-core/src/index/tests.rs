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

#[test]
fn indexed_extensions_returns_unique_non_empty_extensions() {
    let base = unique_temp_dir();
    let index_dir = base.join("index");
    fs::create_dir_all(&base).expect("create temp dir");

    let first = base.join("a.txt");
    let second = base.join("b.TXT");
    let third = base.join("sheet.csv");
    fs::write(&first, "alpha").expect("write first");
    fs::write(&second, "beta").expect("write second");
    fs::write(&third, "x,y").expect("write third");

    let mut index = SearchIndex::open(&index_dir).expect("open index");
    index.add_document(&first).expect("add first");
    index.add_document(&second).expect("add second");
    index.add_document(&third).expect("add third");

    let mut extensions: Vec<String> = index
        .indexed_extensions()
        .expect("read indexed extensions")
        .into_iter()
        .collect();
    extensions.sort();

    assert_eq!(extensions, vec!["csv".to_string(), "txt".to_string()]);

    cleanup_temp_dir(&base);
}

#[test]
fn build_from_scan_indexes_three_files_under_sixty_seconds() {
    let base = unique_temp_dir();
    let index_dir = base.join("index");
    fs::create_dir_all(&base).expect("create temp dir");

    let files = vec![
        base.join("one.txt"),
        base.join("two.txt"),
        base.join("three.txt"),
    ];
    for (index, file) in files.iter().enumerate() {
        fs::write(file, format!("file {index} content")).expect("write source file");
    }

    let scan = ScanResult {
        files,
        errors: Vec::new(),
    };
    let mut index = SearchIndex::open(&index_dir).expect("open index");

    let started = std::time::Instant::now();
    let stats = index.build_from_scan(&scan).expect("build from scan");
    let elapsed = started.elapsed();

    assert_eq!(stats.added, 3);
    assert!(
        elapsed < Duration::from_secs(60),
        "indexing 3 files took {:?}, expected under 60s",
        elapsed
    );

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
