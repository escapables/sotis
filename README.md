# sotis

Portable offline fuzzy file search for Linux. Index folders, then search filenames and file contents with fuzzy matching. Supports PDF, DOCX, EPUB, spreadsheets, and plain text.

## Features

- **Fuzzy search** — typo-tolerant search across filenames and file content
- **Multi-format** — PDF, DOCX, EPUB, XLSX/ODS/CSV, and all plain text files
- **CLI + GUI** — terminal-first with a native GUI option (egui)
- **Offline** — no network, no cloud, everything local
- **Fast** — tantivy full-text search engine, nucleo fuzzy matcher, rayon parallelism

## Quick Start

```bash
# Build
make build

# Add a folder to index
sotis add ~/documents

# Build the search index
sotis index

# Search
sotis search "meeting notes"

# Filename-only search
sotis search -n "report"

# Launch GUI
sotis-gui
```

## Commands

```
sotis search <query>          Search files (content + filename)
sotis search -n <query>       Filename-only search
sotis search -c <query>       Content-only search
sotis index                   Rebuild full index
sotis index --update          Incremental update
sotis add <path>              Add folder to config
sotis remove <path>           Remove folder from config
sotis status                  Show index stats
sotis config                  Show/edit config
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
