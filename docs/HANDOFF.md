---
summary: 'Ephemeral per-session handoff state for cross-agent pickup.'
read_when:
  - Starting a new session.
  - Picking up where a previous agent left off.
---

# Handoff

## Session
2026-02-20 — Manual GUI testing complete. 5 bugs found and documented as TODOs #10–#14.

## Completed
- Built and launched sotis-gui, tested with `/tmp/sotis-test-data/` folder containing mixed file types
- Verified: app launch, folder add/remove, fuzzy search, reindex, file type filters, watcher updates
- Confirmed watcher detects new files and updates index automatically
- Documented 5 bugs in `docs/TODO.md` (#10–#14) and `docs/PRIMARY_TODO.md` (§8)

## Bugs Found (v1.1 — TODOs #10–#14)

| # | Bug | Severity | Files |
|---|-----|----------|-------|
| 10 | Regex can't match across word boundaries (tantivy per-term limitation) | Medium | `search.rs:194-197` |
| 11 | Filename search ignores Regex mode (always uses nucleo fuzzy) | Low | `search.rs:206-225` |
| 12 | Preview highlights never appear for fuzzy queries (exact case-sensitive match) | High | `preview.rs:24-28` |
| 13 | Size filter ignores decimal input (`u64` parse fails on `0.001`) | Medium | `filters.rs:71-78` |
| 14 | ScrollArea ID collision causes red error overlay in results/preview | Medium | `app.rs:283, 316` |

## What Passed
- App launch — clean start, status "Ready"
- Folder add/remove — config persists, reindex triggers
- Fuzzy search — typo-tolerant matching works
- Single-term regex — `.*zzy.*` matches correctly
- File type checkbox filters — narrow results correctly
- Watcher — detects new files, updates index and status bar

## Open Risks / Blockers
- Bug #12 (highlights) is the highest-severity issue — core UX feature is non-functional
- Bug #10 (regex cross-term) may require architectural discussion — tantivy limitation
- Last build time still displayed as raw Unix seconds
- Watcher events not debounced

## Next Actions
- Coding agent: fix TODOs #10–#14 (start with #14 and #13 as quick wins, then #12)
- Re-run manual GUI test after fixes
- Then address previously identified polish items (timestamp formatting, watcher debouncing)
