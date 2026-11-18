# `splice` (`crates/ap-splice`) — Tree-sitter AST Symbol Surgeon, Zero-Escaping Atomic Patcher & Auto-Test Gate

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-splice`) & Skill (`skills/splice/`)** · 6-language Tree-sitter AST · `61 tokens` idle skill cost.

## Why You Need It

Eliminates `904` redundant same-file re-reads, `14,782` manual shell line-slices, `735` patch mismatches, and `392` UTF-8 crashes:
1. **`ap splice view` (`0.81 ms`, `-95.4%` tokens)**: Reads exact classes, structs, `impl` methods, functions, or Markdown `#` sections across **Python, TypeScript/TSX, JS, Go, Rust, and C** via real Tree-sitter AST (immune to unbalanced braces inside raw strings `r#"{"#`!).
2. **`ap splice apply --cwd . --stdin --test` (`3.1 ms`)**: Applies Zero-Escaping `@@ FILE:`, `@@ SYMBOL:`, and `@@ WRITE:` blocks (plus JSON backward compatibility), **verifies real Tree-sitter AST syntax across all 6 languages BEFORE touching disk**, preserves CRLF/LF, and runs auto-detected tests (`cargo test`, `go test`, `node --test`, `pytest`/`unittest`) with optional `--rollback-on-test-fail` in 1 single turn!

## Real Commands

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
