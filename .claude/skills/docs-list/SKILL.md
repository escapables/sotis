---
name: docs-list
description: List and discover project documentation. Use at session start, when looking for relevant docs, or when checking doc health.
---

# Docs List

Discover documentation and check front-matter compliance.

## Usage

Run from project root:
```bash
node scripts/docs-list.mjs
```

## What It Does

1. Walks `docs/` recursively for `.md` files
2. Extracts front-matter (`summary`, `read_when`)
3. Prints formatted list with summaries and read-when hints
4. Flags files missing front-matter or summary

## After Running

- Honor `read_when` hints — if your task matches a hint, read that doc first
- Missing front-matter? Fix it or flag in HANDOFF.md
- Also run `bin/validate-docs` for full validation

## Front-Matter Format

```yaml
---
summary: 'Brief description of what this doc covers'
read_when:
  - When working on X
  - When debugging Y
---
```
