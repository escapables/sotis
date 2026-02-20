---
summary: 'Ephemeral per-session handoff state for cross-agent pickup.'
read_when:
  - Starting a new session.
  - Picking up where a previous agent left off.
---

# Handoff

## Session
2026-02-20 — TODO #3 and #4 approved and committed

## Completed
- TODO #3 (text extraction) and #4 (scanner) reviewed, approved, committed as `8365944`
- All checks pass: build, test (25), clippy, fmt, validate-docs

## Open Risks / Blockers
- None

## Next Actions — TODO #5: Search Index
Implement `crates/sotis-core/src/index.rs` per PRIMARY_TODO.md:

1. **tantivy schema** — define fields exactly as specified in PRIMARY_TODO.md §2:
   - `path` (STRING | STORED), `filename` (TEXT | STORED), `content` (TEXT, not stored)
   - `modified` (u64, INDEXED | STORED), `size` (u64, STORED), `ext` (STRING | STORED)
2. **Index creation** — create/open index at XDG data path (`config::data_dir()/index/`)
3. **Document add** — accept a file path, use `extract::extract_text()` to get content, build tantivy doc, add to index
4. **Document remove** — delete by path field
5. **Document update** — compare mtime, re-extract and re-index if stale
6. **Build from scan** — accept `ScanResult`, index all files, return stats (added/skipped/errors)
7. **Tests** — index creation, add/search round-trip, mtime staleness detection, remove

Run full verification checklist before handing off. Update TODO.md when done.
