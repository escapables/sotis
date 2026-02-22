---
name: pickup
description: Rehydrate context when starting work. Use when picking up work, starting a task, getting context, or resuming work.
---

# Pickup

Quickly rehydrate context for this repo.

## Steps

1. **Docs discovery** — run `node scripts/docs-list.mjs` from project root; honor `read_when` hints
2. **Read handoff** — `docs/HANDOFF.md` for last session state + approval requests
3. **Read TODO** — `docs/TODO.md` for current task
4. **Architecture ref** — `docs/ARCHITECTURE.md` if reviewing design decisions
5. **Roadmap** — `docs/PRIMARY_TODO.md` for milestone steps and current phase
6. **Repo state** — `git status -sb`; check for local commits not pushed
7. **CI/PR** — if PR exists: `gh pr view --comments --files`
8. **Plan** — identify next 2-3 concrete actions as bullets

## Output

Concise bullet summary:
- Branch + working directory state
- PR/issue status (if any)
- Failing checks (if any)
- Current TODO item
- Next 2-3 actions

Keep it short. Execute the first action.
