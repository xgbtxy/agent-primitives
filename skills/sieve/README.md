# `sieve` (`crates/ap-sieve`) — Lossless Output Compressor & Bundled SQLite FTS5 Flight Recorder

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-sieve`) & Skill (`skills/sieve/`)** · Bundled static SQLite FTS5 · `59 tokens` idle skill cost.

## Why You Need It

Over `30,274` tool outputs in our local audit exceeded `8 KB`. `ap sieve` compresses `>1.2 KB` CI logs (**-98.8%**), JSON API dumps (**-99.6%**), **JSONL / NDJSON streams (-97.0%)**, **HTML pages + `<table>` pipe-tables (-98.4%)**, and **CSV/TSV tables** into high-signal digests while indexing raw bytes in `.sieve/index.sqlite` (`FTS5`) for `<6 ms` full-text `search` and lossless `peek` (`<1.2 KB` outputs pass through untouched at `0.0%`).

## Real Commands

```bash
ap sieve run -- cargo test
curl -s https://api.example.com/events.jsonl | ap sieve pipe --label events
ap sieve search "upstream_gateway_timeout"
ap sieve peek <id> --lines 120:160
ap sieve peek <id> --grep "AssertionError|panicked at" -C 5
ap sieve peek <id> --json-path "42.error"
```
