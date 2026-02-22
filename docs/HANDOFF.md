---
summary: 'Ephemeral per-session handoff state for cross-agent pickup.'
read_when:
  - Starting a new session.
  - Picking up where a previous agent left off.
---

# Handoff

## Session
2026-02-22 — v1.2 complete. TODO #18 (preview snippet) added as priority before OCR phase. OCR TODOs renumbered to #19–#21.

## Current Task — TODO #18 (Preview Snippet Context)
**Priority — do this before OCR work.**

The preview pane currently shows the entire extracted document (up to 10k chars). Replace with a focused 5-line snippet centered on the first match.

### What to change
- `crates/sotis-gui/src/preview.rs` — add `fn extract_snippet(text: &str, query: &str, context_lines: usize) -> String` that finds the first case-insensitive or fuzzy match byte offset, maps it to a line number, returns 5 lines (2 above, match line, 2 below)
- `crates/sotis-gui/src/app.rs` `select_result()` (line 412) — after `extract::extract_text()`, call `extract_snippet()` instead of truncating to 10k chars
- Edge cases: empty query or no match → show first 5 lines; document shorter than 5 lines → show all

### Acceptance
- Preview shows max 5 lines, match on line 3
- Highlighting still works within the snippet
- Empty query falls back to first 5 lines

## Pipeline
1. **TODO #18** — preview snippet context (priority, do first)
2. TODO #19 — standalone image OCR (v1.3)
3. TODO #20 — scanned PDF OCR (v1.3)
4. TODO #21 — OCR bundle script (v1.3)

## Next Actions
- Coding agent: implement TODO #18, then proceed to #19
