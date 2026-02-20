---
summary: 'Full architecture spec for SOTIS — fuzzy file search for Linux.'
read_when:
  - Starting implementation of a new component.
  - Reviewing architecture decisions.
  - Checking how components interact.
---

# SOTIS — Architecture Plan

Portable, offline, native Linux app for fuzzy file search. Users pick folders to index, then search filenames and file contents with fuzzy matching. Supports PDF, DOCX, EPUB, spreadsheets, and plain text. Ships as CLI + GUI binaries.

---

## 1. Core Architecture

### Workspace Layout

```
crates/
├── sotis-core/     — index, search, scanner, watcher, text extraction
├── sotis-cli/      — CLI binary (clap)
└── sotis-gui/      — GUI binary (egui/eframe)
```

### Search Strategy

- **tantivy** for fuzzy content search (FuzzyTermQuery, Levenshtein distance)
- **nucleo-matcher** for fuzzy filename matching (Smith-Waterman algorithm)
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
| `search.rs` | Query building, fuzzy search, result ranking and merging |
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

## 3. Crate: sotis-cli

Single binary with clap. Commands:

```
sotis search <query>          # Search files (content + filename)
sotis search -n <query>       # Filename-only search
sotis search -c <query>       # Content-only search
sotis index                   # Rebuild full index
sotis index --update          # Incremental update
sotis add <path>              # Add folder to config
sotis remove <path>           # Remove folder from config
sotis status                  # Show index stats
sotis config                  # Show/edit config
```

Output: ranked results with path, match score, snippet (for content matches).

---

## 4. Crate: sotis-gui

egui/eframe application. Single window:

- **Search bar** at top — type to search, results update live
- **Filter chips** — filename only, content only, file type filter
- **Results list** — path, score, snippet preview
- **Folder management** — add/remove indexed folders
- **Status bar** — index stats, last update time

No GTK/Qt dependency — pure OpenGL via eframe's glow backend.

---

## 5. Build Targets

| Target | Command | Notes |
|--------|---------|-------|
| Debug | `cargo build --workspace` | Both CLI and GUI |
| Release | `cargo build --release --workspace` | Optimized |
| CLI only | `cargo build -p sotis-cli` | Static with musl possible |
| GUI only | `cargo build -p sotis-gui` | Needs OpenGL |

---

## 6. Implementation Sequence

| Step | What | Result |
|------|------|--------|
| 1 | Project skeleton, Cargo workspace, all crates compile | Empty binaries that run |
| 2 | `config.rs` — XDG paths, TOML load/save, folder config | Configuration works |
| 3 | `error.rs` + `extract/` — text extractors for all formats | Can extract text from files |
| 4 | `scanner.rs` — directory walking, file discovery | Can find files in configured folders |
| 5 | `index.rs` — tantivy index create/add/remove | Can build search index |
| 6 | `search.rs` — fuzzy search with tantivy + nucleo | Can search index |
| 7 | CLI commands — search, index, add, remove, status | Usable CLI tool |
| 8 | `watcher.rs` — file system watching, incremental update | Auto-reindex on changes |
| 9 | GUI — basic search window with eframe | Usable GUI |
| 10 | GUI — folder management, filters, polish | Complete GUI |

**Steps 1–7 = usable CLI tool. Steps 8–10 = complete v1.**

---

## 7. Key Design Decisions

1. **Content not stored** — tantivy indexes content for search but does not store it; re-extract on demand for previews
2. **Two search engines** — tantivy for content (inverted index, fast), nucleo for filenames (edit-distance, interactive)
3. **Weighted merge** — combined results use 0.7 content + 0.3 filename scoring
4. **No daemon** — CLI is stateless; GUI runs its own watcher when open
5. **XDG compliance** — config and data in standard Linux paths
6. **Trait-based extraction** — new formats added by implementing `TextExtractor`
7. **Workspace crate split** — core logic reusable, UI binaries thin

---

## 8. Verification

1. `cargo build --workspace` produces two binaries
2. `cargo test --workspace` passes
3. `cargo clippy --workspace -- -D warnings` clean
4. Manual test: index a folder with mixed file types, search returns correct results
5. Fuzzy search: typos in query still find relevant files
6. Large folder: 10k+ files indexes in reasonable time
