use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

fn ap_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ap"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_ap(
    cwd: &Path,
    args: &[&str],
    stdin_data: Option<&str>,
    extra_env: &[(&str, &Path)],
) -> Output {
    let mut cmd = Command::new(ap_bin());
    cmd.args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    if stdin_data.is_some() {
        cmd.stdin(Stdio::piped());
    }
    let mut child = cmd.spawn().expect("failed to spawn ap binary");
    if let Some(input) = stdin_data {
        if let Some(mut sin) = child.stdin.take() {
            sin.write_all(input.as_bytes())
                .expect("failed to write stdin");
        }
    }
    child
        .wait_with_output()
        .expect("failed to wait on ap binary")
}

#[test]
fn test_crate_physical_isolation_and_frontmatter_budget() {
    let root = repo_root();
    let crates_dir = root.join("crates");
    let prims = [
        "sieve", "baton", "recall", "guard", "atlas", "probe", "splice",
    ];

    for name in prims {
        let cargo_toml =
            fs::read_to_string(crates_dir.join(format!("ap-{name}")).join("Cargo.toml")).unwrap();
        for other in prims {
            if other != name {
                assert!(
                    !cargo_toml.contains(&format!("ap-{other}")),
                    "ap-{name}/Cargo.toml must NOT depend on sibling crate ap-{other}"
                );
            }
        }
        let skill_md = fs::read_to_string(root.join("skills").join(name).join("SKILL.md")).unwrap();
        let parts: Vec<&str> = skill_md.split("---").collect();
        assert!(
            parts.len() >= 3,
            "SKILL.md for {name} missing YAML frontmatter"
        );
        let fm_tokens = parts[1].trim().len().div_ceil(4);
        assert!(
            fm_tokens < 85,
            "SKILL.md frontmatter for {name} is {fm_tokens} tokens (exceeds <85 constitutional budget)"
        );
    }
}

#[test]
fn test_sieve_log_json_jsonl_html_csv_fts5_and_negative_control() {
    let tmp = TempDir::new().unwrap();
    let sieve_home = tmp.path().join("sieve_home");

    // 1. Honest Negative Control (<1.2 KB passthrough)
    let out_short = run_ap(
        tmp.path(),
        &["sieve", "pipe", "--label", "git-status", "--json"],
        Some("On branch main\nnothing to commit\n"),
        &[("SIEVE_HOME", &sieve_home)],
    );
    assert!(out_short.status.success());
    let j_short: Value = serde_json::from_slice(&out_short.stdout).unwrap();
    assert_eq!(j_short["mode"], "passthrough");

    // 2. JSONL stream + bundled SQLite FTS5 search + peek --json-path
    let mut jl = String::new();
    for i in 0..80 {
        if i == 42 {
            jl.push_str(
                "{\"seq\":42,\"type\":\"rpc\",\"status\":503,\"error\":\"shard_quorum_lost\"}\n",
            );
        } else {
            jl.push_str(&format!(
                "{{\"seq\":{i},\"type\":\"ping\",\"status\":200,\"host\":\"node-east\"}}\n"
            ));
        }
    }
    let out_jl = run_ap(
        tmp.path(),
        &["sieve", "pipe", "--label", "rpc-stream", "--json"],
        Some(&jl),
        &[("SIEVE_HOME", &sieve_home)],
    );
    assert!(out_jl.status.success());
    let j_jl: Value = serde_json::from_slice(&out_jl.stdout).unwrap();
    assert_eq!(j_jl["mode"], "jsonl");
    assert!(j_jl["rendered"]
        .as_str()
        .unwrap()
        .contains("shard_quorum_lost"));
    let jl_id = j_jl["id"].as_str().unwrap().to_string();

    let out_peek = run_ap(
        tmp.path(),
        &["sieve", "peek", &jl_id, "--json-path", "42.error"],
        None,
        &[("SIEVE_HOME", &sieve_home)],
    );
    assert!(out_peek.status.success());
    assert!(String::from_utf8_lossy(&out_peek.stdout).contains("shard_quorum_lost"));

    let out_fts = run_ap(
        tmp.path(),
        &["sieve", "search", "shard_quorum_lost", "--json"],
        None,
        &[("SIEVE_HOME", &sieve_home)],
    );
    assert!(out_fts.status.success());
    let j_fts: Value = serde_json::from_slice(&out_fts.stdout).unwrap();
    assert!(!j_fts["hits"].as_array().unwrap().is_empty());

    // 3. CSV table compression
    let mut csv_s = String::from("id,service,latency_ms,status\n");
    for i in 0..80 {
        csv_s.push_str(&format!("req_{i},billing-core,{},ok\n", 10 + i));
    }
    let out_csv = run_ap(
        tmp.path(),
        &["sieve", "pipe", "--label", "metrics-csv", "--json"],
        Some(&csv_s),
        &[("SIEVE_HOME", &sieve_home)],
    );
    let j_csv: Value = serde_json::from_slice(&out_csv.stdout).unwrap();
    assert_eq!(j_csv["mode"], "csv");
    assert!(j_csv["rendered"]
        .as_str()
        .unwrap()
        .contains("| id | service |"));
}

#[test]
fn test_baton_and_recall_hybrid_cjk_fts5() {
    let tmp = TempDir::new().unwrap();
    let baton_home = tmp.path().join("baton_home");
    let recall_home = tmp.path().join("recall_home");

    assert!(run_ap(
        tmp.path(),
        &[
            "baton",
            "set",
            "--goal",
            "Migrate core to Rust",
            "--constraint",
            "Zero Python runtime",
            "--next",
            "Run cargo test"
        ],
        None,
        &[("BATON_HOME", &baton_home)]
    )
    .status
    .success());
    assert!(run_ap(
        tmp.path(),
        &[
            "baton",
            "fact",
            "Tree-sitter 0.27 supports all 6 grammars",
            "--ref",
            "Cargo.toml:25"
        ],
        None,
        &[("BATON_HOME", &baton_home)]
    )
    .status
    .success());
    assert!(run_ap(
        tmp.path(),
        &[
            "baton",
            "deadend",
            "Regex brace matching failed on raw strings",
            "--reason",
            "Unbalanced { inside r#\"{\""
        ],
        None,
        &[("BATON_HOME", &baton_home)]
    )
    .status
    .success());

    let res_out = run_ap(
        tmp.path(),
        &["baton", "resume"],
        None,
        &[("BATON_HOME", &baton_home)],
    );
    let resume_md = String::from_utf8_lossy(&res_out.stdout);
    assert!(resume_md.contains("Migrate core to Rust"));
    assert!(resume_md.contains("DO NOT RETRY"));
    assert!(resume_md.len().div_ceil(4) < 500);

    // Recall: English glob + Chinese CJK bigram recall
    assert!(run_ap(
        tmp.path(),
        &[
            "recall",
            "add",
            "--triggers",
            "crates/ap-splice/*,crlf",
            "--rule",
            "Preserve original CRLF line endings on atomic apply.",
            "--kind",
            "gotcha"
        ],
        None,
        &[("RECALL_HOME", &recall_home)]
    )
    .status
    .success());
    assert!(run_ap(
        tmp.path(),
        &[
            "recall",
            "add",
            "--triggers",
            "报销,发票,财务",
            "--rule",
            "差旅发票抬头统一填写独立研究工作室并归档 PDF。",
            "--kind",
            "workflow"
        ],
        None,
        &[("RECALL_HOME", &recall_home)]
    )
    .status
    .success());

    let m_en = run_ap(
        tmp.path(),
        &[
            "recall",
            "match",
            "--query",
            "fix line endings",
            "--files",
            "crates/ap-splice/src/lib.rs",
            "--json",
        ],
        None,
        &[("RECALL_HOME", &recall_home)],
    );
    let j_en: Value = serde_json::from_slice(&m_en.stdout).unwrap();
    assert!(j_en["matches"][0]["rule"]
        .as_str()
        .unwrap()
        .contains("CRLF"));

    let m_zh = run_ap(
        tmp.path(),
        &[
            "recall",
            "match",
            "--query",
            "整理本月差旅报销发票",
            "--json",
        ],
        None,
        &[("RECALL_HOME", &recall_home)],
    );
    let j_zh: Value = serde_json::from_slice(&m_zh.stdout).unwrap();
    assert!(j_zh["matches"][0]["rule"]
        .as_str()
        .unwrap()
        .contains("独立研究"));
}

#[test]
fn test_guard_shield_loop_breaker_and_dual_hooks() {
    let tmp = TempDir::new().unwrap();
    let guard_home = tmp.path().join("guard_home");

    // 1. Direct exit-2 shield block
    let blk = run_ap(
        tmp.path(),
        &["guard", "shield", "--text", "git push --force origin main"],
        None,
        &[("GUARD_HOME", &guard_home)],
    );
    assert_eq!(blk.status.code(), Some(2));

    // 2. Claude/Codex snake_case hook -> exit 2
    let cl_blk = run_ap(
        tmp.path(),
        &["guard", "shield"],
        Some(r#"{"hook_event_name":"PreToolUse","tool_input":{"command":"rm -rf /"}}"#),
        &[("GUARD_HOME", &guard_home)],
    );
    assert_eq!(cl_blk.status.code(), Some(2));

    // 3. Antigravity camelCase hook -> exit 0 + {"decision":"deny"}
    let agy_blk = run_ap(
        tmp.path(),
        &["guard", "shield"],
        Some(
            r#"{"hookEventName":"PreToolUse","toolCall":{"command":"echo sk-proj-1234567890abcdefghijklmnop"}}"#,
        ),
        &[("GUARD_HOME", &guard_home)],
    );
    assert_eq!(agy_blk.status.code(), Some(0));
    let j_agy: Value = serde_json::from_slice(&agy_blk.stdout).unwrap();
    assert_eq!(j_agy["decision"], "deny");

    // 4. Antigravity Stop hook continuationActive loop prevention
    let agy_stop = run_ap(
        tmp.path(),
        &["guard", "stop-gate"],
        Some(
            r#"{"hookEventName":"Stop","continuationActive":true,"terminationReason":"NO_TOOL_CALL"}"#,
        ),
        &[("GUARD_HOME", &guard_home)],
    );
    let j_stop: Value = serde_json::from_slice(&agy_stop.stdout).unwrap();
    assert_eq!(j_stop["decision"], "stop");
}

#[test]
fn test_atlas_and_probe_slice_polyglot_rust_ts_and_c() {
    let tmp = TempDir::new().unwrap();
    let rs_repo = tmp.path().join("rs_proj");
    fs::create_dir_all(rs_repo.join("src")).unwrap();
    fs::write(
        rs_repo.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        rs_repo.join("src/lib.rs"),
        "pub mod engine;\npub fn boot() -> i64 { engine::Engine::new().tick() }\n",
    )
    .unwrap();
    fs::write(
        rs_repo.join("src/engine.rs"),
        "pub struct Engine { n: i64 }\nimpl Engine {\n    pub fn new() -> Self { Self { n: 1 } }\n    pub fn tick(&self) -> i64 { let _raw = r#\"{\"#; self.n + 1 }\n}\n",
    ).unwrap();
    fs::write(
        rs_repo.join("src/dead_legacy.rs"),
        "pub fn unused_fn() -> i64 { 42 }\n",
    )
    .unwrap();

    let atlas_out = run_ap(&rs_repo, &["atlas", "map", ".", "--json"], None, &[]);
    assert!(atlas_out.status.success());
    let j_atlas: Value = serde_json::from_slice(&atlas_out.stdout).unwrap();
    assert_eq!(j_atlas["scanned_files"], 4);

    let slice_out = run_ap(
        &rs_repo,
        &["probe", "slice", ".", "--entry", "src/lib.rs", "--bundle"],
        None,
        &[],
    );
    assert!(slice_out.status.success());
    let slice_txt = String::from_utf8_lossy(&slice_out.stdout);
    assert!(slice_txt.contains("2/3 reachable files (1 dead/unreferenced files pruned)"));
    assert!(slice_txt.contains("### Cargo.toml"));
    assert!(slice_txt.contains("### src/engine.rs"));

    // C header + translation unit reachability slice
    let c_repo = tmp.path().join("c_proj");
    fs::create_dir_all(&c_repo).unwrap();
    fs::write(
        c_repo.join("ring.h"),
        "typedef struct Ring { int cap; } Ring;\nint ring_push(Ring *r, int v);\n",
    )
    .unwrap();
    fs::write(
        c_repo.join("ring.c"),
        "#include \"ring.h\"\nint ring_push(Ring *r, int v) { return r->cap + v; }\n",
    )
    .unwrap();
    fs::write(
        c_repo.join("main.c"),
        "#include \"ring.h\"\nint main(void) { Ring r = {8}; return ring_push(&r, 1); }\n",
    )
    .unwrap();
    fs::write(
        c_repo.join("dead_util.c"),
        "int dead_fn(void) { return 0; }\n",
    )
    .unwrap();

    let c_slice = run_ap(
        &c_repo,
        &["probe", "slice", ".", "--entry", "main.c", "--json"],
        None,
        &[],
    );
    assert!(c_slice.status.success());
    let j_cs: Value = serde_json::from_slice(&c_slice.stdout).unwrap();
    assert_eq!(j_cs["reachable_count"], 3);
    assert_eq!(j_cs["pruned_unreferenced_count"], 1);
}

#[test]
fn test_splice_treesitter_view_zero_escaping_apply_and_syntax_rollback() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("splice_rs");
    fs::create_dir_all(repo.join("src")).unwrap();

    // Include raw string with unbalanced `{` inside Engine::calc — regex heuristics fail, Tree-sitter succeeds!
    let orig_rs = "pub struct Engine;\r\nimpl Engine {\r\n    pub fn calc(&self, x: i64) -> i64 {\r\n        let _brace_trap = r#\"{{{\"#;\r\n        x + 1\r\n    }\r\n}\r\n";
    let lib_p = repo.join("src/lib.rs");
    fs::write(&lib_p, orig_rs.as_bytes()).unwrap();

    let view_out = run_ap(
        &repo,
        &["splice", "view", "src/lib.rs", "--symbol", "Engine.calc"],
        None,
        &[],
    );
    assert!(view_out.status.success());
    let view_s = String::from_utf8_lossy(&view_out.stdout);
    assert!(view_s.contains("L3-L6"));

    // 1. Broken Rust syntax in transaction MUST abort & roll back ALL files atomically!
    let bad_patch = "@@ SYMBOL: src/lib.rs :: Engine.calc\n<<<\npub fn calc(&self, x: i64) -> i64 {\n    x +\n}\n>>>\n@@ WRITE: src/extra.rs\n<<<\npub fn ok() {}\n>>>\n";
    let bad_out = run_ap(
        &repo,
        &["splice", "apply", "--cwd", ".", "--stdin", "--json"],
        Some(bad_patch),
        &[],
    );
    assert!(!bad_out.status.success());
    let j_bad: Value = serde_json::from_slice(&bad_out.stdout).unwrap();
    assert_eq!(j_bad["rolled_back"], true);
    assert!(
        !repo.join("src/extra.rs").exists(),
        "newly staged file must be unlinked on rollback"
    );
    assert_eq!(fs::read(&lib_p).unwrap(), orig_rs.as_bytes());

    // 2. Valid zero-escaping multi-op transaction preserves CRLF and passes Tree-sitter check
    let good_patch = "@@ SYMBOL: src/lib.rs :: Engine.calc\n<<<\npub fn calc(&self, x: i64) -> i64 {\n    x * 2\n}\n>>>\n@@ WRITE: src/helper.c\n<<<\nint add_one(int v) {\n    return v + 1;\n}\n>>>\n";
    let good_out = run_ap(
        &repo,
        &["splice", "apply", "--cwd", ".", "--stdin", "--json"],
        Some(good_patch),
        &[],
    );
    assert!(good_out.status.success());
    let updated_bytes = fs::read(&lib_p).unwrap();
    assert!(
        updated_bytes.windows(2).any(|w| w == b"\r\n"),
        "CRLF must be preserved"
    );
    assert!(String::from_utf8_lossy(&updated_bytes).contains("x * 2"));

    let chk = run_ap(&repo, &["splice", "check", ".", "--json"], None, &[]);
    assert!(chk.status.success());
    let j_chk: Value = serde_json::from_slice(&chk.stdout).unwrap();
    assert_eq!(j_chk["ok"], true);
    assert_eq!(j_chk["checked_rs_go_c_files"], 2);
}

#[test]
fn test_probe_rust_differential_oracle() {
    let tmp = TempDir::new().unwrap();
    if Command::new("rustc").arg("--version").output().is_err() {
        return;
    }
    let diff_out = run_ap(
        tmp.path(),
        &[
            "probe",
            "diff",
            "--lang",
            "rust",
            "--fn-a",
            "|x: i64| (x * 250 + 5000) / 10000",
            "--fn-b",
            "|x: i64| ((x as f64 / 100.0) * 2.5).round() as i64",
            "--json",
        ],
        None,
        &[],
    );
    // Negative inputs diverge because `(x * 250 + 5000) / 10000` rounds wrong on negatives!
    assert_eq!(diff_out.status.code(), Some(1));
    let j_diff: Value = serde_json::from_slice(&diff_out.stdout).unwrap();
    assert_eq!(j_diff["equivalent"], false);
    assert!(!j_diff["divergences"].as_array().unwrap().is_empty());
}

#[test]
fn test_ap_init_here_global_hooks_uninstall_doctor_and_audit() {
    let tmp = TempDir::new().unwrap();
    let fake_home = tmp.path().join("home");
    let proj = tmp.path().join("proj");
    fs::create_dir_all(&fake_home).unwrap();
    fs::create_dir_all(&proj).unwrap();

    // 1. Doctor
    let doc = run_ap(tmp.path(), &["doctor"], None, &[]);
    assert!(doc.status.success());

    // 2. Repo-local `ap init --here --with-hooks`
    let init_here = run_ap(
        &proj,
        &[
            "init",
            "--here",
            "--home",
            fake_home.to_str().unwrap(),
            "--with-hooks",
        ],
        None,
        &[],
    );
    assert!(init_here.status.success());
    assert!(proj.join(".agents/skills/probe/SKILL.md").is_file());
    assert!(proj.join("AGENTS.md").is_file());
    assert!(proj.join(".agents/hooks.json").is_file());

    let un_here = run_ap(
        &proj,
        &["uninstall", "--here", "--home", fake_home.to_str().unwrap()],
        None,
        &[],
    );
    assert!(un_here.status.success());
    assert!(!proj.join("AGENTS.md").exists());

    // 3. Global multi-harness onboarding + clean uninstall
    let init_glob = run_ap(
        tmp.path(),
        &[
            "init",
            "-a",
            "codex,pi,agy,claude,grok",
            "--home",
            fake_home.to_str().unwrap(),
            "--with-hooks",
        ],
        None,
        &[],
    );
    assert!(init_glob.status.success());
    assert!(fake_home.join(".codex/skills/splice/SKILL.md").is_file());
    assert!(fake_home.join(".pi/agent/skills/probe/SKILL.md").is_file());
    assert!(fake_home.join(".gemini/config/hooks.json").is_file());
    assert!(fake_home.join(".claude/settings.json").is_file());

    let st = run_ap(
        tmp.path(),
        &["status", "--home", fake_home.to_str().unwrap()],
        None,
        &[],
    );
    assert!(st.status.success());

    let un_glob = run_ap(
        tmp.path(),
        &["uninstall", "--home", fake_home.to_str().unwrap()],
        None,
        &[],
    );
    assert!(un_glob.status.success());
    assert!(!fake_home.join(".gemini/config/hooks.json").exists());

    // 4. Local telemetry audit with synthetic Pi & Codex session logs
    let pi_sess = fake_home.join(".pi/agent/sessions");
    fs::create_dir_all(&pi_sess).unwrap();
    fs::write(
        pi_sess.join("s1.jsonl"),
        "{\"type\":\"message\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"toolCall\",\"name\":\"bash\",\"arguments\":{\"command\":\"python -c 'import x'\"}}]}}\n\
         {\"type\":\"message\",\"message\":{\"role\":\"toolResult\",\"content\":[{\"text\":\"ModuleNotFoundError: No module named x\"}]}}\n",
    )
    .unwrap();
    fs::write(
        pi_sess.join("s2.jsonl"),
        "{\"type\":\"message\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"toolCall\",\"name\":\"read\",\"arguments\":{\"path\":\"a.rs\"}},{\"type\":\"toolCall\",\"name\":\"read\",\"arguments\":{\"path\":\"a.rs\"}}]}}\n",
    )
    .unwrap();

    let aud = run_ap(
        tmp.path(),
        &[
            "audit",
            "--home",
            fake_home.to_str().unwrap(),
            "--compare",
            "--json",
        ],
        None,
        &[],
    );
    assert!(aud.status.success());
    let j_aud: Value = serde_json::from_slice(&aud.stdout).unwrap();
    assert_eq!(j_aud["harnesses_detected"]["pi_cli"]["sessions"], 2);
    assert_eq!(j_aud["aggregate_friction_counts"]["adhoc_inline_python"], 1);
    assert_eq!(
        j_aud["aggregate_friction_counts"]["redundant_same_file_rereads"],
        1
    );
}

#[test]
fn test_probe_ts_and_py_runtime_microscopes() {
    let tmp = TempDir::new().unwrap();

    // Python runtime microscope: alias + mutation + state no-op + diff True==1 cache drift
    if Command::new("python").arg("--version").output().is_ok()
        || Command::new("python3").arg("--version").output().is_ok()
    {
        let py_dir = tmp.path().join("py_lab");
        fs::create_dir_all(&py_dir).unwrap();
        fs::write(
            py_dir.join("engine.py"),
            "class Box:\n    def __init__(self, d):\n        self.d = d\n    def clone(self):\n        return Box(dict(self.d))\n    def process(self, items):\n        items.append(99)\n        return len(items)\n    def revoke(self, k):\n        self.d.pop('wrong_' + k, None)\n",
        )
        .unwrap();
        let out_py = run_ap(
            &py_dir,
            &[
                "probe",
                "run",
                "--cwd",
                ".",
                "--lang",
                "py",
                "--code",
                "from engine import Box; b = Box({'meta': [1]}); c = b.clone(); b.process([1,2]); b.revoke('meta')",
                "--json",
            ],
            None,
            &[],
        );
        assert!(out_py.status.success());
        let j_py: Value = serde_json::from_slice(&out_py.stdout).unwrap();
        assert!(!j_py["alias_hazards"].as_array().unwrap().is_empty());
        assert!(!j_py["mutation_alerts"].as_array().unwrap().is_empty());
        assert!(!j_py["state_noop_alerts"].as_array().unwrap().is_empty());
    }

    // TypeScript Node 22 V8 Proxy microscope
    if let Ok(no) = Command::new("node").arg("-v").output() {
        let major = String::from_utf8_lossy(&no.stdout)
            .trim()
            .trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|m| m.parse::<u32>().ok())
            .unwrap_or(0);
        if major >= 22 {
            let ts_dir = tmp.path().join("ts_lab");
            fs::create_dir_all(&ts_dir).unwrap();
            fs::write(
                ts_dir.join("ledger.ts"),
                "export class Ledger {\n  items: { tags: string[] } = { tags: ['a'] };\n  snapshot() { return { ...this.items }; }\n  fee(x: number): number { return Math.round(x * 0.01); }\n}\n",
            )
            .unwrap();
            let out_ts = run_ap(
                &ts_dir,
                &[
                    "probe",
                    "run",
                    "--cwd",
                    ".",
                    "--lang",
                    "ts",
                    "--code",
                    "import { Ledger } from './ledger.js'; const l = new Ledger(); l.snapshot(); l.fee(-1);",
                    "--json",
                ],
                None,
                &[],
            );
            assert!(out_ts.status.success());
            let j_ts: Value = serde_json::from_slice(&out_ts.stdout).unwrap();
            assert!(!j_ts["alias_hazards"].as_array().unwrap().is_empty());
            assert!(!j_ts["numeric_hazards"].as_array().unwrap().is_empty());
        }
    }
}

#[test]
fn test_splice_apply_with_test_and_rollback_on_test_fail() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("rb_proj");
    fs::create_dir_all(&repo).unwrap();
    let src_p = repo.join("data.json");
    fs::write(&src_p, "{\n  \"version\": 1\n}\n").unwrap();

    let patch = "@@ FILE: data.json\n<<<\n  \"version\": 1\n===\n  \"version\": 2\n>>>\n@@ WRITE: extra.json\n<<<\n{\"ok\": true}\n>>>\n";
    let fail_cmd = if cfg!(windows) { "exit /b 1" } else { "exit 1" };
    let out = run_ap(
        &repo,
        &[
            "splice",
            "apply",
            "--cwd",
            ".",
            "--stdin",
            "--test-cmd",
            fail_cmd,
            "--rollback-on-test-fail",
            "--json",
        ],
        Some(patch),
        &[],
    );
    assert!(!out.status.success());
    let j: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(j["rolled_back"], true);
    assert!(!repo.join("extra.json").exists());
    assert!(fs::read_to_string(&src_p)
        .unwrap()
        .contains("\"version\": 1"));
}

#[test]
fn test_guard_loop_breaker_and_busybox_argv0() {
    let tmp = TempDir::new().unwrap();
    let guard_home = tmp.path().join("guard_loop");

    // Busybox argv[0] dispatch: copy/hardlink `ap` as `baton`
    let baton_bin = tmp
        .path()
        .join(if cfg!(windows) { "baton.exe" } else { "baton" });
    if fs::hard_link(ap_bin(), &baton_bin).is_err() {
        fs::copy(ap_bin(), &baton_bin).unwrap();
    }
    let b_out = Command::new(&baton_bin)
        .args(["set", "--goal", "Busybox argv0 test"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(b_out.status.success());

    // Guard repetition loop breaker trips exit 2 on 3rd identical failure
    let fail_args: &[&str] = if cfg!(windows) {
        &[
            "guard",
            "run",
            "--max-repeats",
            "3",
            "--",
            "cmd",
            "/C",
            "echo boom && exit /b 1",
        ]
    } else {
        &[
            "guard",
            "run",
            "--max-repeats",
            "3",
            "--",
            "sh",
            "-c",
            "echo boom && exit 1",
        ]
    };
    let r1 = run_ap(tmp.path(), fail_args, None, &[("GUARD_HOME", &guard_home)]);
    let r2 = run_ap(tmp.path(), fail_args, None, &[("GUARD_HOME", &guard_home)]);
    let r3 = run_ap(tmp.path(), fail_args, None, &[("GUARD_HOME", &guard_home)]);
    assert_eq!(r1.status.code(), Some(1));
    assert_eq!(r2.status.code(), Some(1));
    assert_eq!(
        r3.status.code(),
        Some(2),
        "3rd identical failure must trip exit-2 loop breaker"
    );
}

#[test]
fn test_atlas_product_probe_smoke_and_guard_rules() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("public")).unwrap();

    fs::write(
        root.join("public/index.html"),
        r#"<!doctype html><html><head><title>Demo Board</title></head>
<body><input id="q-input" placeholder="Search..." /><button id="btn-save">Save</button></body></html>"#,
    )
    .unwrap();
    fs::write(
        root.join("public/app.js"),
        r#"async function save() {
  try { await fetch('/api/items/wrong-route', { method: 'POST' }); } catch (e) {}
}"#,
    )
    .unwrap();
    fs::write(
        root.join("src/routes.ts"),
        r#"export function handle(method: string, pathname: string, payload: any) {
  if (method === 'GET' && pathname === '/api/items') return payload;
  if (method === 'POST' && pathname === '/api/items/save') return payload;
}"#,
    )
    .unwrap();

    // 1. `atlas product` detects HTML controls, backend routes, broken link & slop smells
    let p_out = run_ap(
        root,
        &[
            "atlas",
            "product",
            ".",
            "--write",
            "PRODUCT_MAP.md",
            "--json",
        ],
        None,
        &[],
    );
    assert!(
        p_out.status.success(),
        "atlas product failed: {}",
        String::from_utf8_lossy(&p_out.stderr)
    );
    assert!(root.join("PRODUCT_MAP.md").is_file());
    let pj: Value = serde_json::from_slice(&p_out.stdout).unwrap();
    let mismatches = pj["contract_mismatches"].as_array().unwrap();
    assert_eq!(mismatches.len(), 1);
    assert!(mismatches[0]
        .as_str()
        .unwrap()
        .contains("/api/items/wrong-route"));
    assert!(!pj["slop_smells"].as_array().unwrap().is_empty());

    // 2. `guard rule init/add` + `guard verify --all` blocks `: any` and empty `catch {}` with exit 2
    assert!(
        run_ap(root, &["guard", "rule", "init", "--cwd", "."], None, &[])
            .status
            .success()
    );
    assert!(run_ap(
        root,
        &[
            "guard",
            "rule",
            "add",
            "--cwd",
            ".",
            "--id",
            "no-wrong-route",
            "--glob",
            "*.js",
            "--forbid",
            "wrong-route",
            "--msg",
            "Use /api/items/save",
        ],
        None,
        &[]
    )
    .status
    .success());
    let v_fail = run_ap(root, &["guard", "verify", "--cwd", ".", "--all"], None, &[]);
    assert_eq!(
        v_fail.status.code(),
        Some(2),
        "guard verify must return exit 2 on slop/rule violation"
    );

    // Clean the slop & broken route -> `guard verify --all` passes
    fs::write(
        root.join("public/app.js"),
        "async function save() { const r = await fetch('/api/items/save', { method: 'POST' }); return r.ok; }\n",
    )
    .unwrap();
    fs::write(
        root.join("src/routes.ts"),
        "export function handle(method: string, pathname: string, payload: Record<string, string>) {\n  if (method === 'POST' && pathname === '/api/items/save') return payload;\n  return null;\n}\n",
    )
    .unwrap();
    let v_ok = run_ap(root, &["guard", "verify", "--cwd", ".", "--all"], None, &[]);
    assert!(
        v_ok.status.success(),
        "guard verify should pass once slop is cleaned: {}",
        String::from_utf8_lossy(&v_ok.stderr)
    );

    // 3. `probe smoke` spawns ephemeral Node HTTP server, tests UTF-8 Chinese query + JSON dot-path & cleans tree
    if Command::new("node").arg("--version").output().is_ok() {
        fs::write(
            root.join("mini_srv.mjs"),
            r#"import http from 'node:http';
const port = Number(process.env.PORT || 0);
http.createServer((req, res) => {
  const u = new URL(req.url, 'http://127.0.0.1');
  res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8' });
  res.end(JSON.stringify({ ok: true, q: u.searchParams.get('q') || '', items: [1, 2, 3] }));
}).listen(port, '127.0.0.1');
"#,
        )
        .unwrap();
        let smoke_spec = "GET /api/search?q=支付\nEXPECT STATUS 200\nEXPECT CONTAINS 支付\nEXPECT JSON items.length == 3\n";
        let s_out = run_ap(
            root,
            &[
                "probe",
                "smoke",
                "--cwd",
                ".",
                "--start",
                "node mini_srv.mjs",
                "--stdin",
                "--json",
            ],
            Some(smoke_spec),
            &[],
        );
        assert!(
            s_out.status.success(),
            "probe smoke failed: {}{}",
            String::from_utf8_lossy(&s_out.stdout),
            String::from_utf8_lossy(&s_out.stderr)
        );
        let sj: Value = serde_json::from_slice(&s_out.stdout).unwrap();
        assert_eq!(sj["ok"], true);
    }
}

#[test]
fn test_mcp_stdio_jsonrpc_server_and_opencode_wiring() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // 1. Verify `ap init --here -a opencode` writes valid opencode.json MCP config
    let init_oc = run_ap(root, &["init", "--here", "-a", "opencode"], None, &[]);
    assert!(init_oc.status.success());
    assert!(root.join("opencode.json").is_file());
    let oc_val: Value =
        serde_json::from_str(&fs::read_to_string(root.join("opencode.json")).unwrap()).unwrap();
    assert_eq!(oc_val["mcp"]["agent-primitives"]["enabled"], true);

    // 2. Test `ap mcp` newline-delimited JSON-RPC 2.0 (`initialize`, `tools/list`, `tools/call` ap_edit + ap_verify)
    let target_py = root.join("calc.py");
    let requests = [
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "protocolVersion": "2024-11-05" }
        })
        .to_string(),
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        })
        .to_string(),
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "ap_edit",
                "arguments": {
                    "cwd": root.to_string_lossy(),
                    "writes": [{ "path": "calc.py", "content": "def add(a: int, b: int) -> int:\n    return a + b\n" }]
                }
            }
        })
        .to_string(),
    ]
    .join("\n")
        + "\n";

    let mcp_out = run_ap(root, &["mcp"], Some(&requests), &[]);
    assert!(mcp_out.status.success());
    let lines: Vec<&str> = std::str::from_utf8(&mcp_out.stdout)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 3);
    let r2: Value = serde_json::from_str(lines[1]).unwrap();
    assert!(r2["result"]["tools"].as_array().unwrap().len() >= 5);
    let r3: Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(r3["result"]["isError"], false);
    assert!(target_py.is_file());

    // 3. Clean uninstall removes opencode.json
    let un_oc = run_ap(root, &["uninstall", "--here"], None, &[]);
    assert!(un_oc.status.success());
    assert!(!root.join("opencode.json").exists());
}

#[test]
fn test_aider_indentation_patch_fallback_and_rtk_pretooluse_rewrite() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // 1. Aider Tier-3 uniform indentation-delta match (file has 4-space indent, LLM emits 0-space `old` & `new`)
    let py_f = root.join("service.py");
    fs::write(
        &py_f,
        "class OrderService:\n    def total(self, a: int, b: int) -> int:\n        tax = 2\n        return a + b + tax\n",
    )
    .unwrap();
    let patch_0_indent = "@@ FILE: service.py\n<<<\ndef total(self, a: int, b: int) -> int:\n    tax = 2\n    return a + b + tax\n===\ndef total(self, a: int, b: int) -> int:\n    tax = 5\n    return a + b + tax\n>>>\n";
    let p_out = run_ap(
        root,
        &["splice", "apply", "--cwd", ".", "--stdin"],
        Some(patch_0_indent),
        &[],
    );
    assert!(
        p_out.status.success(),
        "Tier-3 indentation fallback failed: {}",
        String::from_utf8_lossy(&p_out.stderr)
    );
    let updated = fs::read_to_string(&py_f).unwrap();
    assert!(updated.contains("    def total(self, a: int, b: int) -> int:\n        tax = 5\n"));

    // 2. RTK-style PreToolUse `updatedInput` auto-sieve rewrite for Claude Code Bash hook
    let hook_in = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": { "command": "cargo test --workspace" }
    })
    .to_string();
    let g_out = run_ap(root, &["guard", "shield"], Some(&hook_in), &[]);
    assert!(g_out.status.success());
    let gj: Value = serde_json::from_slice(&g_out.stdout).unwrap();
    let rewritten = gj["hookSpecificOutput"]["updatedInput"]["command"]
        .as_str()
        .unwrap();
    assert!(rewritten.contains("sieve run -- cargo test --workspace"));
}

#[test]
fn test_windows_exec_shield_bom_utf16_splice_and_stats_ledger() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let ap_home = root.join("ap_stats_home");

    // 1. UTF-8 BOM (0xEF,0xBB,0xBF) + UTF-16LE BOM (0xFF,0xFE) + CRLF patching in `ap splice`
    let bom_utf8_file = root.join("config.ts");
    let mut utf8_bom_bytes = b"\xEF\xBB\xBF".to_vec();
    utf8_bom_bytes.extend_from_slice(
        "export const TITLE = \"旧标题\";\r\nexport const PORT = 3000;\r\n".as_bytes(),
    );
    fs::write(&bom_utf8_file, utf8_bom_bytes).unwrap();

    let utf16_file = root.join("ps51_note.js");
    let mut utf16_bytes = vec![0xFF, 0xFE];
    for u in "const mode = 'legacy';\r\n".encode_utf16() {
        utf16_bytes.extend_from_slice(&u.to_le_bytes());
    }
    fs::write(&utf16_file, utf16_bytes).unwrap();

    let patch_bom = "@@ FILE: config.ts\n<<<\nexport const TITLE = \"旧标题\";\n===\nexport const TITLE = \"新标题\";\n>>>\n@@ FILE: ps51_note.js\n<<<\nconst mode = 'legacy';\n===\nconst mode = 'utf8-clean';\n>>>\n";
    let sp_out = run_ap(
        root,
        &["splice", "apply", "--cwd", ".", "--stdin"],
        Some(patch_bom),
        &[("AP_HOME", &ap_home)],
    );
    assert!(
        sp_out.status.success(),
        "BOM/UTF-16LE splice failed: {}",
        String::from_utf8_lossy(&sp_out.stderr)
    );
    let after_ts = fs::read(&bom_utf8_file).unwrap();
    assert!(
        !after_ts.starts_with(b"\xEF\xBB\xBF"),
        "UTF-8 BOM must be stripped cleanly"
    );
    assert!(std::str::from_utf8(&after_ts).unwrap().contains("新标题"));

    let after_js = fs::read(&utf16_file).unwrap();
    assert!(
        !after_js.starts_with(&[0xFF, 0xFE]),
        "UTF-16LE BOM must be converted to clean UTF-8"
    );
    assert!(std::str::from_utf8(&after_js)
        .unwrap()
        .contains("utf8-clean"));

    // 2. Transparent Windows PreToolUse hook Base64 auto-heal (`AP_FORCE_WIN_SHIELD=1`)
    let win_hook_in = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": { "command": "python -c \"print('中文UTF8')\" && curl -s https://example.com/api?a=1&b=2 > /dev/null" }
    })
    .to_string();
    let mut shield_cmd = Command::new(ap_bin());
    shield_cmd
        .args(["guard", "shield"])
        .current_dir(root)
        .env("AP_FORCE_WIN_SHIELD", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut sh_child = shield_cmd.spawn().unwrap();
    sh_child
        .stdin
        .take()
        .unwrap()
        .write_all(win_hook_in.as_bytes())
        .unwrap();
    let sh_out = sh_child.wait_with_output().unwrap();
    assert!(sh_out.status.success());
    let sh_j: Value = serde_json::from_slice(&sh_out.stdout).unwrap();
    let rewritten = sh_j["hookSpecificOutput"]["updatedInput"]["command"]
        .as_str()
        .unwrap();
    assert!(
        rewritten.contains("exec --base64 "),
        "Expected Base64 zero-quoting Windows exec rewrite, got: {rewritten}"
    );

    // 3. `ap exec -c` UTF-8 + `&&` chain + large output auto-sieve + `ap stats` (`ap gain`) ledger
    let noisy_cmd = if Command::new("node").arg("--version").output().is_ok() {
        "node -e \"for(let i=0;i<120;i++) console.log('build step ' + i + ' 中文正常')\" && echo CHAIN_OK"
    } else {
        "echo first_ok && echo CHAIN_OK"
    };
    let ex_out = run_ap(
        root,
        &["exec", "-c", noisy_cmd, "--json"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(
        ex_out.status.success(),
        "ap exec failed: {}{}",
        String::from_utf8_lossy(&ex_out.stdout),
        String::from_utf8_lossy(&ex_out.stderr)
    );
    let ex_j: Value = serde_json::from_slice(&ex_out.stdout).unwrap();
    assert_eq!(ex_j["ok"], true);
    assert!(ex_j["output"].as_str().unwrap().contains("CHAIN_OK"));

    // 4. Verify `ap stats` / `ap gain --json` recorded real savings & resets cleanly
    let st_out = run_ap(root, &["gain", "--json"], None, &[("AP_HOME", &ap_home)]);
    assert!(st_out.status.success());
    let st_j: Value = serde_json::from_slice(&st_out.stdout).unwrap();
    assert!(st_j["total_calls"].as_u64().unwrap() >= 1);

    let rst_out = run_ap(root, &["stats", "--reset"], None, &[("AP_HOME", &ap_home)]);
    assert!(rst_out.status.success());
}

#[test]
fn test_ast_impact_proactive_blast_radius_win_job_and_ide_mcp() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("multi_mod");
    let fake_home = tmp.path().join("ide_home");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::create_dir_all(fake_home.join(".cursor")).unwrap();
    fs::create_dir_all(fake_home.join(".codeium/windsurf")).unwrap();

    fs::write(
        root.join("src/pricing.ts"),
        "export function computeTotal(items: number[]): number {\n  return items.reduce((a, b) => a + b, 0);\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/checkout.ts"),
        "import { computeTotal } from './pricing';\nexport function checkoutCart(xs: number[]) {\n  return computeTotal(xs);\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/invoice.ts"),
        "import { computeTotal } from './pricing';\nexport function renderInvoice(xs: number[]) {\n  const sum = computeTotal(xs);\n  return `Sum=${sum}`;\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/noise.ts"),
        "// computeTotal should not match inside comments\nexport const MSG = 'computeTotal string literal should be ignored';\n",
    )
    .unwrap();
    fs::write(
        root.join("tests/pricing.test.ts"),
        "import { computeTotal } from '../src/pricing';\nif (computeTotal([10, 20]) !== 30) throw new Error('bad');\n",
    )
    .unwrap();

    // 1. Tree-sitter AST `atlas impact` ignores comments/strings & separates definitions, callers, and tests
    let imp_out = run_ap(
        &root,
        &["atlas", "impact", "computeTotal", ".", "--json"],
        None,
        &[],
    );
    assert!(imp_out.status.success());
    let ij: Value = serde_json::from_slice(&imp_out.stdout).unwrap();
    assert_eq!(ij["definitions"].as_array().unwrap().len(), 1);
    let callers = ij["callers_and_refs"].as_array().unwrap();
    assert!(
        callers
            .iter()
            .any(|c| c["file"] == "src/checkout.ts" && c["kind"] == "call"),
        "Expected call site in src/checkout.ts: {callers:?}"
    );
    assert!(
        callers
            .iter()
            .any(|c| c["file"] == "src/invoice.ts" && c["kind"] == "call"),
        "Expected call site in src/invoice.ts: {callers:?}"
    );
    assert!(
        !callers.iter().any(|c| c["file"] == "src/noise.ts"),
        "Comments and string literals in src/noise.ts must NOT produce false-positive AST refs"
    );
    assert!(ij["test_refs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["file"] == "tests/pricing.test.ts"));

    // 2. Proactive `[BLAST RADIUS]` warning when editing only `src/pricing.ts` signature (2-of-7 partial refactor guard)
    let partial_patch = "@@ FILE: src/pricing.ts\n<<<\nexport function computeTotal(items: number[]): number {\n  return items.reduce((a, b) => a + b, 0);\n}\n===\nexport function computeTotal(items: number[], taxRate: number): number {\n  return items.reduce((a, b) => a + b, 0) * (1 + taxRate);\n}\n>>>\n";
    let sp_out = run_ap(
        &root,
        &["splice", "apply", "--cwd", ".", "--stdin"],
        Some(partial_patch),
        &[],
    );
    assert!(sp_out.status.success());
    let sp_txt = String::from_utf8_lossy(&sp_out.stdout);
    assert!(
        sp_txt.contains("[BLAST RADIUS]")
            && sp_txt.contains("src/checkout.ts")
            && sp_txt.contains("src/invoice.ts")
            && sp_txt.contains("tests/pricing.test.ts"),
        "Expected proactive BLAST RADIUS warning listing untouched callers, got:\n{sp_txt}"
    );

    // 3. Win32 Kernel Job Object + timeout process-tree reaping (`exit 124`)
    let sleep_cmd = if cfg!(windows) {
        "ping -n 10 127.0.0.1 > $null"
    } else {
        "sleep 10"
    };
    let to_out = run_ap(
        &root,
        &["exec", "--timeout", "1", "-c", sleep_cmd],
        None,
        &[("AP_HOME", &fake_home)],
    );
    assert_eq!(to_out.status.code(), Some(124));

    // 4. Cursor, Windsurf, and VS Code Copilot MCP auto-wiring & clean uninstall
    let ide_global = run_ap(
        &root,
        &["init", "--home", fake_home.to_str().unwrap()],
        None,
        &[],
    );
    assert!(ide_global.status.success());
    assert!(fake_home.join(".cursor/mcp.json").is_file());
    assert!(fake_home
        .join(".codeium/windsurf/mcp_config.json")
        .is_file());

    let vs_here = run_ap(&root, &["init", "--here", "-a", "vscode"], None, &[]);
    assert!(vs_here.status.success());
    let vs_json: Value =
        serde_json::from_str(&fs::read_to_string(root.join(".vscode/mcp.json")).unwrap()).unwrap();
    assert_eq!(vs_json["servers"]["agent-primitives"]["type"], "stdio");

    assert!(run_ap(&root, &["uninstall", "--here"], None, &[])
        .status
        .success());
    assert!(!root.join(".vscode/mcp.json").exists());
    assert!(run_ap(
        &root,
        &["uninstall", "--home", fake_home.to_str().unwrap()],
        None,
        &[]
    )
    .status
    .success());
    assert!(!fake_home.join(".cursor/mcp.json").exists());
}

#[test]
fn test_general_agent_docx_xlsx_gbk_read_undo_and_claude_desktop() {
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let tmp = TempDir::new().unwrap();
    let space_dir = tmp.path().join("Desktop Reports 2026");
    let ap_home = tmp.path().join("ap_general_home");
    let cd_home = tmp.path().join("cd_home");
    fs::create_dir_all(&space_dir).unwrap();
    fs::create_dir_all(cd_home.join("AppData/Roaming/Claude")).unwrap();

    // 1. Synthesize a real .docx ZIP (`word/document.xml`)
    let docx_p = space_dir.join("Q4 经营纪要.docx");
    {
        let f = fs::File::create(&docx_p).unwrap();
        let mut zw = ZipWriter::new(f);
        zw.start_file("word/document.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document><w:body>
  <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Q4 经营复盘</w:t></w:r></w:p>
  <w:tbl>
    <w:tr><w:tc><w:t>部门</w:t></w:tc><w:tc><w:t>达成率</w:t></w:tc></w:tr>
    <w:tr><w:tc><w:t>华东区</w:t></w:tc><w:tc><w:t>118%</w:t></w:tc></w:tr>
  </w:tbl>
</w:body></w:document>"#
                .as_bytes(),
        )
        .unwrap();
        zw.finish().unwrap();
    }
    let docx_out = run_ap(
        tmp.path(),
        &["read", docx_p.to_str().unwrap()],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(docx_out.status.success());
    let docx_s = String::from_utf8_lossy(&docx_out.stdout);
    assert!(docx_s.contains("# Q4 经营复盘") && docx_s.contains("| 华东区 | 118% |"));

    // 2. Synthesize a real .xlsx ZIP (`xl/workbook.xml` + `xl/sharedStrings.xml` + `xl/worksheets/sheet1.xml`)
    let xlsx_p = space_dir.join("财务台账.xlsx");
    {
        let f = fs::File::create(&xlsx_p).unwrap();
        let mut zw = ZipWriter::new(f);
        zw.start_file("xl/workbook.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            r#"<workbook><sheets><sheet name="报销单" sheetId="1"/></sheets></workbook>"#
                .as_bytes(),
        )
        .unwrap();
        zw.start_file("xl/sharedStrings.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            r#"<sst><si><t>项目</t></si><si><t>金额</t></si><si><t>服务器扩容</t></si></sst>"#
                .as_bytes(),
        )
        .unwrap();
        zw.start_file("xl/worksheets/sheet1.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            br#"<worksheet><sheetData>
              <row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row>
              <row r="2"><c r="A2" t="s"><v>2</v></c><c r="B2"><v>4200</v></c></row>
            </sheetData></worksheet>"#,
        )
        .unwrap();
        zw.finish().unwrap();
    }
    let xlsx_out = run_ap(
        tmp.path(),
        &["read", xlsx_p.to_str().unwrap(), "--query", "4200"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(xlsx_out.status.success());
    let xlsx_s = String::from_utf8_lossy(&xlsx_out.stdout);
    assert!(xlsx_s.contains("报销单") && xlsx_s.contains("| 服务器扩容 | 4200 |"));

    // 3. GBK/GB18030 legacy Windows CSV bill read without mojibake
    let gbk_p = space_dir.join("支付宝账单_gbk.csv");
    let (gbk_cow, _, _) =
        encoding_rs::GB18030.encode("日期,商户,金额\r\n2026-11-18,阿里云计算,368.00\r\n");
    fs::write(&gbk_p, gbk_cow.as_ref()).unwrap();
    let gbk_out = run_ap(
        tmp.path(),
        &["read", gbk_p.to_str().unwrap()],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(gbk_out.status.success());
    let gbk_s = String::from_utf8_lossy(&gbk_out.stdout);
    assert!(gbk_s.contains("阿里云计算") && gbk_s.contains("gb18030/gbk"));

    // 4. Non-Git Desktop folder: `ap_edit` truncation warning + 1-command `ap undo` recovery!
    let note_p = space_dir.join("重要会议纪要.md");
    let orig_note: String = (1..=85)
        .map(|i| format!("第 {i} 条核心决议：保留完整历史记录\n"))
        .collect();
    fs::write(&note_p, &orig_note).unwrap();

    let mcp_bad_overwrite = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "ap_edit",
            "arguments": {
                "cwd": space_dir.to_string_lossy(),
                "writes": [{ "path": "重要会议纪要.md", "content": "# 被大模型不小心截断只剩一行\n" }]
            }
        }
    })
    .to_string()
        + "\n";
    let mcp_res = run_ap(
        &space_dir,
        &["mcp"],
        Some(&mcp_bad_overwrite),
        &[("AP_HOME", &ap_home)],
    );
    assert!(mcp_res.status.success());
    let mcp_str = String::from_utf8_lossy(&mcp_res.stdout);
    assert!(
        mcp_str.contains("TRUNCATION WARNING"),
        "Expected truncation warning on >75% file shrink, got: {mcp_str}"
    );

    // Run `ap undo` in non-Git Desktop folder -> 100% original 85 lines restored!
    let undo_out = run_ap(&space_dir, &["undo"], None, &[("AP_HOME", &ap_home)]);
    assert!(undo_out.status.success());
    assert_eq!(fs::read_to_string(&note_p).unwrap(), orig_note);

    // 5. Claude Desktop `claude_desktop_config.json` auto-wiring & clean uninstall
    assert!(run_ap(
        tmp.path(),
        &["init", "--home", cd_home.to_str().unwrap()],
        None,
        &[]
    )
    .status
    .success());
    assert!(cd_home
        .join("AppData/Roaming/Claude/claude_desktop_config.json")
        .is_file());
    assert!(run_ap(
        tmp.path(),
        &["uninstall", "--home", cd_home.to_str().unwrap()],
        None,
        &[]
    )
    .status
    .success());
    assert!(!cd_home
        .join("AppData/Roaming/Claude/claude_desktop_config.json")
        .exists());
}

#[test]
fn test_sql_query_engine_ast_office_search_and_spa_render() {
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let ap_home = root.join("ap_v16_home");

    // 1. Multi-format `ap query` SQL Join across .xlsx + GBK .csv + .jsonl (with ¥ currency strings)
    let orders_xlsx = root.join("orders.xlsx");
    {
        let f = fs::File::create(&orders_xlsx).unwrap();
        let mut zw = ZipWriter::new(f);
        zw.start_file("xl/workbook.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            r#"<workbook><sheets><sheet name="订单" sheetId="1"/><sheet name="海外" sheetId="2"/></sheets></workbook>"#.as_bytes(),
        )
        .unwrap();
        zw.start_file("xl/sharedStrings.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            r#"<sst><si><t>oid</t></si><si><t>cid</t></si><si><t>amount</t></si><si><t>O1</t></si><si><t>C1</t></si><si><t>¥1,200.50</t></si><si><t>O2</t></si><si><t>C2</t></si><si><t>¥2,000.00</t></si><si><t>O3</t></si><si><t>¥800.50</t></si><si><t>O4</t></si><si><t>¥500.00</t></si></sst>"#
                .as_bytes(),
        )
        .unwrap();
        zw.start_file("xl/worksheets/sheet1.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            br#"<worksheet><sheetData>
              <row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c><c r="C1" t="s"><v>2</v></c></row>
              <row r="2"><c r="A2" t="s"><v>3</v></c><c r="B2" t="s"><v>4</v></c><c r="C2" t="s"><v>5</v></c></row>
              <row r="3"><c r="A3" t="s"><v>6</v></c><c r="B3" t="s"><v>7</v></c><c r="C3" t="s"><v>8</v></c></row>
              <row r="4"><c r="A4" t="s"><v>9</v></c><c r="B4" t="s"><v>4</v></c><c r="C4" t="s"><v>10</v></c></row>
            </sheetData></worksheet>"#,
        )
        .unwrap();
        zw.start_file("xl/worksheets/sheet2.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            br#"<worksheet><sheetData>
              <row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c><c r="C1" t="s"><v>2</v></c></row>
              <row r="2"><c r="A2" t="s"><v>11</v></c><c r="B2" t="s"><v>4</v></c><c r="C2" t="s"><v>12</v></c></row>
            </sheetData></worksheet>"#,
        )
        .unwrap();
        zw.finish().unwrap();
    }

    let (gbk_cow, _, _) = encoding_rs::GB18030.encode("oid,paid_flag\r\nO2,1\r\n");
    fs::write(root.join("bank.csv"), gbk_cow.as_ref()).unwrap();
    fs::write(
        root.join("customers.jsonl"),
        "{\"cid\":\"C1\",\"name\":\"星海集团\"}\n{\"cid\":\"C2\",\"name\":\"远山科技\"}\n",
    )
    .unwrap();

    let sql = "SELECT c.name, ROUND(SUM(o.amount), 2) AS unpaid_sum FROM orders o JOIN customers c ON o.cid = c.cid LEFT JOIN bank b ON o.oid = b.oid WHERE b.oid IS NULL GROUP BY c.name";
    let q_out = run_ap(
        root,
        &[
            "query",
            "-f",
            "orders=orders.xlsx",
            "-f",
            "bank=bank.csv",
            "-f",
            "customers=customers.jsonl",
            "--out",
            "unpaid.csv",
            "--json",
            sql,
        ],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(
        q_out.status.success(),
        "ap query failed: {}",
        String::from_utf8_lossy(&q_out.stderr)
    );
    let qj: Value = serde_json::from_slice(&q_out.stdout).unwrap();
    assert_eq!(qj["row_count"], 1);
    assert_eq!(qj["rows"][0][0], "星海集团");
    assert_eq!(qj["rows"][0][1], "2001.0");
    assert!(root.join("unpaid.csv").is_file());

    // Multi-sheet `orders__all` auto-union + `--stdin` SQL execution
    let q_all = run_ap(
        root,
        &["query", "--table", "orders=orders.xlsx", "--stdin", "--json"],
        Some("SELECT COUNT(*), ROUND(SUM(amount), 2) FROM orders__all WHERE _sheet IN ('订单', '海外')"),
        &[("AP_HOME", &ap_home)],
    );
    assert!(q_all.status.success());
    let q_all_j: Value = serde_json::from_slice(&q_all.stdout).unwrap();
    assert_eq!(q_all_j["rows"][0][0], "4");
    assert_eq!(q_all_j["rows"][0][1], "4501.0");

    // 2. Universal `ap search` with enclosing Tree-sitter AST symbol span + .docx extraction
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/billing.ts"),
        "export class BillingEngine {\n  settleAccount(code: string): boolean {\n    const token = 'VIP_OMEGA_99';\n    return code === token;\n  }\n}\n",
    )
    .unwrap();
    let docx_p = root.join("spec.docx");
    {
        let f = fs::File::create(&docx_p).unwrap();
        let mut zw = ZipWriter::new(f);
        zw.start_file("word/document.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(br#"<w:document><w:body><w:p><w:r><w:t>Use promo token VIP_OMEGA_99 for enterprise tier.</w:t></w:r></w:p></w:body></w:document>"#)
            .unwrap();
        zw.finish().unwrap();
    }

    let s_out = run_ap(
        root,
        &["search", "VIP_OMEGA_99", ".", "--json"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(s_out.status.success());
    let sj: Value = serde_json::from_slice(&s_out.stdout).unwrap();
    let hits = sj["matches"].as_array().unwrap();
    assert!(
        hits.iter().any(|h| h["file"] == "src/billing.ts"
            && h["scope"].as_str().unwrap().contains("settleAccount")),
        "Expected enclosing AST symbol scope on TS search hit, got: {hits:?}"
    );
    assert!(
        hits.iter()
            .any(|h| h["file"] == "spec.docx" && h["scope"] == "DOCX"),
        "Expected .docx search hit in same pass, got: {hits:?}"
    );
}

#[test]
fn test_omni_atlas_sqlite_attach_unified_diff_and_append() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use rusqlite::Connection;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let ap_home = root.join("ap_v17_home");

    // 1. Real SQLite DB + multi-sheet .xlsx + GBK .csv + PNG header + .gz log
    let db_path = root.join("app.sqlite");
    {
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE accounts (cid TEXT PRIMARY KEY, tier TEXT, rate REAL);
             INSERT INTO accounts VALUES ('C1', 'VIP', 0.1), ('C2', 'STD', 0.2);",
        )
        .unwrap();
    }
    let xlsx_p = root.join("ledger.xlsx");
    {
        let f = fs::File::create(&xlsx_p).unwrap();
        let mut zw = ZipWriter::new(f);
        zw.start_file("xl/workbook.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            r#"<workbook><sheets><sheet name="国内" sheetId="1"/><sheet name="海外" sheetId="2"/></sheets></workbook>"#.as_bytes(),
        )
        .unwrap();
        zw.start_file("xl/sharedStrings.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            r#"<sst><si><t>cid</t></si><si><t>amt</t></si><si><t>C1</t></si><si><t>¥100.10</t></si><si><t>¥200.20</t></si></sst>"#.as_bytes(),
        )
        .unwrap();
        zw.start_file("xl/worksheets/sheet1.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            br#"<worksheet><sheetData><row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row><row r="2"><c r="A2" t="s"><v>2</v></c><c r="B2" t="s"><v>3</v></c></row></sheetData></worksheet>"#,
        )
        .unwrap();
        zw.start_file("xl/worksheets/sheet2.xml", SimpleFileOptions::default())
            .unwrap();
        zw.write_all(
            br#"<worksheet><sheetData><row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row><row r="2"><c r="A2" t="s"><v>2</v></c><c r="B2" t="s"><v>4</v></c></row></sheetData></worksheet>"#,
        )
        .unwrap();
        zw.finish().unwrap();
    }
    let (gbk_csv, _, _) = encoding_rs::GB18030.encode("科目,备注\r\n应收,正常\r\n");
    fs::write(root.join("notes_gbk.csv"), gbk_csv.as_ref()).unwrap();

    // 1a. `ap atlas map .` surfaces .xlsx sheets, GBK .csv cols, and .sqlite DB
    let map_out = run_ap(root, &["atlas", "map", "."], None, &[("AP_HOME", &ap_home)]);
    assert!(map_out.status.success());
    let map_s = String::from_utf8_lossy(&map_out.stdout);
    assert!(map_s.contains("ledger.xlsx") && map_s.contains("国内") && map_s.contains("海外"));
    assert!(map_s.contains("notes_gbk.csv") && map_s.contains("gbk csv"));
    assert!(map_s.contains("app.sqlite") && map_s.contains("sqlite3 db"));

    // 2. `ap query` joins SQLite `.sqlite` table `accounts` directly with `ledger__all` (0.1+0.2=0.3 clean!)
    let q_out = run_ap(
        root,
        &[
            "query",
            "-f",
            "app.sqlite",
            "-f",
            "ledger=ledger.xlsx",
            "--json",
            "SELECT a.tier, SUM(l.amt), 0.1 + 0.2 FROM ledger__all l JOIN accounts a ON l.cid = a.cid GROUP BY a.tier",
        ],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(q_out.status.success());
    let qj: Value = serde_json::from_slice(&q_out.stdout).unwrap();
    assert_eq!(qj["rows"][0][0], "VIP");
    assert_eq!(qj["rows"][0][1], "300.3");
    assert_eq!(qj["rows"][0][2], "0.3");

    // 3. `ap read` handles SQLite DB, PNG header dimensions, and .gz log decompression
    let r_db = run_ap(
        root,
        &["read", "app.sqlite"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(String::from_utf8_lossy(&r_db.stdout).contains("Table `accounts` (2 rows)"));

    let mut png_hdr = vec![
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R',
    ];
    png_hdr.extend_from_slice(&640u32.to_be_bytes());
    png_hdr.extend_from_slice(&480u32.to_be_bytes());
    fs::write(root.join("hero.png"), &png_hdr).unwrap();
    let r_png = run_ap(root, &["read", "hero.png"], None, &[("AP_HOME", &ap_home)]);
    assert!(String::from_utf8_lossy(&r_png.stdout).contains("PNG 640x480 px"));

    let gz_path = root.join("deploy.log.gz");
    {
        let mut enc = GzEncoder::new(fs::File::create(&gz_path).unwrap(), Compression::default());
        enc.write_all(b"DEPLOY_OK: service started cleanly\n")
            .unwrap();
        enc.finish().unwrap();
    }
    let r_gz = run_ap(
        root,
        &["read", "deploy.log.gz"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(String::from_utf8_lossy(&r_gz.stdout).contains("DEPLOY_OK"));

    // 4. `ap splice` supports `@@ APPEND:`, Unified Diff, and Tier-4 blank-line-resilient patching
    fs::write(root.join("report.md"), "# Header\n- line 1\n").unwrap();
    fs::write(
        root.join("calc.ts"),
        "export function calcTax(v: number): number {\n  const base = v * 2;\n\n  return base + 5;\n}\n",
    )
    .unwrap();

    let append_spec = "@@ APPEND: report.md\n<<<\n- line 2 appended\n>>>\n@@ FILE: calc.ts\n<<<\n  const base = v * 2;\n  return base + 5;\n===\n  const base = v * 3;\n  return base + 10;\n>>>\n";
    let ap_res = run_ap(
        root,
        &["splice", "apply", "--cwd", ".", "--stdin"],
        Some(append_spec),
        &[("AP_HOME", &ap_home)],
    );
    assert!(
        ap_res.status.success(),
        "Expected APPEND + Tier-4 blank-line match to succeed: {}",
        String::from_utf8_lossy(&ap_res.stderr)
    );
    assert!(fs::read_to_string(root.join("report.md"))
        .unwrap()
        .contains("- line 1\n- line 2 appended"));

    let udiff = "--- a/calc.ts\n+++ b/calc.ts\n@@ -1,4 +1,4 @@\n export function calcTax(v: number): number {\n-  const base = v * 3;\n+  const base = v * 4;\n   return base + 10;\n }\n";
    let ud_res = run_ap(
        root,
        &["splice", "apply", "--cwd", ".", "--stdin"],
        Some(udiff),
        &[("AP_HOME", &ap_home)],
    );
    assert!(ud_res.status.success());
    assert!(fs::read_to_string(root.join("calc.ts"))
        .unwrap()
        .contains("v * 4"));
}

#[test]
fn test_creation_contract_linker_api_fuzzer_and_ast_mutator() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let ap_home = root.join("ap_v18_home");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("test")).unwrap();

    // 1. Creation Shield: Cross-File Phantom Named Export & Call-Arity Linker (<5ms)
    fs::write(
        root.join("src/pricing.mjs"),
        "export function calcSubtotal(price, qty) {\n  return price * qty;\n}\n",
    )
    .unwrap();
    let bad_checkout = "@@ WRITE: src/checkout.mjs\n<<<\nimport { calcTotal, calcSubtotal } from './pricing.mjs';\nexport const demo = () => calcSubtotal(100) + calcTotal(1, 2);\n>>>\n";
    let ap_warn = run_ap(
        root,
        &["splice", "apply", "--cwd", ".", "--stdin"],
        Some(bad_checkout),
        &[("AP_HOME", &ap_home)],
    );
    assert!(ap_warn.status.success());
    let warn_s = String::from_utf8_lossy(&ap_warn.stdout);
    assert!(
        warn_s.contains("CONTRACT WARNING")
            && warn_s.contains("Phantom named import `calcTotal`")
            && warn_s.contains("Call arity mismatch `calcSubtotal(...1 args)`"),
        "Expected cross-file phantom import & arity warnings, got: {warn_s}"
    );
    fs::remove_file(root.join("src/checkout.mjs")).unwrap();

    // 2. Polyglot Tree-sitter AST Mutation Tester (`ap probe mutate`)
    let coupon_src = "export function eligibleForVip(spend) {\n  if (spend >= 1000) {\n    return true;\n  }\n  return false;\n}\n";
    let coupon_p = root.join("src/coupon.mjs");
    fs::write(&coupon_p, coupon_src).unwrap();
    // Weak test: checks 2000 and 100, completely missing boundary 1000 (`>=` -> `>` survives!)
    fs::write(
        root.join("test/coupon.test.mjs"),
        "import assert from 'node:assert/strict';\nimport { eligibleForVip } from '../src/coupon.mjs';\nassert.equal(eligibleForVip(2000), true);\nassert.equal(eligibleForVip(100), false);\n",
    )
    .unwrap();
    let mut_weak = run_ap(
        root,
        &[
            "probe",
            "mutate",
            "src/coupon.mjs",
            "--cwd",
            ".",
            "--test-cmd",
            "node test/coupon.test.mjs",
            "--json",
        ],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(
        !mut_weak.status.success(),
        "Weak test should let boundary mutant survive"
    );
    assert_eq!(
        fs::read_to_string(&coupon_p).unwrap(),
        coupon_src,
        "RAII guard must restore exact source"
    );
    let mw_j: Value = serde_json::from_slice(&mut_weak.stdout).unwrap();
    assert!(!mw_j["survivors"].as_array().unwrap().is_empty());

    // Add exact boundary assertion `eligibleForVip(1000) === true` -> 100% mutants killed!
    fs::write(
        root.join("test/coupon.test.mjs"),
        "import assert from 'node:assert/strict';\nimport { eligibleForVip } from '../src/coupon.mjs';\nassert.equal(eligibleForVip(1000), true);\nassert.equal(eligibleForVip(999), false);\n",
    )
    .unwrap();
    let mut_strong = run_ap(
        root,
        &[
            "probe",
            "mutate",
            "src/coupon.mjs",
            "--cwd",
            ".",
            "--test-cmd",
            "node test/coupon.test.mjs",
            "--json",
        ],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(mut_strong.status.success());

    // 3. Adversarial API & Concurrency Chaos Fuzzer (`ap probe fuzz`)
    fs::write(
        root.join("fragile_srv.mjs"),
        "import http from 'node:http';\nconst s = http.createServer((req, res) => {\n  if (req.method === 'POST' && req.url === '/api/order') {\n    let b = ''; req.on('data', c => b += c);\n    req.on('end', () => {\n      try { const j = JSON.parse(b); res.writeHead(200); res.end(JSON.stringify({ total: j.amount * 2 })); }\n      catch (e) { res.writeHead(500); res.end(String(e)); }\n    });\n    return;\n  }\n  res.writeHead(200); res.end('ok');\n});\ns.listen(Number(process.env.PORT || 0), '127.0.0.1');\n",
    )
    .unwrap();
    let fz_fail = run_ap(
        root,
        &[
            "probe",
            "fuzz",
            "--cwd",
            ".",
            "--start",
            "node fragile_srv.mjs",
            "--concurrency",
            "4",
            "--json",
        ],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(
        !fz_fail.status.success(),
        "Fuzzer must catch 500 on truncated JSON"
    );
    let fz_j: Value = serde_json::from_slice(&fz_fail.stdout).unwrap();
    assert!(!fz_j["failures"].as_array().unwrap().is_empty());

    // 4. `ap guard verify` blocks `.only(` test cheating
    fs::write(
        root.join("test/cheat.test.mjs"),
        "test.only('skip everything else', () => {});\n",
    )
    .unwrap();
    let gv = run_ap(
        root,
        &["guard", "verify", "--cwd", ".", "--all"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(!gv.status.success());
    let gv_err = format!(
        "{}{}",
        String::from_utf8_lossy(&gv.stdout),
        String::from_utf8_lossy(&gv.stderr)
    );
    assert!(gv_err.contains("no-focused-test-only"));
    fs::remove_file(root.join("test/cheat.test.mjs")).unwrap();
    fs::remove_file(root.join("fragile_srv.mjs")).unwrap();
    fs::remove_file(root.join("src/pricing.mjs")).unwrap();
    fs::write(
        root.join("package.json"),
        "{\"type\":\"module\",\"scripts\":{\"test\":\"node test/coupon.test.mjs\"}}\n",
    )
    .unwrap();

    // 5. Fused `ap guard verify --deep` passes when contracts + anti-slop + tests + AST mutation are all green
    let gv_deep = run_ap(
        root,
        &["guard", "verify", "--cwd", ".", "--deep"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(
        gv_deep.status.success(),
        "Expected guard verify --deep to pass on strong suite: {}",
        String::from_utf8_lossy(&gv_deep.stderr)
    );
}

#[test]
fn test_ui_layout_network_auditor_sec_scanner_and_fault_localizer() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let ap_home = root.join("ap_v19_home");
    fs::create_dir_all(root.join("src")).unwrap();

    // 1. Sub-5ms AI-Code OWASP Security Scanner (`ap guard sec`)
    fs::write(
        root.join("src/unsafe_api.mjs"),
        concat!(
            "export function findUser(db, id) { return db.exec(`SELECT * FROM users WHERE id = ${id}`); }\n",
            "export function renderBanner(el, title) { el.innerHTML = `<h1>${title}</h1>`; }\n",
            "export function serveFile(fs, path, req) { return fs.readFileSync(path.join('./files', req.url)); }\n"
        ),
    )
    .unwrap();
    let sec_bad = run_ap(
        root,
        &["guard", "sec", ".", "--json"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(!sec_bad.status.success());
    let sj: Value = serde_json::from_slice(&sec_bad.stdout).unwrap();
    let findings = sj["findings"].as_array().unwrap();
    assert!(findings
        .iter()
        .any(|f| f["cwe"].as_str().unwrap().contains("CWE-89")));
    assert!(findings
        .iter()
        .any(|f| f["cwe"].as_str().unwrap().contains("CWE-79")));
    assert!(findings
        .iter()
        .any(|f| f["cwe"].as_str().unwrap().contains("CWE-22")));

    // Fix all 3 vulnerabilities -> `ap guard sec` returns 0!
    fs::write(
        root.join("src/unsafe_api.mjs"),
        concat!(
            "export function findUser(db, id) { return db.prepare('SELECT * FROM users WHERE id = ?').get(id); }\n",
            "export function renderBanner(el, title) { el.innerHTML = `<h1>${escapeHtml(title)}</h1>`; }\n",
            "export function serveFile(fs, path, req) { const p = path.resolve('./files', path.basename(req.url)); if (!p.startsWith(path.resolve('./files'))) throw new Error('bad'); return fs.readFileSync(p); }\n"
        ),
    )
    .unwrap();
    let sec_ok = run_ap(root, &["guard", "sec", "."], None, &[("AP_HOME", &ap_home)]);
    assert!(sec_ok.status.success());

    // 2. Hierarchical Agentless/AutoCodeRover Fault Localizer (`ap atlas locate`)
    fs::write(
        root.join("src/auth.ts"),
        "export function verifyJwtToken(token: string) { return token.length > 10; }\n",
    )
    .unwrap();
    fs::write(
        root.join("src/shipping.ts"),
        "export function calcFreightWeight(kg: number) { return kg * 12; }\n",
    )
    .unwrap();
    fs::write(
        root.join("src/coupon.ts"),
        "export function applyCouponRetry(orderId: string, discountRate: number) {\n  // double discount applied on retry!\n  return discountRate * 2;\n}\n",
    )
    .unwrap();
    let loc = run_ap(
        root,
        &[
            "atlas",
            "locate",
            "double discount on coupon retry",
            ".",
            "--json",
        ],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(loc.status.success());
    let lj: Value = serde_json::from_slice(&loc.stdout).unwrap();
    let top = &lj["hits"].as_array().unwrap()[0];
    assert_eq!(top["file"], "src/coupon.ts");
    assert_eq!(top["symbol"], "applyCouponRetry");

    // 3. Dual-Observability Headless UI Auditor (`ap probe ui`: layout overflow + overlapped button + 404 network + PNG screenshot)
    let has_browser = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
    ]
    .iter()
    .any(|p| Path::new(p).is_file());

    if has_browser {
        fs::write(
            root.join("broken_ui.mjs"),
            concat!(
                "import http from 'node:http';\n",
                "http.createServer((req, res) => {\n",
                "  if (req.url === '/') {\n",
                "    res.writeHead(200, {'Content-Type':'text/html; charset=utf-8'});\n",
                "    res.end('<!doctype html><html><body style=\"margin:0\"><div style=\"width:2400px\">Wide</div><button id=\"buy-btn\" style=\"position:fixed;top:20px;left:20px;width:120px;height:40px\">Buy</button><div id=\"blocker\" style=\"position:fixed;top:0;left:0;width:300px;height:200px;z-index:99;background:rgba(0,0,0,0.5)\">Overlay</div><img src=\"/missing.png\"/></body></html>');\n",
                "    return;\n",
                "  }\n",
                "  res.writeHead(404); res.end('missing');\n",
                "}).listen(Number(process.env.PORT || 0), '127.0.0.1');\n"
            ),
        )
        .unwrap();
        let ui_bad = run_ap(
            root,
            &[
                "probe",
                "ui",
                "--cwd",
                ".",
                "--start",
                "node broken_ui.mjs",
                "--json",
            ],
            None,
            &[("AP_HOME", &ap_home)],
        );
        assert!(!ui_bad.status.success());
        let uj: Value = serde_json::from_slice(&ui_bad.stdout).unwrap();
        let vp0_issues = uj["viewports"][0]["issues"].to_string();
        assert!(
            vp0_issues.contains("Horizontal overflow"),
            "Expected overflow in {uj}"
        );
        assert!(
            vp0_issues.contains("Overlapped/blocked control #buy-btn"),
            "Expected overlap in {uj}"
        );
        assert!(
            !uj["network_errors"].as_array().unwrap().is_empty(),
            "Expected 404 network error: {uj}"
        );

        fs::write(
            root.join("clean_ui.mjs"),
            concat!(
                "import http from 'node:http';\n",
                "http.createServer((req, res) => {\n",
                "  res.writeHead(200, {'Content-Type':'text/html; charset=utf-8'});\n",
                "  res.end('<!doctype html><html><body style=\"margin:0;padding:16px;box-sizing:border-box\"><h1>Clean Dashboard</h1><button id=\"ok-btn\" style=\"padding:8px 16px\">Submit</button></body></html>');\n",
                "}).listen(Number(process.env.PORT || 0), '127.0.0.1');\n"
            ),
        )
        .unwrap();
        let ui_ok = run_ap(
            root,
            &[
                "probe",
                "ui",
                "--cwd",
                ".",
                "--start",
                "node clean_ui.mjs",
                "--screenshot",
                "snap.png",
            ],
            None,
            &[("AP_HOME", &ap_home)],
        );
        assert!(
            ui_ok.status.success(),
            "Clean UI should pass probe ui: {}{}",
            String::from_utf8_lossy(&ui_ok.stdout),
            String::from_utf8_lossy(&ui_ok.stderr)
        );
        assert!(
            root.join("snap.png").is_file()
                && fs::metadata(root.join("snap.png")).unwrap().len() > 200
        );
    }

    // 4. Multi-line DOM XSS (safe 1st expr, unsafe 2nd expr), Vacuous 0-test detection & Active-file receipt in `guard verify --deep`
    let vac_dir = tmp.path().join("vac_and_active");
    fs::create_dir_all(vac_dir.join("src")).unwrap();
    fs::create_dir_all(vac_dir.join("test")).unwrap();
    fs::write(
        vac_dir.join("src/view.js"),
        "export function render(el, id, user) {\n  el.innerHTML = `\n    <span data-id=\"${Number(id)}\">${user.bio}</span>\n  `;\n}\n",
    )
    .unwrap();
    let sec_ml = run_ap(
        &vac_dir,
        &["guard", "sec", "."],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert_eq!(sec_ml.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&sec_ml.stderr).contains("CWE-79 DOM XSS"),
        "Expected multi-line DOM XSS catch: {}",
        String::from_utf8_lossy(&sec_ml.stderr)
    );
    fs::remove_file(vac_dir.join("src/view.js")).unwrap();

    fs::write(
        vac_dir.join("package.json"),
        r#"{"type":"module","scripts":{"test":"node --test test/empty.test.mjs"}}"#,
    )
    .unwrap();
    fs::write(vac_dir.join("test/empty.test.mjs"), "// no tests yet\n").unwrap();
    let vac_res = run_ap(
        &vac_dir,
        &["guard", "verify", "--cwd", "."],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert_eq!(vac_res.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&vac_res.stderr).contains("[VACUOUS TEST RUN]"),
        "Expected vacuous 0-test catch: {}",
        String::from_utf8_lossy(&vac_res.stderr)
    );
    fs::remove_file(vac_dir.join("test/empty.test.mjs")).unwrap();

    // Untouched distractor `aaa_unused.mjs` must NOT derail `guard verify --deep` when `promo.mjs` is the active tested module
    fs::write(
        vac_dir.join("src/aaa_unused.mjs"),
        "export function helperA(x) { return x > 10 ? x - 1 : x + 1; }\n",
    )
    .unwrap();
    fs::write(
        vac_dir.join("src/promo.mjs"),
        "export function applyBonus(x, cap) { return x > cap ? cap + 5 : x; }\n",
    )
    .unwrap();
    fs::write(
        vac_dir.join("package.json"),
        r#"{"type":"module","scripts":{"test":"node --test test/promo.test.mjs"}}"#,
    )
    .unwrap();
    fs::write(
        vac_dir.join("test/promo.test.mjs"),
        "import test from 'node:test'; import assert from 'node:assert/strict'; import { applyBonus } from '../src/promo.mjs';\ntest('bonus', () => { assert.equal(applyBonus(15, 10), 15); assert.equal(applyBonus(10, 10), 10); assert.equal(applyBonus(5, 10), 5); });\n",
    )
    .unwrap();
    let deep_ok = run_ap(
        &vac_dir,
        &["guard", "verify", "--cwd", ".", "--deep"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    let deep_out = String::from_utf8_lossy(&deep_ok.stdout);
    assert!(
        deep_ok.status.success(),
        "Expected active-file deep verify PASS: {}{}",
        deep_out,
        String::from_utf8_lossy(&deep_ok.stderr)
    );
    assert!(deep_out.contains("[sec]") && deep_out.contains("[mutate]"));
}

#[test]
fn test_minified_asar_binary_cdp_attach_and_callchain_trace() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let ap_home = root.join("ap_v20_home");

    // 1. Minified 60KB single-line JS bundle with 2 matches far from byte 0
    let pad = "var _x=1;".repeat(2000);
    let minified = format!(
        "{pad}function handleA(u){{if(u){{return postSecretToken(u);}}}}{pad}function handleB(v){{return postSecretToken(v+1);}}{pad}"
    );
    fs::write(root.join("bundle.min.js"), &minified).unwrap();
    let ms = run_ap(
        root,
        &[
            "search",
            "postSecretToken",
            "bundle.min.js",
            "--pretty",
            "--json",
        ],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(ms.status.success());
    let mj: Value = serde_json::from_slice(&ms.stdout).unwrap();
    let hits = mj["matches"].as_array().unwrap();
    assert_eq!(hits.len(), 2, "Expected 2 matches on minified line 1: {mj}");
    assert!(hits[0]["scope"]
        .as_str()
        .unwrap()
        .contains("minified @off="));
    assert!(hits[0]["snippet"]
        .as_str()
        .unwrap()
        .contains("postSecretToken(u)"));
    let off0: usize = hits[0]["scope"]
        .as_str()
        .unwrap()
        .split("@off=")
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let mr = run_ap(
        root,
        &[
            "read",
            "bundle.min.js",
            "--offset",
            &off0.saturating_sub(30).to_string(),
            "--chars",
            "160",
            "--pretty",
        ],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(mr.status.success());
    let mr_s = String::from_utf8_lossy(&mr.stdout);
    assert!(mr_s.contains("postSecretToken(u)") && mr_s.contains('\n'));

    // 2. Real Electron `.asar` archive (Chromium Pickle header + JSON tree + payload)
    let preload_bytes = b"export function initBridge(){ return ipcBridge.invoke('status'); }\n";
    let hdr_json = format!(
        r#"{{"files":{{"preload.js":{{"size":{},"offset":"0"}}}}}}"#,
        preload_bytes.len()
    );
    let hdr_b = hdr_json.as_bytes();
    let pad_len = (4 - (hdr_b.len() % 4)) % 4;
    let json_pickle_payload = 4u32 + (hdr_b.len() + pad_len) as u32;
    let outer_pickle_payload = 4u32 + json_pickle_payload;
    let mut asar = Vec::new();
    asar.extend_from_slice(&4u32.to_le_bytes());
    asar.extend_from_slice(&outer_pickle_payload.to_le_bytes());
    asar.extend_from_slice(&json_pickle_payload.to_le_bytes());
    asar.extend_from_slice(&(hdr_b.len() as u32).to_le_bytes());
    asar.extend_from_slice(hdr_b);
    asar.resize(asar.len() + pad_len, 0);
    asar.extend_from_slice(preload_bytes);
    fs::write(root.join("app.asar"), &asar).unwrap();

    let as_list = run_ap(root, &["read", "app.asar"], None, &[("AP_HOME", &ap_home)]);
    assert!(as_list.status.success());
    assert!(String::from_utf8_lossy(&as_list.stdout).contains("preload.js"));

    let as_ent = run_ap(
        root,
        &["read", "app.asar", "--entry", "preload.js"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(as_ent.status.success());
    assert!(String::from_utf8_lossy(&as_ent.stdout).contains("ipcBridge.invoke"));

    let as_srch = run_ap(
        root,
        &["search", "ipcBridge", "app.asar"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(as_srch.status.success());
    assert!(String::from_utf8_lossy(&as_srch.stdout).contains("app.asar::preload.js"));

    // 3. Synthetic PE x86_64 binary containing both ASCII & Windows UTF-16LE wide strings
    let mut pe = vec![0u8; 0x100];
    pe[0] = b'M';
    pe[1] = b'Z';
    pe[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
    pe[0x80..0x84].copy_from_slice(b"PE\0\0");
    pe[0x84..0x86].copy_from_slice(&0x8664u16.to_le_bytes()); // x86_64
    pe.extend_from_slice(b"\0\0AsciiTelemetryUrl=https://api.example.local/v1\0\0");
    for ch in "NamedPipeEndpoint_V2".encode_utf16() {
        pe.extend_from_slice(&ch.to_le_bytes());
    }
    pe.extend_from_slice(&[0, 0]);
    fs::write(root.join("helper.dll"), &pe).unwrap();

    let bin_r = run_ap(
        root,
        &["read", "helper.dll", "--query", "NamedPipeEndpoint"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(bin_r.status.success());
    let bin_s = String::from_utf8_lossy(&bin_r.stdout);
    assert!(bin_s.contains("Windows PE (x86_64,"), "Got: {bin_s}");
    assert!(bin_s.contains("[utf16le ") && bin_s.contains("NamedPipeEndpoint_V2"));

    let bin_s2 = run_ap(
        root,
        &["search", "NamedPipeEndpoint", "helper.dll"],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(bin_s2.status.success());
    assert!(String::from_utf8_lossy(&bin_s2.stdout).contains("NamedPipeEndpoint_V2"));

    // 4. Multi-hop Tree-sitter Call-Chain Tracer (`ap atlas trace`)
    fs::create_dir_all(root.join("svc")).unwrap();
    fs::write(
        root.join("svc/fs_util.ts"),
        "import fs from 'node:fs';\nexport function readDiskAsset(p: string) { return fs.readFileSync(p, 'utf8'); }\nexport function resolveTemplatePath(n: string) { return readDiskAsset('/tpl/' + n); }\n",
    )
    .unwrap();
    fs::write(
        root.join("svc/export.ts"),
        "import { resolveTemplatePath } from './fs_util';\nexport function dispatchExport(name: string) { return resolveTemplatePath(name); }\n",
    )
    .unwrap();
    fs::write(
        root.join("svc/router.ts"),
        "import { dispatchExport } from './export';\nexport function httpRouter(req: any) { return dispatchExport(req.file); }\n",
    )
    .unwrap();
    let tr = run_ap(
        root,
        &[
            "atlas",
            "trace",
            "readDiskAsset",
            "svc",
            "--depth",
            "3",
            "--json",
        ],
        None,
        &[("AP_HOME", &ap_home)],
    );
    assert!(tr.status.success());
    let tj: Value = serde_json::from_slice(&tr.stdout).unwrap();
    let ups: Vec<&str> = tj["upstream_callers"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["symbol"].as_str())
        .collect();
    assert!(ups.contains(&"resolveTemplatePath"), "hop 1 missing: {tj}");
    assert!(ups.contains(&"dispatchExport"), "hop 2 missing: {tj}");
    assert!(ups.contains(&"httpRouter"), "hop 3 missing: {tj}");
    assert!(tj["notable_sinks"].to_string().contains("readFileSync"));

    // 5. Live Running Browser CDP Attach (`ap probe cdp --port P --list / --eval`) WITHOUT killing host browser
    let browser_opt = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
    ]
    .iter()
    .find(|p| Path::new(p).is_file())
    .copied();

    if let Some(browser) = browser_opt {
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let prof = root.join("ext_cdp_prof");
        let _ = fs::create_dir_all(&prof);
        let mut ext_browser = Command::new(browser)
            .args([
                "--headless=new",
                "--disable-gpu",
                "--disable-dev-shm-usage",
                "--disable-setuid-sandbox",
                "--no-sandbox",
                "--no-first-run",
                "--remote-debugging-address=127.0.0.1",
                &format!("--remote-debugging-port={port}"),
                "--remote-allow-origins=*",
                &format!("--user-data-dir={}", prof.display()),
                "about:blank",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let mut listed = false;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(12);
        while std::time::Instant::now() < deadline {
            let ls = run_ap(
                root,
                &[
                    "probe",
                    "cdp",
                    "--port",
                    &port.to_string(),
                    "--list",
                    "--json",
                ],
                None,
                &[("AP_HOME", &ap_home)],
            );
            if ls.status.success() {
                let lj: Value = serde_json::from_slice(&ls.stdout).unwrap();
                if lj["count"].as_u64().unwrap_or(0) >= 1 {
                    listed = true;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(120));
        }
        assert!(
            listed,
            "Expected ap probe cdp --list to discover running browser"
        );

        let ev = run_ap(
            root,
            &[
                "probe",
                "cdp",
                "--port",
                &port.to_string(),
                "--eval",
                "40 + 2",
                "--json",
            ],
            None,
            &[("AP_HOME", &ap_home)],
        );
        assert!(
            ev.status.success(),
            "ap probe cdp --eval failed: {}{}",
            String::from_utf8_lossy(&ev.stdout),
            String::from_utf8_lossy(&ev.stderr)
        );
        let ej: Value = serde_json::from_slice(&ev.stdout).unwrap();
        assert_eq!(ej["eval"], "42");
        // Verify detaching NEVER killed the external host browser process!
        assert!(
            ext_browser.try_wait().unwrap().is_none(),
            "ap probe cdp must NOT kill the attached host browser process"
        );
        let _ = ext_browser.kill();
        let _ = ext_browser.wait();
    }
}
