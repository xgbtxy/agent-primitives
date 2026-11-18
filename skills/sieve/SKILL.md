---
name: sieve
description: Compress noisy CLI/CI logs, JSON/JSONL streams, HTML tables, and CSV/TSV by 80-99% into a deterministic digest with sub-ms bundled SQLite FTS5 `search` and lossless `peek` (`--lines`, `--grep`, `--json-path`).
---

# Sieve — Lossless Output Compressor, Table Extractor & FTS5 Flight Recorder

Keep massive tool outputs out of the context window without losing a single stack frame, JSONL error event, or HTML/CSV table row. Native Rust single-binary primitive with bundled SQLite FTS5 (`0` runtime dependencies).

## Usage

```bash
# 1. Wrap any noisy command (captures stdout+stderr+exit code losslessly)
ap sieve run -- cargo test

# 2. Pipe API JSON, JSONL/NDJSON event streams, scraped HTML (extracts <table>!), or CSV/TSV
curl -s https://api.example.com/events.jsonl | ap sieve pipe --label api-events

# 3. Search across ALL archived outputs from the session in <1ms via bundled SQLite FTS5
ap sieve search "upstream_timeout"

# 4. Losslessly inspect exact slices from any archived stream (zero re-execution!)
ap sieve peek <id> --lines 120:160
ap sieve peek <id> --grep "AssertionError|panicked at" -C 6
ap sieve peek <id> --json-path "16.error"
```
