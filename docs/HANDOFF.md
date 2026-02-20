---
summary: 'Ephemeral per-session handoff state for cross-agent pickup.'
read_when:
  - Starting a new session.
  - Picking up where a previous agent left off.
---

# Handoff

## Session
2026-02-20 — TODO #7 approved and committed

## Completed
- TODO #7 (GUI Search Window) reviewed, approved, committed as `e50ed15`
- All checks pass: build, test (35), clippy, fmt, validate-docs

## Open Risks / Blockers
- Schema duplicated between `index.rs` and `search.rs` — acceptable for now
- Preview highlighting is token-based literal matching (not regex-aware)

## Next Actions — TODO #8: GUI Filters and Folders
Implement in `crates/sotis-gui/src/app.rs` per PRIMARY_TODO.md §3:

1. **Folder management** — add/remove indexed folders from the GUI, persist to config via `config::Config` save
2. **Re-index trigger** — after folder add/remove, rebuild index via `SearchIndex::build_from_scan`
3. **File type filter** — checkboxes for supported extensions, filter results client-side or pass to search
4. **Filesize range filter** — min/max size inputs to narrow results
5. **Search mode selector** — expose `SearchMode::Combined`, `FilenameOnly`, `ContentOnly` in the UI
6. **Status bar** — show index stats (total docs, last build time) alongside result count

Keep `app.rs` under 500 LOC — extract a `filters.rs` or `panels.rs` in sotis-gui if needed.
Run full verification checklist before handing off. Update TODO.md when done.
