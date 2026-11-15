# `splice` — Polyglot Symbol Surgeon, Atomic Patcher & Auto-Test Gate

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Mini-Project (`skills/splice/`)** · Zero third-party dependencies · `~64 tokens` idle skill cost.

## Why You Need It (Mined Across 69,963 Real Tool Calls in Codex, Antigravity, Grok & Claude Code)

Auditing real multi-file coding sessions across four agent harnesses revealed massive universal read/edit/test friction:

- **50.1% Redundant Same-File Re-Reads** and **4,835 Manual Shell Line-Slice Calls** just to inspect a single method or re-check shifted line numbers after an edit.
- **244 Patch / Target-Mismatch Failures** (`apply_patch` / `edit_file` / `search_replace` failing on CRLF vs LF, trailing spaces, or stale line context) + **101+ Windows `cp936`/`gbk` UTF-8 Crashes** + stale `__pycache__/*.pyc` bytecode traps.
- **Wasted Test-Discovery Turns**: Agents repeatedly guess `python -m pytest` (`No module named pytest`) on `unittest` repos, miss Node 22 `--experimental-strip-types --test` flags on TypeScript repos, or burn extra turns deleting temporary JSON edit files.

`splice` gives both **Coding Agents** and **Personal Assistant Agents** deterministic surgical read/write/verify primitives:

- **`splice view`**: Extracts exact symbol spans (`Class`, `Class.method`, `function`, `const fn =>`, `Recv.Method`, `impl` methods) across **Python, TypeScript/JavaScript, Go, and Rust**, line ranges (`--lines`), or Markdown `## Heading` sections across **1 or multiple files** in `< 5 ms`.
- **`splice apply`**: Applies multi-file JSON edit transactions (`replace`, `replace_symbol` across Py/TS/JS/Go/Rust, `write`) atomically via `--stdin`, `--b64`, or `--spec ... --unlink-spec`—enforcing UTF-8, preserving CRLF/LF, verifying syntax before commit (rolling back all files on error), and purging stale `.pyc` caches.
- **`splice check [--test]`**: Sub-50ms repository UTF-8, syntax, merge-conflict, and Python + TypeScript/JS relative-import verifier—plus **`--test`** to auto-detect and execute the repo's real test runner (`pytest`, `unittest discover`, `node --experimental-strip-types --test`, `node --test`, `go test`, or `cargo test`) with bounded failure-preserving output.

## 1-Command Onboarding (or Copy Just This Folder)

```bash
# Recommended: 1-command repo setup
python ap.py init --here

# Or copy just this single standalone folder:
cp -r skills/splice ~/.agents/skills/
```

## Real Commands (PowerShell & Bash Safe)

```bash
# View exact methods across 1+ files (Py, TS/JS, Go, Rust) or Markdown headings
python .agents/ap.py splice view src/ledger.ts src/money.ts --symbol LedgerEngine.postBatch --symbol computePlatformFee
python .agents/ap.py splice view NOTES.md --heading "Action Items"

# Apply multi-file atomic transaction (PowerShell `@'...'@ |`, Bash `<<'EOF'`, or `--spec edits.json --unlink-spec`)
python .agents/ap.py splice apply --cwd . --spec _edits.json --unlink-spec

# Sub-50ms static check + auto-detected test suite in 1 call
python .agents/ap.py splice check . --test
```
