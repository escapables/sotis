---
summary: 'Active task execution checklist.'
read_when:
  - Starting a session.
  - Looking for what to work on next.
---

# TODO

## Completed

1–9 (v1.0), 10–14 (v1.1), 15–17 (v1.2), 18 (v1.2.1), 19 (v1.3 image OCR) — see PRIMARY_TODO for details.

## Active

### 20. Scanned PDF Fallback
Task: Tiered PDF text extraction — `pdf_extract` → pdfium text layer → Tesseract OCR (user-approved only). See PRIMARY_TODO v1.3 step 20 for full scope.
Status: In progress — revised approach, previous image-based OCR too slow.
Done when:
- PDFs with embedded text layers indexed fast via pdfium fallback
- Truly image-only PDFs flagged in GUI; OCR only on user approval
- No regression for normal text PDFs

### 21. OCR Bundle Script
Task: Bundle script for distributable directory with all OCR deps. See PRIMARY_TODO v1.3 step 21.
Done when:
- Bundle runs on fresh system without system-installed Tesseract

### 22. Loading Indicator
Task: Spinner/progress bar during indexing and search. See PRIMARY_TODO v1.4.
Done when:
- User sees feedback during long operations; no frozen UI

### 23. Folder File Picker
Task: Native folder picker dialog instead of manual path entry. See PRIMARY_TODO v1.4.
Done when:
- "Add Folder" opens OS file picker

### 24. Larger Preview Snippet
Task: Increase preview from 5 lines to ~30 lines. See PRIMARY_TODO v1.4.
Done when:
- Preview shows ~30 lines centered on match; highlighting works

### 25. Indexing Performance
Task: Parallelize OCR, cache results, batch writes. See PRIMARY_TODO v1.4.
Done when:
- 3-file index under 60s; no quality regression
