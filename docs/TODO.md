---
summary: 'Active task execution checklist.'
read_when:
  - Starting a session.
  - Looking for what to work on next.
---

# TODO

### 1. DONE Project Skeleton
Task: Set up Cargo workspace with sotis-core and sotis-gui compiling.
Scope:
- Workspace `Cargo.toml` with sotis-core and sotis-gui
- Skeleton `lib.rs` and `main.rs` for each crate
- All module files with placeholder types
Done when:
- DONE `cargo build --workspace` compiles cleanly
- DONE `cargo test --workspace` passes
- DONE `cargo clippy --workspace -- -D warnings` clean

### 2. DONE Configuration Layer
Task: Implement XDG-compliant config loading with TOML serialization.
Scope:
- `config.rs` — Config struct, folder entries, XDG path resolution
- `error.rs` — unified error type with thiserror
- TOML load/save with default creation on first run
Done when:
- DONE Config loads from `$XDG_CONFIG_HOME/sotis/config.toml`
- DONE Default config created on first run
- DONE Unit tests for path resolution and serialization

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

### 6. Fuzzy and Regex Search
Task: Implement fuzzy and regex search across content and filenames.
Scope:
- `search.rs` — tantivy FuzzyTermQuery + RegexQuery for content
- nucleo-matcher integration for filename matching
- Weighted score merging (0.7 content + 0.3 filename)
Done when:
- Fuzzy search: typo-tolerant results ranked by combined score
- Regex search: pattern matching returns correct results
- Filename-only and content-only modes work

### 7. GUI Search Window
Task: Build the main search GUI with eframe and both search modes.
Scope:
- Search bar with fuzzy/regex mode toggle
- Results list with path, score, size, snippet
- Preview pane with keyword highlighting
Done when:
- Typing a query returns live results
- Fuzzy and regex modes both work from the GUI
- Preview shows extracted text with matches highlighted

### 8. GUI Filters and Folders
Task: Add folder management, file type filters, and status bar.
Scope:
- Add/remove indexed folders from GUI
- File type checkboxes, filesize range filter
- Status bar with index stats and result count
Done when:
- Folders can be added and removed from the GUI
- Filters narrow search results correctly
- Status bar reflects current index state

### 9. File Watcher
Task: Implement file system watching for automatic incremental re-indexing.
Scope:
- `watcher.rs` — notify-based file system watcher
- Incremental re-index on file create/modify/delete
- Watcher runs while GUI is open
Done when:
- File changes detected and index updated automatically
- No full re-index needed for single file changes
- Watcher lifecycle tied to GUI lifecycle
