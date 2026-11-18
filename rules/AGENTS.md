# Agent Primitives Runtime Routing (`ap` v1.2)

1. **Orient (`atlas` / `probe` / `ap_orient`)**: Empty/new dir -> `ap atlas map .` (host runtimes). Existing repo -> `ap probe slice . --bundle` + `ap atlas product .` (Personalized PageRank, UI/API/DOM `#id` contracts & `[SLOP RADAR]`). Skip unreferenced dead files.
2. **Edit (`splice` / `ap_edit`)**: Inspect symbols via `ap splice view <file> --symbol <Sym>`. Batch writes/edits + tests in 1 call via `ap splice apply --cwd . --stdin --test` (`@@ WRITE: p` `<<<`...`>>>`, `@@ FILE: p` `<<<` old `===` new `>>>`, `@@ SYMBOL: p :: S` `<<<`...`>>>`).
3. **Verify (`probe smoke` + `guard verify` / `ap_smoke` + `ap_verify`)**: Test web/API apps in 1 call via `ap probe smoke --cwd . --stdin` (`OPEN /`, `FILL #id v`, `SELECT #id v`, `CLICK #id`|`text=Btn`, `EVAL <js>`, `GET/POST /api/...`, `EXPECT STATUS/CONTAINS/JSON` — auto-binds `PORT`, checks 0 JS/stderr errors & kills process tree). Finish with `ap guard verify`.
4. **Diagnose & Persist**: `ap probe run --cwd . --stdin` / `probe diff`; `ap sieve run -- <cmd>`; `ap guard rule add`; `ap recall`; `ap baton`.
