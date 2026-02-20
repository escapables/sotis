# sotis

Portable offline fuzzy file search for Linux. Index folders, then search filenames and file contents with fuzzy matching and regex. Supports PDF, DOCX, EPUB, spreadsheets, and plain text.

## Features

- **Fuzzy search** — typo-tolerant search across filenames and file content
- **Regex search** — full regular expression support for precise matching
- **Multi-format** — PDF, DOCX, EPUB, XLSX/ODS/CSV, and all plain text files
- **Native GUI** — single-window desktop app (egui/eframe), no GTK/Qt dependency
- **Offline** — no network, no cloud, everything local
- **Fast** — tantivy full-text search engine, nucleo fuzzy matcher, rayon parallelism

## Quick Start

```bash
# Build
make build

# Launch
sotis-gui
```

## Data Storage

- Config: `$XDG_CONFIG_HOME/sotis/config.toml` (default: `~/.config/sotis/`)
- Index: `$XDG_DATA_HOME/sotis/index/` (default: `~/.local/share/sotis/`)

Override with `$SOTIS_CONFIG` and `$SOTIS_DATA` environment variables.

## Building

```bash
make build          # Debug build
make release        # Release build
make test           # Run tests
make check          # Full quality gate (build + test + clippy + fmt + docs)
```

## Documentation

See `docs/README.md` for the full documentation index.

## License

TBD
