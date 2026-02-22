---
summary: 'Active task execution checklist.'
read_when:
  - Starting a session.
  - Looking for what to work on next.
---

# TODO

### 1. DONE Scanned PDF Fallback
Task: Tiered PDF text extraction (`pdf_extract` -> pdfium text layer -> user-approved Tesseract OCR).
Scope:
- Refactor PDF extraction into tiered fallback flow
- Surface OCR-pending PDFs to GUI with explicit approval action
- Keep normal text PDF extraction behavior unchanged
Done when:
- DONE PDFs with embedded text layers index fast via pdfium fallback
- DONE Image-only PDFs are flagged in GUI and OCR runs only after approval click
- DONE No regression for normal text PDFs in build/test/clippy

### 2. DONE OCR Bundle Script
Task: Bundle a distributable directory containing all OCR dependencies.
Scope:
- Package binary with runtime OCR libraries and traineddata
- Validate startup on fresh system
Done when:
- DONE Bundle runs on fresh system without system-installed Tesseract

### 3. DONE Pdfium Fallback Bug
Task: Fix tier-2 pdfium text extraction failing for certain PDFs that show as OCR-pending instead of indexed.
Scope:
- Debug why `pdfium_extract_text()` fails or returns empty for this PDF (library not loading? text encoding issue?)
- Add logging to `pdf.rs` fallback chain so each tier's result is visible in stderr
- Add regression test with a PDF where `pdf_extract` fails but pdfium succeeds
Done when:
- DONE bundler/runtime path search includes executable-relative pdfium locations; no cwd dependency
- DONE fallback chain has visible diagnostic output across all tiers
- DONE regression test covers `pdf_extract` failure with pdfium recovery path

### 4. DONE Manual Search Trigger
Task: Replace live-as-you-type search with explicit submit via Search button or Enter key.
Scope:
- Remove auto-search on text change; only trigger search on button click or Enter keypress
- Add a "Search" button next to the search bar
- Bind Enter key in the search input to trigger search
Done when:
- DONE Typing does not trigger search until user clicks Search or presses Enter
- DONE Enter key and Search button both trigger the same background search action

### 5. DONE Loading Indicator
Task: Add a loading indicator for indexing and search operations.
Scope:
- Add progress feedback state to long-running operations
- Ensure UI stays responsive
Done when:
- DONE Search and reindex run in background worker threads so UI remains responsive
- DONE Search bar, folder panel, and status bar show spinner/loading state during active work

### 6. DONE Folder File Picker
Task: Replace manual folder path entry with a native folder picker dialog.
Scope:
- Replace manual path text input with OS picker flow
- Preserve add/remove/reindex behaviors
Done when:
- DONE "Add Folder" opens OS file picker

### 7. DONE Larger Preview Snippet
Task: Increase preview context from 5 lines to about 30 lines.
Scope:
- Expand snippet context window around match
- Keep highlight behavior correct
Done when:
- DONE Preview shows ~30 lines centered on match; highlighting works

### 8. DONE Indexing Performance
Task: Improve indexing performance with parallel OCR, caching, and batched writes.
Scope:
- Parallelize expensive OCR/document extraction paths
- Cache OCR output where safe
- Reduce write overhead in indexing loop
Done when:
- DONE 3-file index under 60s; no quality regression

### 9. Preview Match Navigation
Task: Replace snippet preview with full text, add Prev/Next match navigation with counter.
Scope:
- Remove snippet extraction; show full text with all keywords highlighted
- Add `find_all_match_positions` to `preview.rs`; add match state + Prev/Next UI to `app.rs`
- Show "Match X of Y" counter; scroll preview to current match on Prev/Next
- Update tests: replace snippet tests with match-position tests
Done when:
- Preview shows full extracted text with all keywords highlighted
- Prev/Next buttons navigate between matches; counter shows "Match X of Y"
- No matches shows "No matches" label; buttons disabled
