---
name: handoff
description: Receive work order from reviewer, execute, wrap up session. Use at session start to pick up directives and at session end to prepare handoff.
---

# Handoff

Work order lifecycle: receive directives, do work, update handoff state.

## Receive

1. **Read work order** — `docs/HANDOFF.md` Next Actions section = your directives. Read `docs/TODO.md` for task details.
2. **Understand scope** — Reviewer verdict, required fixes, which TODO to pick up. If unclear, check `docs/PRIMARY_TODO.md` for milestone context.
3. **Execute** — Implement per directives. Follow ARCHITECTURE.md + STYLE.md conventions.

## Wrap Up

4. **Run verification:**
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   bin/validate-docs
   ```
   Record pass/fail for HANDOFF.md.

5. **Update TODO.md** — Mark completed items DONE per WORKFLOW.md lifecycle. Preserve numbering.

6. **Update HANDOFF.md** — Replace stale content (<60 lines). Sections:
   - **Session:** date, branch, what was done
   - **Completed:** short deltas (max 4 bullets)
   - **Verification Run:** command + result
   - **Open Risks / Blockers:** anything unresolved
   - **Next Actions:** 2-3 concrete next steps
   - **Approval request** for reviewer describing code changes needing review.

7. **Update PRIMARY_TODO.md** — If milestone step completed, update status.

8. **Final check** — `git status` + `git diff --stat`. Summarise all work + doc updates. Do not commit.

## Rules

- HANDOFF.md always updated, even if nothing else changed.
- Never commit, push, or approve own work — leave for reviewer.
- Doc hierarchy authoritative: ARCHITECTURE > PRIMARY_TODO > TODO > HANDOFF.
- TODO lifecycle per WORKFLOW.md (mark DONE, never renumber).
- Verification runs before HANDOFF.md update (results go in Verification Run).
- Flag architecture deviations in Open Risks.
