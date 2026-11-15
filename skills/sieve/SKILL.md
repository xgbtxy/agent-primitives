---
name: sieve
description: Compress noisy command outputs, test/CI logs, large JSON API responses, and scraped HTML by 80-99% into a deterministic digest while archiving raw output locally for lossless `peek` retrieval by line range, regex, or JSON path. Use before inspecting long command stdout/stderr, logs, or web/API payloads.
---

# Sieve — Lossless Output Compressor & Flight Recorder

Keep massive tool outputs out of the context window without losing a single stack frame or record. Zero dependencies (Python 3 stdlib only). Works identically for **Coding Agents** (pytest/cargo/CI/git diffs) and **Personal Assistant Agents** (JSON APIs, scraped HTML, CSVs, system/email logs).

## Usage

```bash
# 1. Wrap any noisy command (captures stdout+stderr+exit code losslessly)
python scripts/sieve.py run -- pytest -q

# 2. Pipe long API JSON, scraped HTML, or log files
curl -s https://api.example.com/items | python scripts/sieve.py pipe --label api-items

# 3. Losslessly inspect exact slices from the archived raw stream (zero re-execution!)
python scripts/sieve.py peek <id> --lines 120:160
python scripts/sieve.py peek <id> --grep "AssertionError|Traceback" -C 6
python scripts/sieve.py peek <id> --json-path "items.0"
```

## Operating Rule

- Never dump raw `>4 KB` logs, HTML, or JSON arrays directly into context when `sieve.py` is available.
- Read the `<1.5 KB` digest first; use `peek <id>` with exact `--lines`, `--grep`, or `--json-path` only for the slice you actually need.
