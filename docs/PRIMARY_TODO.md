---
summary: 'Implementation roadmap with detailed steps for each release milestone.'
read_when:
  - Starting implementation of a new component.
  - Checking what to build next.
  - Reviewing milestone progress.
---

# Roadmap

## v1.0 Core

| Step | What | Result |
|------|------|--------|
| 1 | Project skeleton, Cargo workspace, all crates compile | Empty binaries that run |
| 2 | `config.rs` — XDG paths, TOML load/save, folder config | Configuration works |
| 3 | `error.rs` + `extract/` — text extractors for all formats | Can extract text from files |
| 4 | `scanner.rs` — directory walking, file discovery | Can find files in configured folders |
| 5 | `index.rs` — tantivy index create/add/remove | Can build search index |
| 6 | `search.rs` — fuzzy + regex search with tantivy + nucleo | Can search index both ways |
| 7 | GUI — basic search window with eframe | Usable app with search |
| 8 | GUI — folder management, filters, preview pane | Feature-complete GUI |
| 9 | `watcher.rs` — file system watching, incremental update | Auto-reindex on changes |

**Steps 1–6 = core library complete. Steps 7–9 = complete v1.**

---

## v1.1 Bug Fixes

From manual GUI testing.

| Step | What | Result |
|------|------|--------|
| 10 | Regex cross-term matching — tantivy RegexQuery only matches single terms | DONE: Closed as by-design (inverted-index constraint) |
| 11 | Filename regex — `filename_scores()` ignores QueryMode, always uses nucleo fuzzy | DONE: Regex path for filenames via `regex` crate; Fuzzy button greyed out when Regex + FilenameOnly |
| 12 | Preview highlights — `build_highlight_job` uses exact case-sensitive `match_indices`; broken for all fuzzy queries | DONE: Case-insensitive + fuzzy fallback highlights implemented |
| 13 | Size filter decimal input — `parse_megabytes_input` parses `u64`, rejects `0.001` | DONE: Fractional MB values parsed and applied |
| 14 | ScrollArea ID collision — results and preview panels share auto-generated egui ID | DONE: Unique ScrollArea IDs added for results/preview |

---

## v1.2 Polish

| Step | What | Result |
|------|------|--------|
| 15 | Dynamic file type filters — only show checkboxes for types present in indexed folders | Checkboxes reflect actual indexed content, update on reindex/watcher |
| 16 | Split search tests — extract test helpers to bring `search.rs` under 500 LOC | File under limit, all tests pass |
| 17 | ODT format support — text extraction for LibreOffice Writer files | ODT files indexed, searchable, and filterable |

---

## v1.2.1 UX Fix

| Step | What | Result |
|------|------|--------|
| 18 | Preview snippet context — replace full-document preview with 5-line snippet centered on first match | DONE: Preview shows focused context, not entire document |

---

## v1.3 Image OCR

All OCR functionality gated behind `ocr` cargo feature so the project still builds without C++ toolchain. Runtime gated behind `ocr_enabled: bool` config field (default `false`).

```toml
# crates/sotis-core/Cargo.toml
[features]
default = []
ocr = ["dep:tesseract", "dep:pdfium-render"]
```

- `cargo build --workspace` — builds without OCR (current behavior, no new deps)
- `cargo build --workspace --features ocr` — builds with OCR support

| Step | What | Result |
|------|------|--------|
| 19 | DONE: Standalone image OCR — Tesseract-based `ImageExtractor` for PNG/JPG/TIFF/BMP, feature-gated behind `ocr` cargo feature, `ocr_enabled` config flag | Image files indexed and searchable when OCR enabled; default build unaffected |
| 20 | Scanned PDF tiered fallback — see detailed scope below | In progress — revised approach |
| 21 | OCR bundled distribution — `scripts/bundle.sh` packages binary + libpdfium + libtesseract + libleptonica + traineddata | Distributable directory runs OCR on fresh systems |

### Step 20 — Scanned PDF Tiered Fallback (Revised)

Previous approach (render all pages to images → Tesseract) was too slow — 10+ min for a 276-page PDF, freezing the app. Many "scanned" PDFs already have embedded text layers that browsers read instantly via Ctrl+F.

**Tiered extraction:**
1. **Tier 1 (fast)**: `pdf_extract` crate — current default for normal text PDFs
2. **Tier 2 (fast)**: pdfium text extraction — when tier 1 returns garbage/whitespace, use `pdfium-render` to read the embedded text layer directly. Handles "scanned" PDFs with baked-in OCR text. Near-instant.
3. **Tier 3 (slow, user-approved)**: Tesseract image OCR — only for truly image-only PDFs where tiers 1 and 2 both return nothing. **Must prompt user in GUI** with warning that OCR will be slow and require manual approval before proceeding. Do not run automatically during indexing.

**Implementation:**
- `pdf.rs` — implement tiered fallback: try `pdf_extract`, check quality (trim + ratio), if bad try pdfium text extraction, if still empty flag file as OCR-pending
- `pdf_ocr.rs` — split into `pdfium_extract_text()` (tier 2, fast) and `ocr_scanned_pdf()` (tier 3, slow). Tier 3 only on explicit user approval
- GUI — when truly image-only PDFs found during indexing, show notification listing files needing OCR with estimated time and "Approve OCR" button. Do not block indexing.

### Key Risks

| Risk | Mitigation |
|------|-----------|
| Build requires `libtesseract-dev` + `clang` | Feature-gated — default build unaffected |
| `libpdfium.so` not in distro repos | Bundle pre-built binary from bblanchon/pdfium-binaries |
| OCR is slow (seconds per page vs ms for text extraction) | Index time warning in GUI; `ocr_enabled` off by default |
| Tesseract init fails at runtime (missing libs/data) | Graceful error: log warning, skip OCR, return empty text |
| pdfium dynamic load fails | Check at startup, disable scanned-PDF OCR if unavailable |

### OCR Verification

1. `cargo build --workspace` — still compiles without OCR deps (no regression)
2. `cargo build --workspace --features ocr` — compiles with Tesseract + pdfium
3. `cargo test --workspace --features ocr` — new tests pass
4. `cargo clippy --workspace --features ocr -- -D warnings` — clean
5. Manual test: index a folder with a scanned PDF and a PNG of text — both appear in search results
6. `ocr_enabled = false` — image files and scanned PDFs yield no text (current behavior preserved)
7. Bundle script produces working distributable directory on fresh system

---

## v1.4 UX Improvements

| Step | What | Result |
|------|------|--------|
| 22 | Loading indicator — spinner/progress bar during indexing and search operations | User sees clear feedback, no frozen UI |
| 23 | Folder file picker — native dialog for folder selection instead of manual path entry | Add Folder opens OS file picker |
| 24 | Increase preview snippet to 30 lines — more context around matches | Preview shows ~30 lines centered on match |
| 25 | Indexing performance — parallelize OCR, cache results, reduce DPI, batch writes | 3-file index under 60s, no quality regression |
