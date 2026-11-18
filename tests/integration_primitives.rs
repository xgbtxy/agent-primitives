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
