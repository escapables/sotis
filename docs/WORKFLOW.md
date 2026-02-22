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

- Coding agent may complete up to **2 TODO items** per session before requesting reviewer approval.
- Keep `docs/HANDOFF.md` under `60` total lines (including front matter).
- Replace stale content; do not accumulate long historical logs.
- Keep section shape: `Session`, `Completed`, `Verification Run`, `Open Risks / Blockers`, `Next Actions`.
- Keep `Completed` to short session deltas only (max `4` bullets).
- Keep `Verification Run` concrete (command + result).
- Keep `Next Actions` to `2-3` bullets with concrete commands or checks.

## Portability

Development must maintain a portable state at all times. The release build should run on a fresh system without requiring system-installed dependencies.

- All required runtime dependencies (shared libraries, data files) must live in the release folder
- Never rely on system-wide installs (`LD_LIBRARY_PATH`, system lib paths) during development or testing
- Test by running from the release directory, not via `cargo run` with environment overrides
- If a dependency cannot be bundled yet, document the gap as a blocker in HANDOFF.md
- **Reviewer**: rebuild portable bundle (`cargo build --release --workspace`) after refactors, feature additions, or program-breaking bug fixes. Verify clean compile before committing approval.

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
