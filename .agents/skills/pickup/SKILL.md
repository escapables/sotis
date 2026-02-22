---
name: pickup
description: Rehydrate context when starting work. Use when picking up work, starting a task, or resuming.
---

# Pickup

Rehydrate context, read work-order state, determine turn, act.

## Gather

1. **AGENTS.md** — `./AGENTS.md` for agent guidance.
2. **Docs discovery** — `node scripts/docs-list.mjs`; honor `read_when` hints.
3. **Work-order state** — `docs/HANDOFF.md`: who acted last, what delivered, what pending.
4. **TODO** — `docs/TODO.md` for current task + acceptance criteria.
5. **Architecture** — `docs/ARCHITECTURE.md` if task requires context.
6. **Roadmap** — `docs/PRIMARY_TODO.md` for milestone context.
7. **Repo state** — `git status -sb`; note uncommitted work from previous session.

## Determine Turn

8. **Read HANDOFF.md sections:**
   - Next Actions with coding directives → **your turn**. Proceed to Receive phase (handoff skill).
   - Approval Request present → **reviewer's turn**. You delivered last, waiting for review. Do not start new work.
   - Ambiguous / stale → flag it. Summarise, ask for clarification.

## Act

9. **Your turn** → execute first directive from HANDOFF.md Next Actions. Use `/handoff` when done to wrap up.
10. **Not your turn** → summarise what's waiting. No action beyond summary.

## Output

Concise bullets:
- Branch + working directory state
- Current TODO item
- Reviewer feedback (if any)
- Whose turn + why
- Next 2-3 actions

Short. If your turn, execute first action.
