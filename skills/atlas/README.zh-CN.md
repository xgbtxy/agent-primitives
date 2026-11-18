# `atlas` (`crates/ap-atlas`) — `<50ms` 6 语言 Tree-sitter 结构地图

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立纯 Rust Crate（`crates/ap-atlas`）与技能目录（`skills/atlas/`）** · 内置 6 语言 Tree-sitter + `ignore` 引擎 · 待机仅 `58 tokens`。

## 解决什么真实痛点？

Agent 冷启动探索陌生代码仓或数百篇个人笔记库时，常盲跑几十轮搜索与列目录（本机审计抓到 **8,952 次**宽泛搜索）。`ap atlas` 内嵌 **Python、TypeScript/TSX、JavaScript、Go、Rust、C** 六套官方 Tree-sitter C 语法树与 `ripgrep` 同款 `.gitignore` 遍历器：**`16.3 ms`** 扫完 64 模块仓库（定向 Token 直降 **99.0%**），按被引用中心度与 `[dirty]` Git 热点排序，并标出 `>90 天` 陈旧 TODO 诱饵。

## 常用命令

```bash
ap atlas map .
ap atlas map . --focus "exporter" --max-files 20
ap atlas map . --json
```
