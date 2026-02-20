---
summary: 'Full architecture spec for SOTIS — fuzzy file search for Linux.'
read_when:
  - Starting implementation of a new component.
  - Reviewing architecture decisions.
  - Checking how components interact.
---

# SOTIS — Architecture Plan

Portable, offline, native Linux app for fuzzy file search. Users pick folders to index, then search filenames and file contents with fuzzy matching and regex. Supports PDF, DOCX, EPUB, spreadsheets, and plain text. Ships as a single GUI binary.

---

## 1. Core Architecture

### Workspace Layout

```
crates/
├── sotis-core/     — index, search, scanner, watcher, text extraction
└── sotis-gui/      — GUI binary (egui/eframe)
```

### Search Strategy

- **tantivy** for content search — FuzzyTermQuery (Levenshtein) and RegexQuery
- **nucleo-matcher** for fuzzy filename matching (Smith-Waterman algorithm)
- Two search modes selectable in GUI: **Fuzzy** (default) and **Regex**
- Combined search mode: content score × 0.7 + filename score × 0.3
- Content is NOT stored in the index — only path, filename, and metadata stored
- On search hit, content is re-extracted on-demand for preview snippets

### XDG Paths

| What | Path |
|------|------|
| Config | `$XDG_CONFIG_HOME/sotis/config.toml` (default: `~/.config/sotis/`) |
| Index | `$XDG_DATA_HOME/sotis/index/` (default: `~/.local/share/sotis/`) |

Override with `$SOTIS_CONFIG` and `$SOTIS_DATA` env vars.

---

## 2. Crate: sotis-core

### Modules

| Module | Purpose |
|--------|---------|
| `config.rs` | Config loading/saving, TOML serialization, XDG path resolution |
| `error.rs` | Unified error type (`thiserror`) |
| `index.rs` | tantivy index creation, schema, document add/remove/update |
| `search.rs` | Query building, fuzzy + regex search, result ranking and merging |
| `scanner.rs` | Directory walking, file discovery, MIME detection |
| `watcher.rs` | File system watcher (notify crate), incremental re-index |
| `extract/` | Text extraction from various formats |

### Text Extraction (extract/)

| Module | Formats | Crate |
|--------|---------|-------|
| `plaintext.rs` | .txt, .md, .rs, .py, .json, etc. | std |
| `pdf.rs` | .pdf | pdf-extract |
| `docx.rs` | .docx | dotext |
| `epub.rs` | .epub | epub |
| `spreadsheet.rs` | .xlsx, .xls, .ods, .csv | calamine |

Each extractor implements a common `TextExtractor` trait:
```rust
pub trait TextExtractor {
    fn can_extract(&self, path: &Path) -> bool;
    fn extract(&self, path: &Path) -> Result<String>;
}
```

### tantivy Index Schema

```rust
// Fields stored in tantivy
schema.add_text_field("path", STRING | STORED);       // full path
schema.add_text_field("filename", TEXT | STORED);      // filename only
schema.add_text_field("content", TEXT);                // extracted text (indexed, NOT stored)
schema.add_u64_field("modified", INDEXED | STORED);    // mtime for staleness check
schema.add_u64_field("size", STORED);                  // file size
schema.add_text_field("ext", STRING | STORED);         // file extension
```

### Config (config.toml)

```toml
[general]
max_file_size_mb = 50

[[folders]]
path = "/home/user/documents"
recursive = true
extensions = []  # empty = all supported

[[folders]]
path = "/home/user/projects"
recursive = true
extensions = [".rs", ".md", ".txt"]
```

---

## 3. Crate: sotis-gui

egui/eframe application. Single window:

- **Search bar** at top — type to search, results update live
- **Search mode toggle** — Fuzzy (default) / Regex
- **Filter panel** — file type checkboxes, filesize range, filename-only / content-only
- **Results list** — path, score, file size, snippet preview
- **Preview pane** — extracted text with keyword highlighting, page navigation
- **Folder management** — add/remove indexed folders
- **Status bar** — index stats, result count, last update time

No GTK/Qt dependency — pure OpenGL via eframe's glow backend.

---

## 4. Build Targets

| Target | Command | Notes |
|--------|---------|-------|
| Debug | `cargo build --workspace` | Core lib + GUI binary |
| Release | `cargo build --release --workspace` | Optimized |
| GUI only | `cargo build -p sotis-gui` | Needs OpenGL |

---

## 5. Implementation Sequence

| Step | What | Result |
|------|------|--------|
| 1 | Project skeleton, Cargo workspace, all crates compile | Empty binaries that run |
| 2 | `config.rs` — XDG paths, TOML load/save, folder config | Configuration works |
| 3 | `error.rs` + `extract/` — text extractors for all formats | Can extract text from files |
| 4 | `scanner.rs` — directory walking, file discovery | Can find files in configured folders |
| 5 | `index.rs` — tantivy index create/add/remove | Can build search index |
| 6 | `search.rs` — fuzzy + regex search with tantivy + nucleo | Can search index both ways |
| 7 | GUI — basic search window with eframe | Usable app with search |
| 8 | GUI — folder management, filters, preview pane | Feature-complete GUI |
| 9 | `watcher.rs` — file system watching, incremental update | Auto-reindex on changes |

**Steps 1–6 = core library complete. Steps 7–9 = complete v1.**

### v1.1 Bug Fixes (from manual GUI testing)

| Step | What | Result |
|------|------|--------|
| 10 | Regex cross-term matching — tantivy RegexQuery only matches single terms; multi-word patterns like `fuzzy.*search` fail | Regex works across word boundaries or limitation clearly communicated in UI |
| 11 | Filename regex — `filename_scores()` ignores QueryMode, always uses nucleo fuzzy | Regex toggle works for filename search, or is disabled when FilenameOnly active |
| 12 | Preview highlights — `build_highlight_job` uses exact case-sensitive `match_indices`; broken for all fuzzy queries | Fuzzy queries produce visible highlights in preview pane |
| 13 | Size filter decimal input — `parse_megabytes_input` parses `u64`, rejects `0.001` | Fractional MB values accepted |
| 14 | ScrollArea ID collision — results and preview panels share auto-generated egui ID | No red error overlay, panels scroll independently |

**Steps 10–14 = v1.1 bug fixes. Must pass before post-v1 polish.**

---

## 6. Key Design Decisions

1. **GUI-only** — no CLI; regex and fuzzy search both available in the GUI
2. **Content not stored** — tantivy indexes content for search but does not store it; re-extract on demand for previews
3. **Two search engines** — tantivy for content (inverted index, fast), nucleo for filenames (edit-distance, interactive)
4. **Two search modes** — Fuzzy (FuzzyTermQuery + nucleo) and Regex (RegexQuery) selectable in GUI
5. **Weighted merge** — combined results use 0.7 content + 0.3 filename scoring
6. **No daemon** — GUI runs its own watcher when open
7. **XDG compliance** — config and data in standard Linux paths
8. **Trait-based extraction** — new formats added by implementing `TextExtractor`
9. **Workspace crate split** — core logic reusable, GUI binary thin

---

## 7. Verification

1. `cargo build --workspace` produces one binary (sotis-gui)
2. `cargo test --workspace` passes
3. `cargo clippy --workspace -- -D warnings` clean
4. Manual test: index a folder with mixed file types, search returns correct results
5. Fuzzy search: typos in query still find relevant files
6. Regex search: patterns match expected content
7. Large folder: 10k+ files indexes in reasonable time
