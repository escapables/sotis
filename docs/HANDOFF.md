---
summary: 'Ephemeral per-session handoff state for cross-agent pickup.'
read_when:
  - Starting a new session.
  - Picking up where a previous agent left off.
---

# Handoff

## Session
2026-02-20 — Fixed TODOs #14, #13, #12 in order (GUI bugfix pass 1 of 2).

## Completed
- TODO #14: Added unique ScrollArea IDs via `.id_salt("results")` and `.id_salt("preview")` in `crates/sotis-gui/src/app.rs`
- TODO #13: Updated `parse_megabytes_input` to parse `f64` MB and convert to bytes; wired `apply_client_filters` to use byte values directly
- TODO #12: Reworked preview highlighting in `crates/sotis-gui/src/preview.rs`:
  case-insensitive token matching + fuzzy fallback (subsequence/Levenshtein over words)
- Added regression tests:
  - `filters::tests::parses_decimal_megabytes_input`
  - `preview::tests::highlights_case_insensitive_exact_matches`
  - `preview::tests::highlights_fuzzy_word_match_when_exact_token_is_missing`
- Updated docs:
  - `docs/TODO.md` marks #12/#13/#14 as DONE
  - `docs/PRIMARY_TODO.md` v1.1 table updated for #12/#13/#14 completion

## Verification Run
- `cargo build --workspace` ✅
- `cargo test --workspace` ✅
- `cargo clippy --workspace -- -D warnings` ✅
- `cargo fmt --all -- --check` ✅
- `bin/validate-docs` ✅

## Open Risks / Remaining Work
- Bug #10 (regex cross-term) may require architectural discussion — tantivy limitation
- Bug #11 (filename regex mode) still open

## Next Actions
- Implement TODO #10 (regex cross-term handling or explicit UI limitation)
- Implement TODO #11 (filename regex mode behavior)
- Re-run manual GUI test on desktop compositor after #10/#11

## Approval Request
- Reviewer: please verify TODOs #12/#13/#14 behavior in GUI and approve this bugfix batch so we can proceed to #10/#11.
