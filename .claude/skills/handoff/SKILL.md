---
name: handoff
description: Orchestrate handoff between reviewer and coding subagent. Use when finishing a review session, updating handoff state, or preparing context for the next coding session.
---

# Handoff

Orchestrate session handoffs: read state, update docs, review code, commit.

## Steps

1. **Read state** — Read `docs/HANDOFF.md` and `docs/TODO.md` to understand current session state and active tasks.

2. **Review and update docs** — Following the doc hierarchy (ARCHITECTURE > PRIMARY_TODO > TODO > HANDOFF):
   - Analyze `git diff` and `git log` to understand what changed since last handoff.
   - Always update `docs/HANDOFF.md` per the WORKFLOW.md contract (under 60 lines, replace stale content, maintain section shape: Session, Completed, Verification Run, Open Risks / Blockers, Next Actions).
   - **Next Actions must always contain concrete directives for the coding agent** — e.g. which TODO item to pick up next, whether to reprioritise tasks, or specific instructions on how to address a denied approval request. Never leave Next Actions empty or vague.
   - Update `docs/PRIMARY_TODO.md` and `docs/TODO.md` when tasks have been completed or status has changed (mark DONE per TODO lifecycle in WORKFLOW.md).

3. **Determine change scope and commit** — Check `git status` to classify changes:
   - **Docs-only changes:** Run `bin/validate-docs`, then use `/committer` skill to commit all doc changes. Do not push.
   - **Code + docs changes:** Review code changes against the TODO list and project conventions (ARCHITECTURE.md, STYLE.md). Run relevant validation (`cargo build/test/clippy/fmt`). Present a summary of: code changes and their approval status, HANDOFF updates, and any other doc changes. Ask user for permission before using `/committer` skill. All approved changes (docs + code) go in a single commit. Do not push.

## Rules

- HANDOFF.md is always updated, even if nothing else changed.
- Never push — committer handles commit only.
- Doc hierarchy is authoritative: ARCHITECTURE > PRIMARY_TODO > TODO > HANDOFF.
- TODO items follow the lifecycle in WORKFLOW.md (mark DONE, never renumber).
- `bin/validate-docs` must pass before committing any doc changes.
- Next Actions in HANDOFF.md must always give the coding agent a clear next move: the next task to implement, tasks to reprioritise, or how to fix a denied approval.
