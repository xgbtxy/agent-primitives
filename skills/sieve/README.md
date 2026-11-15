# `sieve` — Lossless Output Compressor & Flight Recorder

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Mini-Project (`skills/sieve/`)** · Zero third-party dependencies · `83 tokens` idle skill cost.

## Why You Need It

A single `pytest`, `cargo test`, CI log, large JSON API response, or scraped webpage (`50–300 KB`) floods the context window, triggering **Context Rot** and burning $0.50–$3.00 of input tokens in subsequent turns. Existing CLI proxies (`rtk`, etc.) use **lossy truncation**—when a crucial stack frame or JSON field gets stripped, the agent must re-run the entire slow command.

`sieve` archives the **100% lossless raw stream** in a bounded local ring buffer (`.sieve/<id>.raw`, auto-pruned at 50 MB) while returning a `< 1.5 KB` deterministic high-signal digest:
- **Test / Build / System Logs**: Collapses repetitive noise (`[x142] ...`), extracts deduplicated exception/traceback windows with exact `L<start>-L<end>` line numbers, and keeps head/tail anchors.
- **JSON Payloads**: Extracts top-level keys, array lengths, item schemas, and `error`/`warning`/`status` fields (-99.6% on 400-record API dumps).
- **HTML Pages**: Strips `<script>`, `<style>`, and `<svg>` bloat, preserving `<title>`, `H1–H3` outline, link count, and lead text (-98.4%).
- **Short Outputs (`< 1.2 KB`)**: Passes through untouched (`0%` overhead—never summarizes a 5-line `git status`).

## Copy Just This Folder

```bash
cp -r skills/sieve ~/.agents/skills/
# or: npx skills add xgbtxy/agent-primitives --skill sieve
```

## Real Commands

```bash
# Coding Agent: run noisy test suite or build
python scripts/sieve.py run -- pytest -q

# Personal Assistant Agent: compress large API JSON or scraped HTML
curl -s https://api.example.com/events | python scripts/sieve.py pipe --label events

# Lossless instant retrieval (zero re-execution!)
python scripts/sieve.py peek <id> --lines 910:930
python scripts/sieve.py peek <id> --grep "AssertionError|KeyError" -C 6
python scripts/sieve.py peek <id> --json-path "items.0"
```
