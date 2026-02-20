---
name: fixissue
description: Fix an issue end-to-end with tests, changelog, commit, and closure. Use when fixing a bug, closing an issue, or resolving a problem report.
---

# Fix Issue

End-to-end issue resolution.

## Workflow

1. **Understand** — read the issue; reproduce if possible; find root cause
2. **Fix** — fix root cause, not symptoms; refactor if needed
3. **Test** — add regression test that fails before fix, passes after; run full suite:
   ```bash
   cargo test --workspace
   ```
4. **Verify** — build + lint + docs:
   ```bash
   cargo build --workspace
   cargo clippy --workspace -- -D warnings
   bin/validate-docs
   ```
5. **Commit** — use `/committer` skill; Conventional Commits (`fix: ...`)
6. **Close** — comment on issue with what changed; close if appropriate
   - No issue URL/number? Ask before closing.

## Checklist

- [ ] Root cause fixed (not band-aid)
- [ ] Regression test added + passing
- [ ] Build clean
- [ ] Committed with clear message
- [ ] Issue commented + closed
