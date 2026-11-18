# `splice` (`crates/ap-splice`) — 6 语言 Tree-sitter 符号精读、零转义原子补丁与自动测试门禁

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立纯 Rust Crate（`crates/ap-splice`）与技能目录（`skills/splice/`）** · 6 语言真 Tree-sitter AST · 待机仅 `61 tokens`。

## 解决什么真实痛点？

根治本机审计抓到的 **904 次** 同文件重复重读、**14,782 次** 手工终端切行、**735 次** 补丁失配与 **392 次** UTF-8 编码崩溃：
1. **`ap splice view`（`0.81 ms`，直省 `95.4%` Token）**：基于真 Tree-sitter AST 跨 **Python、TypeScript/TSX、JS、Go、Rust、C** 与 Markdown 提取精确类、`impl` 方法、函数或章节（彻底免疫原始字符串 `r#"{"#` 内的括号干扰！）。
2. **`ap splice apply --cwd . --stdin --test`（`3.1 ms`）**：支持零转义纯文本事务块（`@@ FILE:` / `@@ SYMBOL:` / `@@ WRITE:`，同时兼容 JSON），**落盘前对全部 6 种语言做真 Tree-sitter 语法树校验（任一文件语法报错即全量原子回滚！）**，保留原文件 CRLF/LF，并在同一轮自动运行正确测试套件（`cargo test` / `go test` / `node --test` / `pytest` / `unittest`）。

## 常用命令

```bash
ap splice view src/lib.rs --symbol Engine.commit
ap splice apply --cwd . --stdin --test --rollback-on-test-fail <<'EOF'
@@ SYMBOL: src/lib.rs :: Engine.commit
<<<
pub fn commit(&mut self, cents: i64) -> i64 {
    self.total += cents;
    self.total
}
>>>
EOF
ap splice check . --test
```
