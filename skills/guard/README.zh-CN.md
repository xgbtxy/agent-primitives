# `guard` (`crates/ap-guard`) — `<2ms` 原生 `exit 2` 真值门禁、死循环熔断与护盾

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立纯 Rust Crate（`crates/ap-guard`）与技能目录（`skills/guard/`）** · `< 2 ms` 原生启动 · 待机仅 `57 tokens`。

## 解决什么真实痛点？

1. **虚报完工（`exit 1` 不拦截陷阱）**：在 Claude Code、Codex 等宿主中，`exit 1` 仅被视为非阻塞警告；唯有 **`exit 2` + `stderr` 诊断** 才能物理拦住未过检的“已完成”宣告。
2. **同错死循环重试**：本机审计抓到 **237 次** 完全相同的报错命令连刷。
3. **密钥泄露与高危操作**：拦截 `sk-...` / `ghp_...` / `AKIA...` 私钥外泄及 `git push --force`、`rm -rf /`、`DROP TABLE` 等高爆炸半径动作。

采用纯 Rust 编译为原生机器码，每次 `PreToolUse` Hook 启动仅需 **`< 2 ms`**（比 Python 脚本快 25 倍以上），并原生双协议兼容 **Claude Code / Codex**（`exit 2`）与 **Google Antigravity**（camelCase `deny`/`allow`/`continue`/`stop` + `continuationActive` 防死循环）。

## 自动挂载 Hook

```bash
ap init --with-hooks
```

## 常用命令

```bash
ap guard shield --text "git push --force origin main"
ap guard run --max-repeats 3 -- cargo test
ap guard stop-gate --check "cargo test --quiet"
```
