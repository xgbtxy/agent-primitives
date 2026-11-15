---
name: splice
description: Read exact symbol spans across Py/TS/JS/Go/Rust or Markdown headings (`view`), apply UTF-8/CRLF-safe syntax-verified atomic multi-file edits with rollback (`apply`), and run sub-50ms integrity + auto-test checks (`check --test`).
---

# Splice — Polyglot Symbol Surgeon, Atomic Patcher & Auto-Test Gate

Eliminates redundant full-file re-reads, CRLF/whitespace patch mismatches, silent `str.replace()` 0-op bugs, Windows `cp936` UTF-8 crashes, and wasted test-runner guessing turns.

## Cross-Shell Safe Usage (`PowerShell` & `Bash`)

```bash
# 1. Read exact classes, methods, functions, or Markdown sections across 1+ files (Py, TS/JS, Go, Rust, MD)
python .agents/ap.py splice view src/ledger.ts src/rates.ts --symbol Ledger.post --symbol splitFee
python .agents/ap.py splice view PLAYBOOK.md --heading "Quick Start"

# 2a. Atomically apply multi-file edits on Windows PowerShell (`@' ... '@ |` — NEVER use `<<` in PowerShell!):
@'
[
  {"path": "src/a.py", "op": "replace_symbol", "symbol": "Envelope.clone", "new": "def clone(self):\n    return deepcopy(self)"},
  {"path": "src/b.ts", "op": "replace", "old": "Math.round(a * r)", "new": "Math.trunc(a * r)"}
]
'@ | python .agents/ap.py splice apply --cwd . --stdin

# 2b. Atomically apply multi-file edits on POSIX bash/zsh (or pass `--spec _edits.json --unlink-spec` on any OS):
python .agents/ap.py splice apply --cwd . --stdin <<'EOF'
[{"path": "src/b.ts", "op": "replace", "old": "old_line", "new": "new_line"}]
EOF

# 3. Verify UTF-8, syntax, local Py/TS/JS imports AND auto-run repo tests (pytest/unittest/node --test/go/cargo) in 1 call
python .agents/ap.py splice check . --test
```
