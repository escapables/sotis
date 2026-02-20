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

### 3. DONE Text Extraction
Task: Implement text extractors for all supported file formats.
Scope:
- `TextExtractor` trait definition
- Plaintext, PDF, DOCX, EPUB, spreadsheet extractors
- MIME type detection for format routing
Done when:
- DONE Each extractor handles its format correctly
- DONE Graceful error handling for corrupt/unreadable files
- DONE Unit tests with fixture files

### 4. DONE Directory Scanner
Task: Implement recursive directory walking with configured folder support.
Scope:
- `scanner.rs` — walkdir-based file discovery
- Respect folder config (recursive flag, extension filters)
- Skip hidden files and common ignore patterns
Done when:
- DONE Scanner finds all matching files in configured folders
- DONE Extension filtering works correctly
- DONE Performance acceptable for 10k+ file directories

### 5. DONE Search Index
Task: Create tantivy index with document add/remove/update operations.
Scope:
- `index.rs` — schema definition, index creation
- Document indexing with extracted content
- Staleness detection via mtime comparison
Done when:
- DONE Can build index from scanned files
- DONE Can update index incrementally
- DONE Index persists to disk at XDG data path

### 6. DONE Search Modes
Task: Implement fuzzy and regex search across content and filenames.
Scope:
- `search.rs` — tantivy FuzzyTermQuery + RegexQuery for content
- nucleo-matcher integration for filename matching
- Weighted score merging (0.7 content + 0.3 filename)
Done when:
- DONE Fuzzy search: typo-tolerant results ranked by combined score
- DONE Regex search: pattern matching returns correct results
- DONE Filename-only and content-only modes work

### 7. DONE GUI Search Window
Task: Build the main search GUI with eframe and both search modes.
Scope:
- Search bar with fuzzy/regex mode toggle
- Results list with path, score, size, snippet
- Preview pane with keyword highlighting
Done when:
- DONE Typing a query returns live results
- DONE Fuzzy and regex modes both work from the GUI
- DONE Preview shows extracted text with matches highlighted

### 8. DONE GUI Filters
Task: Add folder management, file type filters, and status bar.
Scope:
- Add/remove indexed folders from GUI
- File type checkboxes, filesize range filter
- Status bar with index stats and result count
Done when:
- DONE Folders can be added and removed from the GUI
- DONE Filters narrow search results correctly
- DONE Status bar reflects current index state

### 9. DONE File Watcher
Task: Implement file system watching for automatic incremental re-indexing.
Scope:
- `watcher.rs` — notify-based file system watcher
- Incremental re-index on file create/modify/delete
- Watcher runs while GUI is open
Done when:
- DONE File changes detected and index updated automatically
- DONE No full re-index needed for single file changes
- DONE Watcher lifecycle tied to GUI lifecycle

### 10. Regex Cross-Term
Task: Fix tantivy RegexQuery to support multi-word regex patterns which currently fail because regex only matches individual indexed terms.
Scope:
- `crates/sotis-core/src/search.rs:194-197` — `RegexQuery::from_pattern` path
- Evaluate workaround: split multi-word regex into per-term boolean AND, or document single-term limitation in UI
Done when:
- Multi-word regex patterns return expected results
- Or UI clearly communicates that regex matches individual terms

### 11. Filename Regex Mode
Task: Add regex code path for filename search since `filename_scores()` always uses nucleo fuzzy matching regardless of QueryMode.
Scope:
- `crates/sotis-core/src/search.rs:206-225` — `filename_scores` method
- Add regex code path using standard `regex` crate, or disable Regex toggle when FilenameOnly is selected
Done when:
- Regex mode with FilenameOnly returns correct regex matches
- Or Regex toggle is disabled when FilenameOnly is active

### 12. Preview Highlight Fix
Task: Fix `build_highlight_job` which uses exact case-sensitive `match_indices` so highlights never appear for fuzzy queries.
Scope:
- `crates/sotis-gui/src/preview.rs:24-28` — highlight matching logic
- Switch to case-insensitive substring matching at minimum
- Consider highlighting actual matched terms from the index rather than raw query
Done when:
- Fuzzy search results show highlighted matches in preview pane
- Case-insensitive matching works for exact queries

### 13. Decimal Size Filter
Task: Fix parse_megabytes_input which parses to u64 so fractional MB values silently apply no filter.
Scope:
- `crates/sotis-gui/src/filters.rs:71-78` — `parse_megabytes_input` function
- Parse as `f64`, convert to bytes with `(mb * 1_048_576.0) as u64`
Done when:
- Typing `0.001` in Max MB filters out files larger than ~1 KB
- Integer inputs still work

### 14. ScrollArea ID Fix
Task: Add explicit id_source to ScrollArea widgets in results and preview panels to fix egui ID collision causing red error overlay.
Scope:
- `crates/sotis-gui/src/app.rs:283` — results ScrollArea
- `crates/sotis-gui/src/app.rs:316` — preview ScrollArea
- Add `.id_source("results")` and `.id_source("preview")`
Done when:
- No red error text overlays in the GUI
- Both panels scroll independently
