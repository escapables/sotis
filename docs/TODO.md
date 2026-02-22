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

### 10. DONE Regex Cross-Term
Task: Closed as by-design since tantivy RegexQuery inherently matches individual indexed terms not full text.
Scope:
- This is a fundamental inverted-index constraint, not a bug
- Single-term regex patterns work correctly
Done when:
- DONE Accepted as by-design, no code change needed

### 11. DONE Filename Regex Mode
Task: Add regex code path for filename search since `filename_scores()` always uses nucleo fuzzy matching regardless of QueryMode.
Scope:
- `crates/sotis-core/src/search.rs:206-225` — `filename_scores` method
- Add `query_mode` parameter, branch on it, use `regex::Regex` for regex path
- In GUI: when Regex + FilenameOnly are both active, grey out and disable the Fuzzy query mode button
Done when:
- DONE Regex mode with FilenameOnly returns correct regex matches on filenames
- DONE Fuzzy query mode button is greyed out and unclickable when Regex + FilenameOnly active

### 12. DONE Preview Highlight Fix
Task: Fix `build_highlight_job` which uses exact case-sensitive `match_indices` so highlights never appear for fuzzy queries.
Scope:
- `crates/sotis-gui/src/preview.rs:24-28` — highlight matching logic
- Switch to case-insensitive substring matching at minimum
- Consider highlighting actual matched terms from the index rather than raw query
Done when:
- DONE Fuzzy search results show highlighted matches in preview pane
- DONE Case-insensitive matching works for exact queries

### 13. DONE Decimal Size Filter
Task: Fix parse_megabytes_input which parses to u64 so fractional MB values silently apply no filter.
Scope:
- `crates/sotis-gui/src/filters.rs:71-78` — `parse_megabytes_input` function
- Parse as `f64`, convert to bytes with `(mb * 1_048_576.0) as u64`
Done when:
- DONE Typing `0.001` in Max MB filters out files larger than ~1 KB
- DONE Integer inputs still work

### 14. DONE ScrollArea ID Fix
Task: Add explicit ScrollArea IDs in results and preview panels to fix egui ID collision causing red error overlay.
Scope:
- `crates/sotis-gui/src/app.rs:283` — results ScrollArea
- `crates/sotis-gui/src/app.rs:316` — preview ScrollArea
- Add `.id_salt("results")` and `.id_salt("preview")`
Done when:
- DONE No red error text overlays in the GUI
- DONE Both panels scroll independently

### 15. DONE Dynamic File Filters
Task: Show file type checkboxes only for types that exist in the currently indexed folders.
Scope:
- After reindex or watcher update, collect the set of extensions present in the index
- `render_filters_panel` only renders checkboxes for file types that have at least one indexed file
- When folders change, the visible checkboxes update accordingly
Done when:
- DONE File type checkboxes reflect actual indexed content
- DONE Adding a folder with PDFs makes the PDF checkbox appear
- DONE Removing all PDFs hides the PDF checkbox

### 16. DONE Split Search Tests
Task: Extract test helpers from search module to bring the file under the 500 line limit.
Scope:
- `crates/sotis-core/src/search.rs` is 509 lines, limit is ~500
- Move `build_index`, `unique_temp_dir`, `cleanup_temp_dir` helpers to a shared test utility or a `tests/` submodule
- Alternatively move the `#[cfg(test)] mod tests` block to a sibling file
Done when:
- DONE `search.rs` is under 500 lines
- DONE All existing search tests still pass

### 17. DONE ODT Format Support
Task: Add text extraction for ODT files following the existing TextExtractor trait pattern.
Scope:
- New `crates/sotis-core/src/extract/odt.rs` implementing `TextExtractor`
- ODT is a ZIP containing `content.xml`; extract and strip XML tags
- Register in `extract/mod.rs` extractor chain and add `odt` to filter extensions
- Unit tests with fixture file
Done when:
- DONE ODT files are indexed and searchable
- DONE Extractor handles corrupt ODT gracefully
- DONE File type filter includes ODT

### 18. Standalone Image OCR
Task: Add OCR-based text extraction for standalone image files (PNG, JPG, TIFF, BMP) via Tesseract, feature-gated behind `ocr` cargo feature.
Scope:
- New `crates/sotis-core/src/extract/image.rs` implementing `TextExtractor` for png/jpg/jpeg/tiff/tif/bmp using `tesseract` crate (0.15); graceful error handling when Tesseract unavailable
- Modify `crates/sotis-core/src/extract/mod.rs` — add `Image` variant to `ExtractorKind`, magic-byte detection (PNG/JPEG/TIFF), extension match arms, dispatch to `ImageExtractor`, gate behind `ocr_enabled` config
- Modify `crates/sotis-core/Cargo.toml` and workspace `Cargo.toml` — add `tesseract = "0.15"` as optional dep behind `ocr` feature, forward feature from workspace
- Modify `crates/sotis-core/src/config.rs` — add `ocr_enabled: bool` (default `false`) and `tessdata_path: Option<String>` to config
- Modify `crates/sotis-gui/src/filters.rs` — add image extensions to file type filter list
Done when:
- `cargo build --workspace` still compiles without OCR deps (no regression)
- `cargo build --workspace --features ocr` compiles with Tesseract
- `detect_extractor_kind` returns `Image` for `.png`/`.jpg`/`.tiff`
- `ocr_enabled = false` → image files yield no text (current behavior preserved)
- `cargo clippy --workspace --features ocr -- -D warnings` clean

### 19. Scanned PDF OCR
Task: Detect scanned PDFs (image-only) and fall back to OCR when text extraction yields nothing.
Scope:
- Modify `crates/sotis-core/src/extract/pdf.rs` — after `pdf_extract::extract_text_from_mem`, check if result near-empty (`text.trim().len() < 50`); if scanned and OCR enabled, render pages to raster via pdfium then OCR each page
- New `crates/sotis-core/src/extract/pdf_ocr.rs` — `fn ocr_scanned_pdf(path: &Path) -> Result<String>` using `pdfium-render` to iterate pages at 300 DPI grayscale, pass to Tesseract, concatenate results
- Modify `crates/sotis-core/Cargo.toml` and workspace `Cargo.toml` — add `pdfium-render = "0.8"` behind `ocr` feature
Done when:
- Scanned PDF with known text is indexed and searchable when OCR enabled
- Normal text PDFs still use fast `pdf_extract` path (no performance regression)
- `ocr_enabled = false` → scanned PDFs yield empty text (current behavior)
- pdfium load failure handled gracefully (log warning, skip OCR)

### 20. OCR Bundle Script
Task: Create a bundle script that produces a distributable directory with all OCR dependencies.
Scope:
- New `scripts/bundle.sh` — copies binary + `libpdfium.so` + `libtesseract.so` + `libleptonica.so` + `eng.traineddata` into distributable directory
- Sets `RPATH` on binary to look for libs in `./lib/` via `patchelf`; target layout: `sotis/sotis-gui`, `sotis/lib/`, `sotis/share/tessdata/`
Done when:
- Bundle script produces working directory on fresh system
- Binary finds shared libs and traineddata from relative paths
- Can run OCR search from bundled directory without system-installed Tesseract
