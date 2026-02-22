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
| 19 | Standalone image OCR — Tesseract-based `ImageExtractor` for PNG/JPG/TIFF/BMP, feature-gated behind `ocr` cargo feature, `ocr_enabled` config flag | Image files indexed and searchable when OCR enabled; default build unaffected |
| 20 | Scanned PDF OCR fallback — pdfium-render pages to raster + Tesseract OCR when `pdf_extract` yields empty text | Scanned PDFs searchable; normal PDFs use fast path |
| 21 | OCR bundled distribution — `scripts/bundle.sh` packages binary + libpdfium + libtesseract + libleptonica + traineddata | Distributable directory runs OCR on fresh systems |

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
