---
name: splice
description: Read exact AST-bounded functions/methods or Markdown sections without full-file re-reads (`view`), apply UTF-8/CRLF-safe syntax-verified atomic multi-file edits with automatic rollback (`apply`), and run sub-50ms repo integrity checks (`check`).
---

# Splice — AST Symbol Surgeon & Atomic Multi-File Patcher

Eliminates 50% redundant same-file re-reads, blind shell line-slicing, CRLF/whitespace patch mismatches, silent `str.replace()` 0-op bugs, and Windows `cp936`/`gbk` UTF-8 crashes.

## Usage

```bash
# 1. Read ONLY an exact class, method, or Markdown heading with line numbers (~100 tokens)
python scripts/splice.py view pkg/engine.py --symbol PipelineEngine.process
python scripts/splice.py view PLAYBOOK.md --heading "Quick Start"

# 2. Atomically apply multi-file edits (UTF-8 enforced, CRLF preserved, syntax-verified + rollback, purges .pyc)
python scripts/splice.py apply --cwd . --stdin << 'EOF'
[
  {"path": "pkg/a.py", "op": "replace_symbol", "symbol": "Envelope.clone", "new": "def clone(self):\n    return deepcopy(self)"},
  {"path": "pkg/b.py", "op": "replace", "old": "delay = int(h)", "new": "delay = int(h) * 1000"}
]
EOF

# 3. Verify UTF-8, AST syntax, conflict markers & local imports across repo in <50ms
python scripts/splice.py check .
```
