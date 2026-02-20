---
summary: 'Unified pickup/handoff workflow and execution checklist for cross-agent continuity.'
read_when:
  - Starting a session.
  - Handing off work.
  - Reviewing cross-agent continuity expectations.
---

# Workflow

`HANDOFF.md` is a short continuity snapshot for cross-agent pickup.

## Handoff Contract (Repo-Specific)

- Keep `docs/HANDOFF.md` under `60` total lines (including front matter).
- Replace stale content; do not accumulate long historical logs.
- Keep section shape: `Session`, `Completed`, `Verification Run`, `Open Risks / Blockers`, `Next Actions`.
- Keep `Completed` to short session deltas only (max `4` bullets).
- Keep `Verification Run` concrete (command + result).
- Keep `Next Actions` to `2-3` bullets with concrete commands or checks.

## TODO Item Lifecycle

When finishing a TODO item:
- Mark `DONE` in the task title: `### 3. DONE Harden Security`
- Mark each `Done when:` step as `DONE`: `- DONE Security hardened`
- Do **not** renumber the list; preserve original numbering

When adding a new TODO item:
- Append to the current list with the next sequential number
- Do **not** renumber existing items

Format example:
```markdown
### 3. DONE Harden Security
Task: Implement security hardening measures.
Scope:
- Review authentication flows
- Add rate limiting
Done when:
- DONE Authentication reviewed
- DONE Rate limiting implemented
```
