---
name: handoff
description: Receive coding agent's work order, review, issue next directives. Use when finishing a review session, updating handoff state, or preparing context for the next coding session.
---

# Handoff

Work order lifecycle: receive deliverables, review, issue next directives.

## Receive

1. **Read deliverables** — `docs/HANDOFF.md` Completed + Verification Run + Approval Request = what coding agent delivered. Read `docs/TODO.md` for task acceptance criteria.
2. **Understand scope** — `git diff` + `git log` to see what changed. Cross-check against TODO items claimed as done.

## Review

3. **Verify** — Run `cargo build/test/clippy/fmt` + `bin/validate-docs`. Compare results against coding agent's reported verification.
4. **Review code** — Check against TODO acceptance criteria, ARCHITECTURE.md, STYLE.md. Files <500 LOC.
5. **Verdict** — Approve or deny each change. Denied = specific fix instructions in Next Actions.

## Issue

6. **Update docs** — Doc hierarchy: ARCHITECTURE > PRIMARY_TODO > TODO > HANDOFF.
   - Update `docs/HANDOFF.md` per WORKFLOW.md contract (<60 lines, replace stale content). Sections: Session, Completed, Verification Run, Open Risks / Blockers, Next Actions.
   - **Next Actions: always concrete directives for coding agent** — which TODO next, reprioritise, or how to fix a denied approval. Never empty/vague.
   - Mark DONE in TODO.md / PRIMARY_TODO.md when tasks accepted.

7. **Classify & commit** — `git status`:
   - **Docs-only:** `bin/validate-docs`, then `/committer`. No push.
   - **Code + docs:** Summarise: code approval status, HANDOFF updates, other doc changes. Ask user before `/committer`. Single commit. No push.

## Rules

- HANDOFF.md always updated, even if nothing else changed.
- Never push — committer handles commit only.
- Doc hierarchy authoritative: ARCHITECTURE > PRIMARY_TODO > TODO > HANDOFF.
- TODO lifecycle per WORKFLOW.md (mark DONE, never renumber).
- `bin/validate-docs` must pass before any doc commit.
- Next Actions must give coding agent a clear next move.
