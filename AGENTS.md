# `agent-primitives` Repository Operating Rules

Before proposing, adding, or modifying anything in this repository, read **[HANDBOOK.md](./HANDBOOK.md)** / **[HANDBOOK.zh-CN.md](./HANDBOOK.zh-CN.md)**.

## Six Non-Negotiable Invariants
1. **Physical Boundaries Only (Zero Prompt Crutches)**: Every primitive must execute deterministic OS/AST/Heap/Byte/SQLite computation across one of the 4 irreducible physical boundaries—never pure-Markdown roleplay or prompt checklists.
2. **One Folder = Complete Standalone Mini-Project**: Every `skills/<name>/` folder must run in isolation with **zero cross-folder imports** when copied alone (`test_standalone_no_cross_imports`), while `ap.py` provides optional 1-command onboarding (`python ap.py init` / `python ap.py init --here`).
3. **Zero Third-Party Dependencies**: Python 3.9+ standard library only (`0` pip packages, `0` background daemons, `0` cloud uploads/keys) across Linux, macOS, and Windows (`pwsh` + `cp936` safe).
4. **Hard Context & Latency Budgets**: `SKILL.md` YAML frontmatter **`< 85 tokens`** per folder; core CLI latency **`< 100 ms`**; bounded stdout with lossless on-demand `peek` and call-site loop deduplication.
5. **Deepen Before Multiplying**: Prefer extending the existing 7 orthogonal primitives (`atlas`, `recall`, `probe`, `splice`, `sieve`, `guard`, `baton`) over creating new folders unless a new physical boundary is empirically proven.
6. **Empirical Truth & Bilingual Parity**: Every iteration must follow the 5-step loop in `HANDBOOK.md` (`audit_sessions.py` -> 4-gate check -> zero-dep code + isolation tests -> live CLI A/B + negative controls -> 1:1 `*.md` & `*.zh-CN.md` sync + Evolution Log entry).

## Required Verification Before Every Commit
```bash
python ap.py doctor
python -m unittest discover -s tests -v
python bench/run_bench.py
```
