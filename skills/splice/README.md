# `splice` — AST Symbol Surgeon & Atomic Multi-File Patcher

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Mini-Project (`skills/splice/`)** · Zero third-party dependencies · `~68 tokens` idle skill cost.

## Why You Need It (Mined Across 185 Real Sessions in Codex, Antigravity & Grok)

Auditing real multi-file coding sessions across **OpenAI Codex CLI**, **Google Antigravity CLI**, and **xAI Grok CLI** revealed massive universal read/edit friction:

- **50.1% Redundant Same-File Re-Reads** (`660 / 1,318` `read_file` calls in Antigravity) and **4,835 Manual Shell Line-Slice Calls** (`Get-Content | Select-Object -Skip -First` in Codex) just to inspect a single 20-line method or re-check shifted line numbers after an edit.
- **231 Patch / Target-Mismatch Failures** (`apply_patch` / `edit_file` / `search_replace` failing on CRLF vs LF, trailing spaces, or stale line context) + **silent `str.replace()` 0-op bugs** when agents fall back to inline Python edit scripts.
- **155 Windows `cp936`/`gbk` Encoding Crashes** when inline edit scripts call `Path.read_text()` / `write_text()` without `encoding="utf-8"` on files containing Unicode (`—`, `→`, CJK), plus stale `__pycache__/*.pyc` bytecode traps.

`splice` gives both **Coding Agents** and **Personal Assistant Agents** deterministic surgical read/write primitives:

- **`splice view`**: Extracts only the exact AST-bounded `Class`, `Class.method`, `function`, or Markdown `## Heading` section with line numbers (`~100 tokens` vs `4,000+` tokens).
- **`splice apply`**: Applies multi-file JSON edit transactions (`replace`, `replace_symbol`, `write`) atomically—enforcing UTF-8, tolerating & preserving CRLF/LF and trailing whitespace, rejecting 0-match no-ops loudly, verifying `ast.parse` / `json.loads` **before** committing (rolling back all files on any syntax error), and purging stale `.pyc` caches.
- **`splice check`**: Sub-50ms repository UTF-8, AST syntax, merge-conflict, and intra-repo import verifier.

## Copy Just This Folder

```bash
cp -r skills/splice ~/.agents/skills/
```

## Real Commands

```bash
# View exact method or Markdown section
python scripts/splice.py view src/engine.py --symbol Engine.process
python scripts/splice.py view NOTES.md --heading "Action Items"

# Apply multi-file atomic transaction from stdin or file
python scripts/splice.py apply --cwd . --spec edits.json

# Sub-50ms syntax + UTF-8 + local-import gate
python scripts/splice.py check .
```
