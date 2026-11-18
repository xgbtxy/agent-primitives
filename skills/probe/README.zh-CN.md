# `probe` (`crates/ap-probe`) — 6 语言 Tree-sitter 可达性打包器与运行时因果显微镜

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立纯 Rust Crate（`crates/ap-probe`）与技能目录（`skills/probe/`）** · 6 语言 Tree-sitter + Py/TS/JS/Rust 探针 · 待机仅 `59 tokens`。

## 解决什么真实痛点？

精准根治本机审计抓到的 **4,604 次** 临时手写内联探针（引发 **185 次** `ImportError` 与 **131 次** 引号崩溃）及数万 Token 文字空想内存状态：
1. **`ap probe slice . --bundle`**：基于 Tree-sitter 与导入/`#include` 可达性图，跨 **Python、TypeScript/JS、Go、Rust、C**（24 模块 TS 仓库仅需 **`7.1 ms`**！）剪除未引用死文件，并在 1 次输出中打包清单与全部活跃带行号源码。
2. **`ap probe run` 与 `ap probe diff`**：支持 Python（`sys.settrace`）、TypeScript/JS（Node 22 V8 `Proxy`）与 **Rust（原生 `rustc` 编译差分）**，自动抓出 **`[ALIAS HAZARD]`**、**`[IN-PLACE MUTATION]`**（含异常未回滚半途修改）、**`[STATE NO-OP]`**、**`-0`/`NaN`**、缓存键碰撞与负数整除舍入偏差。

## 常用命令

```bash
ap probe slice . --bundle
ap probe run --cwd . --code "import { Ledger } from './src/ledger.js'; const l = new Ledger(); l.snapshot();"
ap probe diff --lang rust --fn-a "|x: i64| (x * 250 + 5000) / 10000" --fn-b "|x: i64| ((x as f64 / 100.0) * 2.5).round() as i64"
```
