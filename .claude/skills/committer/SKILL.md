---
name: committer
description: Safe git commit helper. Use when committing changes to ensure proper staging, validation, and Conventional Commit messages.
---

# Committer

Safe commit workflow with guardrails.

## Steps

1. **Check state** — `git status` and `git diff --stat` to see what changed
2. **Stage explicitly** — add specific files by name; never `git add .` or `git add -A`
3. **Validate** before commit:
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   bin/validate-docs
   ```
4. **Commit** — Conventional Commits format:
   - `feat:` new feature
   - `fix:` bug fix
   - `refactor:` code restructure
   - `docs:` documentation only
   - `test:` test additions/changes
   - `chore:` build/tooling/config
   - `style:` formatting only
   - `perf:` performance improvement
5. **Do not push** unless user explicitly asks

## Rules

- Never stage `.` — list specific files
- Non-empty commit message required
- Check `git diff --staged` before committing
- No `--amend` unless user asks
- No `--force` push ever unless user explicitly requests
