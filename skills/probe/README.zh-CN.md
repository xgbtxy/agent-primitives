# `probe` (`crates/ap-probe`) — 活跃代码打包切片、真服务/浏览器冒烟与运行时显微镜

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立物理隔离 Rust Crate（`crates/ap-probe`）与技能包（`skills/probe/`）** · 6 语言 Tree-sitter + 原生防挂起 HTTP/无头浏览器冒烟 + Py/Node-V8/`rustc` 探针 · 静默成本 `<65 tokens`。

## 解决什么痛点

1. **一次调用剪除历史废文件（`probe slice --bundle`）**：`6.3 ms` 追踪 Python、TS/JS、Go、Rust、C 真实导入可达性，直接剥离未引用的旧屎山模块，避免模型误读照抄。
2. **砍掉 Windows 上高达 70% 的手工验证 Token 税（`probe smoke`）**：自动分配空闲 `PORT`、将服务端日志落盘防 Windows 管道死锁、调用本机无头 **Edge / Chrome** 渲染真实动态页面 DOM（`BROWSER /`）、原生处理中文 UTF-8 查询与 JSON/CSV 断言、扫描服务端 `stderr` 报错栈，并在结束时通过 `taskkill /T /F` 100% 清理整棵进程树——**把 20 轮交互式验证压缩为 1 次 `<3 秒` 调用**。
3. **运行时与差分显微镜（`probe run` / `probe diff`）**：横跨 Python、Node 22 V8 `Proxy` 与原生 `rustc` 自动抓出浅拷贝别名污染、原地修改与 `-0.0` 舍入边界。

## 常用命令

```bash
ap probe slice . --bundle
ap probe smoke --cwd . --stdin
ap probe run --cwd . --code "import { Ledger } from './src/ledger.js'; new Ledger().snapshot();"
ap probe diff --lang rust --fn-a "|x: i64| (x * 250 + 5000) / 10000" --fn-b "|x: i64| ((x as f64 / 100.0) * 2.5).round() as i64"
```
