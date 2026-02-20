---
name: guardrails
description: Load global agent guardrails and work conventions. Use at session start to understand workspace rules, git practices, and communication style.
---

# Guardrails

## Work Style

- Telegraph; noun-phrases ok; drop grammar; min tokens in replies
- Keep files <500 LOC; split/refactor as needed
- Prefer end-to-end verify; if blocked, say what's missing

## Git

- Safe by default: `git status/diff/log` first
- Push only when user asks
- Conventional Commits: `feat|fix|refactor|build|ci|chore|docs|style|perf|test`
- No amend unless asked; no force push unless explicit
- Stage specific files, never `.`
- Big review: `git --no-pager diff --color=never`

## Docs

- Run `/docs-list` before coding; honor `read_when` hints
- Update docs when behavior/API changes — no ship without docs
- Add `read_when` hints on cross-cutting docs

## Build/Test

- Before handoff: run full gate (`cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`, `bin/validate-docs`)
- CI red: fix, push, repeat til green

## Critical Thinking

- Fix root cause, not band-aid
- Unsure: read more code; if still stuck, ask with short options
- Unrecognized changes: assume other agent; keep going; focus your changes

## Skills

- `/pickup` — context rehydration
- `/fixissue` — end-to-end issue resolution
- `/committer` — safe git commits
- `/docs-list` — documentation discovery
