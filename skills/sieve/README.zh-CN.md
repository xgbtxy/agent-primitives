# `sieve` (`crates/ap-sieve`) — 无损输出压缩器、表格提取与内置 SQLite FTS5 黑匣子

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立纯 Rust Crate（`crates/ap-sieve`）与技能目录（`skills/sieve/`）** · 内置静态 SQLite FTS5 · 待机仅 `59 tokens`。

## 解决什么真实痛点？

本机审计发现 **30,274 次** 工具输出超过 `8 KB`。`ap sieve` 将 `>1.2 KB` 的 CI 日志（**-98.8%**）、JSON 接口（**-99.6%**）、**JSONL / NDJSON 事件流（-97.0%，自动提取异常行）**、**复杂 HTML 网页（含 `<table>` 管道表，-98.4%）** 与 **CSV/TSV 数据表** 压缩为高信噪比摘要，同时将原文索引至内置静态 SQLite `FTS5`，支持 `<6 ms` 全文检索 `search` 与按需无损切片 `peek`（`<1.2 KB` 短输出诚实原样直通）。

## 常用命令

```bash
ap sieve run -- cargo test
curl -s https://api.example.com/events.jsonl | ap sieve pipe --label events
ap sieve search "upstream_gateway_timeout"
ap sieve peek <id> --lines 120:160
ap sieve peek <id> --grep "AssertionError|panicked at" -C 5
ap sieve peek <id> --json-path "42.error"
```
