---
summary: 'Ephemeral per-session handoff state for cross-agent pickup.'
read_when:
  - Starting a new session.
  - Picking up where a previous agent left off.
---

# Handoff

## Session
2026-02-22 — reviewer session: smoke test failed (OCR too slow, app froze). Revised #20 to tiered fallback approach. Docs updated, ready to commit.

## Completed
- Smoke test verdict: OCR reindex never finished (276-page scanned PDF, app froze). User confirmed `Inventing Reality` is the scanned PDF, `grundrisse` is non-scanned.
- Revised TODO #20 architecture: tiered fallback (`pdf_extract` → pdfium text layer → Tesseract with user approval). See PRIMARY_TODO v1.3 step 20.
- Condensed TODO.md: collapsed 19 completed items into summary line, active items are concise with PRIMARY_TODO pointers.
- Updated PRIMARY_TODO.md with detailed step 20 revised scope.

## Verification Run
- Previous session: build/test/clippy/fmt/docs PASS (default + OCR, 60 tests).
- No new code changes this session — docs only.

## Open Risks / Blockers
- Uncommitted changes: #19 code, #20 code (old approach — needs rework by coding agent), docs, manifests.
- `libpdfium.so` and `run-ocr.sh` in project root — not tracked, should be .gitignored or cleaned up.

## Next Actions
- **Reviewer**: commit current docs + code changes, push.
- **Coding agent**: rework #20 per revised tiered fallback in PRIMARY_TODO. Key change: use pdfium text extraction (fast) before Tesseract OCR (slow, user-approved only). Existing `pdf_ocr.rs` needs refactoring — split into `pdfium_extract_text()` (tier 2) and `ocr_scanned_pdf()` (tier 3). Add GUI prompt for tier 3 approval.
- After #20, proceed to #21 (bundle script), then v1.4 (#22–#25).
