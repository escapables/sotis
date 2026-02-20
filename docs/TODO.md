---
summary: 'Active task execution checklist.'
read_when:
  - Starting a session.
  - Looking for what to work on next.
---

# TODO

### 1. Project Skeleton
Task: Set up Cargo workspace with all three crates compiling as empty binaries.
Scope:
- Workspace `Cargo.toml` with sotis-core, sotis-cli, sotis-gui
- Skeleton `lib.rs` and `main.rs` for each crate
- All module files with placeholder types
Done when:
- `cargo build --workspace` compiles cleanly
- `cargo test --workspace` passes
- `cargo clippy --workspace -- -D warnings` clean

### 2. Configuration Layer
Task: Implement XDG-compliant config loading with TOML serialization.
Scope:
- `config.rs` — Config struct, folder entries, XDG path resolution
- `error.rs` — unified error type with thiserror
- TOML load/save with default creation on first run
Done when:
- Config loads from `$XDG_CONFIG_HOME/sotis/config.toml`
- Default config created on first run
- Unit tests for path resolution and serialization

### 3. Text Extraction
Task: Implement text extractors for all supported file formats.
Scope:
- `TextExtractor` trait definition
- Plaintext, PDF, DOCX, EPUB, spreadsheet extractors
- MIME type detection for format routing
Done when:
- Each extractor handles its format correctly
- Graceful error handling for corrupt/unreadable files
- Unit tests with fixture files

### 4. Directory Scanner
Task: Implement recursive directory walking with configured folder support.
Scope:
- `scanner.rs` — walkdir-based file discovery
- Respect folder config (recursive flag, extension filters)
- Skip hidden files and common ignore patterns
Done when:
- Scanner finds all matching files in configured folders
- Extension filtering works correctly
- Performance acceptable for 10k+ file directories

### 5. Search Index
Task: Create tantivy index with document add/remove/update operations.
Scope:
- `index.rs` — schema definition, index creation
- Document indexing with extracted content
- Staleness detection via mtime comparison
Done when:
- Can build index from scanned files
- Can update index incrementally
- Index persists to disk at XDG data path

### 6. Fuzzy Search
Task: Implement combined fuzzy search across content and filenames.
Scope:
- `search.rs` — tantivy FuzzyTermQuery for content
- nucleo-matcher integration for filename matching
- Weighted score merging (0.7 content + 0.3 filename)
Done when:
- Typo-tolerant search returns relevant results
- Filename-only and content-only modes work
- Results ranked by combined score

### 7. CLI Commands
Task: Wire up clap CLI with all search, index, and config commands.
Scope:
- search, index, add, remove, status, config commands
- Formatted output with match highlights
- Error messages with actionable suggestions
Done when:
- All commands from PRIMARY_TODO.md §3 implemented
- `sotis search` returns ranked results with snippets
- `sotis index` builds/updates index
