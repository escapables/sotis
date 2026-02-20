# AGENTS.md

You: **coding agent**. Write code, debug, test. Do not approve own work.

## Do Not Read

- `CLAUDE.md` or `.claude` dir — reviewer instructions, not yours

## Session Start

1. Run `pickup` skill or manually:
   - `docs/HANDOFF.md` — pick up last session
   - `docs/TODO.md` — current task
   - `docs/PRIMARY_TODO.md` — architecture ref
   - `docs/STYLE.md` — conventions
2. Run `node scripts/docs-list.mjs` — discover docs, honor `read_when` hints

## Work Loop

1. Implement current TODO item per architecture plan
2. Verify:
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   bin/validate-docs
   ```
3. Update `docs/TODO.md` — mark DONE per `docs/WORKFLOW.md`
4. Update `docs/HANDOFF.md` — summarize work, leave **approval request** for reviewer

## Rules

- Follow architecture in `docs/PRIMARY_TODO.md`; flag deviations in HANDOFF.md
- Follow `docs/STYLE.md`; files <500 LOC; split if needed
- HANDOFF.md <60 lines; replace stale content
- Do not commit, push, merge, or approve own work — leave approval requests
- Blocked: document in HANDOFF.md, move to next unblocked task
- Bugs: add regression test
- Fix root cause, not band-aid

## Skills

- `pickup` — context rehydration at session start
- `fixissue` — end-to-end issue resolution
- `docs-list` — documentation discovery

## Key Paths

| Path | What |
|------|------|
| `crates/sotis-core/src/` | Core library (index, search, scanner, extract) |
| `crates/sotis-cli/src/` | CLI binary |
| `crates/sotis-gui/src/` | GUI binary (egui/eframe) |
| `tests/integration/` | Integration tests |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tantivy` | Full-text search engine |
| `nucleo-matcher` | Fuzzy filename matching |
| `eframe` | GUI framework (egui) |
| `clap` | CLI argument parsing |
| `pdf-extract` | PDF text extraction |
| `dotext` | DOCX text extraction |
| `epub` | EPUB text extraction |
| `calamine` | Spreadsheet reading |
| `walkdir` | Recursive directory traversal |
| `notify` | File system watching |
| `serde` / `toml` | Configuration serialization |
| `rayon` | Parallel processing |
