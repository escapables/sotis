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

### 3. Pdfium Fallback Bug
Task: Fix tier-2 pdfium text extraction failing for certain PDFs that show as OCR-pending instead of indexed.
Scope:
- Debug why `pdfium_extract_text()` fails or returns empty for this PDF (library not loading? text encoding issue?)
- Add logging to `pdf.rs` fallback chain so each tier's result is visible in stderr
- Add regression test with a PDF where `pdf_extract` fails but pdfium succeeds
Done when:
- grundrisse.pdf indexes without OCR approval
- Fallback chain has visible diagnostic output

### 4. Loading Indicator
Task: Add a loading indicator for indexing and search operations.
Scope:
- Add progress feedback state to long-running operations
- Ensure UI stays responsive
Done when:
- User sees feedback during long operations; no frozen UI

### 5. Folder File Picker
Task: Replace manual folder path entry with a native folder picker dialog.
Scope:
- Replace manual path text input with OS picker flow
- Preserve add/remove/reindex behaviors
Done when:
- "Add Folder" opens OS file picker

### 6. Larger Preview Snippet
Task: Increase preview context from 5 lines to about 30 lines.
Scope:
- Expand snippet context window around match
- Keep highlight behavior correct
Done when:
- Preview shows ~30 lines centered on match; highlighting works

### 7. Indexing Performance
Task: Improve indexing performance with parallel OCR, caching, and batched writes.
Scope:
- Parallelize expensive OCR/document extraction paths
- Cache OCR output where safe
- Reduce write overhead in indexing loop
Done when:
- 3-file index under 60s; no quality regression
