---
name: handoff
description: Wrap up a coding session and prepare handoff for reviewer. Use when finishing implementation work, before ending a session.
---

# Handoff

Wrap up coding session and prepare for reviewer.

## Steps

1. **Run verification** — Execute the full validation suite:
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   bin/validate-docs
   ```
   Record pass/fail results for HANDOFF.md.

2. **Update TODO.md** — Mark completed items DONE per WORKFLOW.md lifecycle (mark title and `Done when:` steps, preserve numbering).

3. **Update HANDOFF.md** — Replace stale content, maintain section shape per WORKFLOW.md contract (under 60 lines):
   - **Session:** date, branch, what was worked on
   - **Completed:** short session deltas (max 4 bullets)
   - **Verification Run:** command + result from step 1
   - **Open Risks / Blockers:** anything unresolved
   - **Next Actions:** 2-3 concrete next steps
   - Include an **approval request** for the reviewer describing what code changes need review.

4. **Update PRIMARY_TODO.md** — If a milestone step was completed, update its status.

5. **Final check** — Run `git status` and `git diff --stat` to summarize uncommitted changes. Present a summary of all work done and docs updated. Do not commit.

## Rules

- HANDOFF.md is always updated, even if nothing else changed.
- Never commit, push, or approve own work — leave approval requests for reviewer.
- Doc hierarchy is authoritative: ARCHITECTURE > PRIMARY_TODO > TODO > HANDOFF.
- TODO items follow the lifecycle in WORKFLOW.md (mark DONE, never renumber).
- Verification suite must run before updating HANDOFF.md (results go in Verification Run section).
- Flag any architecture deviations in HANDOFF.md Open Risks section.
