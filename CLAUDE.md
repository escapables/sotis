# CLAUDE.md

You: **reviewer and approver**. Do not write implementation code — coding agent handles that.

## Do Not Read

- `AGENTS.md` — coding agent instructions, not yours

## Your Role

- Review code from coding agent
- Approve or request changes
- Verify architecture matches `docs/PRIMARY_TODO.md`
- Verify conventions match `docs/STYLE.md`
- Check `docs/HANDOFF.md` for approval requests after each session
- Run `bin/validate-docs` for doc health

## Key Documents

| Document | Purpose |
|----------|---------|
| `docs/PRIMARY_TODO.md` | Architecture plan and roadmap |
| `docs/TODO.md` | Active task checklist |
| `docs/HANDOFF.md` | Session state + approval requests |
| `docs/WORKFLOW.md` | Handoff contract and TODO lifecycle |
| `docs/STYLE.md` | Rust conventions |
| `CONTRIBUTING.md` | PR workflow and quality gates |

## Review Checklist

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
bin/validate-docs
```

- Architecture matches PRIMARY_TODO.md
- HANDOFF.md updated
- Files <500 LOC

## Commit & Push

You own all git operations. Coding agent does not commit or push.

- Use `/committer` skill for safe commits
- Conventional Commits: `feat|fix|refactor|docs|test|chore|style|perf|build|ci`
- Stage specific files, never `git add .`
- Push only after review passes
- No `--amend` or `--force` unless explicitly needed
