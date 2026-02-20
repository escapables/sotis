---
name: pickup
description: Rehydrate context when starting work. Use when picking up work, starting a task, or resuming.
---

# Pickup

Rehydrate context for coding session.

## Steps

1. **Docs discovery** — run `node scripts/docs-list.mjs`; honor `read_when` hints
2. **Read handoff** — `docs/HANDOFF.md` for last session state + reviewer feedback
3. **Read TODO** — `docs/TODO.md` for current task
4. **Architecture ref** — `docs/PRIMARY_TODO.md` if task requires context
5. **Repo state** — `git status -sb`; note any uncommitted work from previous session

## Output

Concise bullet summary:
- Branch + working directory state
- Current TODO item
- Reviewer feedback from HANDOFF.md (if any)
- Next 2-3 actions

Keep it short. Execute the first action.
