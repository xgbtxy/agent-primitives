---
name: splice
description: Read exact Tree-sitter AST symbol spans across Py/TS/JS/Go/Rust/C or Markdown (`view`), apply zero-escaping atomic multi-file edits + auto-tests (`apply --stdin --test`), and run sub-25ms AST checks (`check --test`).
---

# Splice — Tree-sitter AST Symbol Surgeon, Zero-Escaping Atomic Patcher & Auto-Test Gate

Eliminates redundant full-file re-reads, JSON-escaping boilerplate, CRLF/whitespace patch mismatches, silent 0-op bugs, broken syntax writes across Py/TS/JS/Go/Rust/C, and separate test-verification turns.

## Cross-Shell Safe Usage (`PowerShell` & `Bash`)

```bash
# 1. Read exact classes, structs, impl methods, functions, or Markdown sections across 1+ files (Py, TS/JS, Go, Rust, C, MD)
ap splice view src/ledger.ts src/rates.ts --symbol Ledger.post --symbol splitFee

# 2a. Zero-Escaping Atomic Patch + Write Test + Run Test Suite in ONE Turn (PowerShell `@' ... '@ |`):
@'
@@ FILE: src/rates.ts
<<<
  return Math.round(amount * rate);
===
  const r = Math.round(amount * rate);
  return Object.is(r, -0) ? 0 : r;
>>>
@@ SYMBOL: src/lib.rs :: Engine.commit
<<<
pub fn commit(&mut self, cents: i64) -> i64 {
    self.total += cents;
    self.total
}
>>>
@@ WRITE: tests/repro.test.ts
<<<
import test from "node:test";
import assert from "node:assert/strict";
>>>
'@ | ap splice apply --cwd . --stdin --test --rollback-on-test-fail

# 2b. POSIX bash/zsh equivalent (JSON `[...]` specs also remain 100% supported):
ap splice apply --cwd . --stdin --test <<'EOF'
@@ FILE: src/b.rs
<<<
old_line
===
new_line
>>>
EOF

# 3. Standalone sub-25ms UTF-8 + 6-language Tree-sitter AST + local-import + auto-test gate
ap splice check . --test

# 4. Tree-sitter AST-exact workspace symbol rename (ignores comments, strings & substrings; supports --dry-run & --rollback-on-test-fail)
ap splice rename oldSymbol newSymbol . --dry-run
ap splice rename oldSymbol newSymbol . --test --rollback-on-test-fail
```
