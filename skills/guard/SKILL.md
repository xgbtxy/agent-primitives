---
name: guard
description: Run 1-call fused 6-gate deep verification (`verify --deep`: OWASP `sec`, contracts, anti-slop, non-vacuous tests, API fuzz, UI & AST mutation), `rule`, `stop-gate`, and `shield`.
---

# Guard — 6-in-1 Deep Verification Gate, OWASP Scanner & Anti-Slop Shield

One command replaces manual multi-turn test/browser/security scripts with deterministic **`exit 2`** gates and a full verified proof receipt.

## Usage

```bash
# 1. Recommended before delivery: 6-in-1 Deep Gate (OWASP Sec + Contracts + Anti-Slop + Non-Vacuous Tests + API Chaos Fuzz + Desktop/Mobile UI + AST Mutation)
ap guard verify --cwd . --deep

# 2. Standalone <5ms OWASP Security Audit (SQLi, Path Traversal, Cmd Injection, DOM XSS, TLS/Debug)
ap guard sec .

# 3. Custom deterministic rules, pre-exec blast shield & 3x loop breaker
ap guard rule add --id no-legacy --glob "src/*.ts" --forbid "old_dump" --msg "Use new exporter"
ap guard shield --text "git push --force origin main"
ap guard run --max-repeats 3 -- npm test
```
