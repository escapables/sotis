---
summary: 'Rust coding conventions for sotis.'
read_when:
  - Writing new code.
  - Reviewing a pull request.
---

# Style

## Formatting

- All code must pass `cargo fmt`. No exceptions.
- Run `cargo fmt --all` before committing.
- All code must pass `cargo clippy -- -D warnings`. No exceptions.

## Naming

- **Crates:** lowercase with hyphens (`sotis-core`, `sotis-cli`)
- **Modules:** lowercase, single word (`config`, `search`, `extract`)
- **Types:** PascalCase (`SearchResult`, `TextExtractor`, `IndexConfig`)
- **Functions:** snake_case (`build_index`, `extract_text`)
- **Constants:** SCREAMING_SNAKE_CASE (`MAX_FILE_SIZE`, `DEFAULT_PORT`)
- **Acronyms in types:** PascalCase (`PdfExtractor`, not `PDFExtractor`)

## Error Handling

- Use `thiserror` for error types in sotis-core.
- Use `anyhow` for error propagation in binaries (CLI, GUI).
- Return `Result<T>`, don't `unwrap()` or `panic!` except in tests.
- Provide context with `.context("doing X")` (anyhow) or descriptive error variants.

## File Size

- Keep files under ~500 LOC. Split/refactor when approaching the limit.

## Project Layout

- `crates/sotis-core/` — all business logic, reusable as a library
- `crates/sotis-cli/` — thin CLI wrapper, parse args, call core, format output
- `crates/sotis-gui/` — thin GUI wrapper, call core, render with egui
- Keep binary crates thin — business logic belongs in core

## Imports

Group and separate with blank lines:

```rust
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use sotis_core::config::Config;
use sotis_core::search::SearchResult;
```

Order: std, external crates, internal crates.

## Testing

- Tests live in the same file (`#[cfg(test)] mod tests`) or in `tests/` for integration
- Use `#[test]` functions with descriptive names: `fn search_with_typo_returns_fuzzy_match()`
- Test fixtures go in `tests/fixtures/`
- Run with: `cargo test --workspace`

## Comments

- Don't over-comment obvious code
- Do comment: public API (rustdoc), non-obvious algorithms, edge case handling
- Use `///` for public items, `//` for internal notes
- The search scoring algorithm deserves inline comments explaining each phase

## Unsafe

- No `unsafe` code without explicit justification in a comment
- Prefer safe abstractions from well-maintained crates
