use crate::install::all_embedded_skills;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::Instant;

struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn range_u32(&mut self, low: u32, high_incl: u32) -> u32 {
        let span = (high_incl - low + 1) as u64;
        low + (self.next_u64() % span) as u32
    }
}

fn bench_sieve(tmp: &Path) -> Vec<Value> {
    let store = tmp.join("sieve_store");
    let _ = fs::create_dir_all(&store);
    let mut rng = SplitMix64(20261115);
    let mut cases = Vec::new();

    // Case 1: 2,500-line CI/pytest output with 2 buried failures
    let mut ci_lines = Vec::new();
    for i in 0..2500usize {
        if i == 914 {
            ci_lines.extend([
                "FAIL tests/exporter/test_otlp_retry.py::test_429_jitter".to_string(),
                "Traceback (most recent call last):".to_string(),
                "  File \"tests/exporter/test_otlp_retry.py\", line 142, in test_429_jitter".to_string(),
                "    assert backoff_ms <= 5000".to_string(),
                "AssertionError: assert 8192 <= 5000".to_string(),
            ]);
        } else if i == 1880 {
            ci_lines.extend([
                "ERROR tests/sdk/test_batch_processor.py::test_shutdown_flush".to_string(),
                "RuntimeError: flush deadline exceeded after 3000ms".to_string(),
            ]);
        } else {
            ci_lines.push(format!(
                "2026-11-18T04:12:{:02}Z INFO  runner.worker test_case_{} PASSED [0.01s]",
                i % 60,
                i % 40
            ));
        }
    }
    let raw_ci = format!("{}\n", ci_lines.join("\n"));
    let t0 = Instant::now();
    let r_ci = ap_sieve::compress_stream_in(&store, &raw_ci, "pytest-ci", 1, 0.0);
    let lat_ci = (t0.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
    cases.push(json!({
        "workload": "sieve: 2,500-line CI/pytest log (Coding Agent)",
        "raw_bytes": r_ci.raw_bytes,
        "out_bytes": r_ci.digest_bytes,
        "raw_tokens_est": r_ci.raw_tokens_est,
        "out_tokens_est": r_ci.digest_tokens_est,
        "reduction_pct": r_ci.saved_pct,
        "latency_ms": lat_ci,
        "signal_preserved": r_ci.rendered.contains("AssertionError: assert 8192 <= 5000")
            && r_ci.rendered.contains("flush deadline exceeded"),
    }));

    // Case 2: 400-item JSON API payload
    let items: Vec<Value> = (0..400)
        .map(|k| {
            json!({
                "id": format!("evt_{k:04}"),
                "owner": format!("user_{}", rng.range_u32(1, 30)),
                "amount_usd": (rng.range_u32(1000, 95000) as f64) / 100.0,
                "summary": "Monthly SaaS invoice and automated reconciliation record with audit trail metadata",
                "tags": ["finance", "ops", "q4-2026"],
            })
        })
        .collect();
    let raw_json = serde_json::to_string_pretty(&json!({
        "status": "partial_warning",
        "warnings": ["rate_limit_remaining=2"],
        "items": items,
    }))
    .unwrap();
    let t0 = Instant::now();
    let r_js = ap_sieve::compress_stream_in(&store, &raw_json, "api-invoices", 0, 0.0);
    let lat_js = (t0.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
    cases.push(json!({
        "workload": "sieve: 400-record JSON API dump (Personal/Coding Agent)",
        "raw_bytes": r_js.raw_bytes,
        "out_bytes": r_js.digest_bytes,
        "raw_tokens_est": r_js.raw_tokens_est,
        "out_tokens_est": r_js.digest_tokens_est,
        "reduction_pct": r_js.saved_pct,
        "latency_ms": lat_js,
        "signal_preserved": r_js.rendered.contains("rate_limit_remaining=2") && r_js.rendered.contains("Array[400]"),
    }));

    // Case 3: 300-record JSONL event stream + bundled SQLite FTS5 search
    let mut jl_lines = Vec::new();
    for k in 0..300usize {
        if k == 177 {
            jl_lines.push(json!({"seq": k, "event": "rpc", "status": 502, "error": "upstream_gateway_timeout"}).to_string());
        } else {
            jl_lines.push(json!({"seq": k, "event": "heartbeat", "status": 200, "region": "ap-east"}).to_string());
        }
    }
    let raw_jl = format!("{}\n", jl_lines.join("\n"));
    let t0 = Instant::now();
    let r_jl = ap_sieve::compress_stream_in(&store, &raw_jl, "events-jsonl", 0, 0.0);
    let lat_jl = (t0.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
    let (hits, fts_ms) = ap_sieve::search_archives(&store, "upstream_gateway_timeout", 5);
    cases.push(json!({
        "workload": "sieve: 300-record JSONL stream + bundled FTS5 search",
        "raw_bytes": r_jl.raw_bytes,
        "out_bytes": r_jl.digest_bytes,
        "raw_tokens_est": r_jl.raw_tokens_est,
        "out_tokens_est": r_jl.digest_tokens_est,
        "reduction_pct": r_jl.saved_pct,
        "latency_ms": lat_jl,
        "fts5_search_latency_ms": fts_ms,
        "signal_preserved": r_jl.rendered.contains("upstream_gateway_timeout") && !hits.is_empty(),
    }));

    // Case 4: Noisy HTML page w/ JS+CSS bloat & table
    let script_noise = format!("<script>{}</script>", "const chunk = '0123456789abcdef'.repeat(40);\n".repeat(90));
    let style_noise = format!("<style>{}</style>", ".cls-item { display:flex; margin:4px; color:#333; }\n".repeat(400));
    let raw_html = format!(
        "<!doctype html><html><head><title>Vendor Release Notes v4.8</title>{style_noise}{script_noise}</head>\
         <body><h1>Breaking Changes in v4.8</h1><p>Webhook HMAC header renamed to X-Sig-256 on 2026-11-01.</p>\
         <table><tr><th>Endpoint</th><th>Status</th></tr><tr><td>/v2/webhooks</td><td>Required</td></tr></table>\
         {}</body></html>",
        "<div><a href='/archive'>Archive Link</a></div>".repeat(500)
    );
    let t0 = Instant::now();
    let r_ht = ap_sieve::compress_stream_in(&store, &raw_html, "web-scrape", 0, 0.0);
    let lat_ht = (t0.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
    cases.push(json!({
        "workload": "sieve: Noisy HTML page w/ JS+CSS + <table> (Personal Agent)",
        "raw_bytes": r_ht.raw_bytes,
        "out_bytes": r_ht.digest_bytes,
        "raw_tokens_est": r_ht.raw_tokens_est,
        "out_tokens_est": r_ht.digest_tokens_est,
        "reduction_pct": r_ht.saved_pct,
        "latency_ms": lat_ht,
        "signal_preserved": r_ht.rendered.contains("X-Sig-256") && r_ht.rendered.contains("/v2/webhooks"),
    }));

    // Case 5: Honest Negative Control (<1.2 KB short output passthrough)
    let short_txt = "On branch main\nChanges not staged for commit:\n  modified: src/main.rs\n";
    let t0 = Instant::now();
    let r_short = ap_sieve::compress_stream_in(&store, short_txt, "git-status", 0, 0.0);
    let lat_short = (t0.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
    cases.push(json!({
        "workload": "sieve: Negative control (<1.2 KB short output passthrough)",
        "raw_bytes": r_short.raw_bytes,
        "out_bytes": r_short.digest_bytes,
        "raw_tokens_est": r_short.raw_tokens_est,
        "out_tokens_est": r_short.digest_tokens_est,
        "reduction_pct": 0.0,
        "latency_ms": lat_short,
        "signal_preserved": true,
    }));
    cases
}

fn bench_recall(tmp: &Path) -> Value {
    let db = tmp.join("recall_bench.db");
    let conn = ap_recall::open_vault(&db).unwrap();
    for i in 0..100 {
        let _ = ap_recall::add_rule(
            &conn,
            &format!("pkg/service_{i}/*,endpoint_{i}"),
            &format!("Service {i} requires idempotency-key header on retries and strict UTC timestamps in payload."),
            "gotcha",
        );
    }
    for i in 0..100 {
        let _ = ap_recall::add_rule(
            &conn,
            &format!("client_{i},合同_{i},ops_{i}"),
            &format!("客户 {i} 周报需附带里程碑验收哈希并抄送项目归档邮箱 archive_{i}@example.org。"),
            "workflow",
        );
    }
    let stats = ap_recall::compute_stats(&conn).unwrap();
    let monolith_tokens = stats.monolithic_tokens_if_always_loaded.max(1);
    let mut latencies = Vec::new();
    let mut top1_correct = 0usize;
    let mut injected_toks = Vec::new();

    for idx in 0..10 {
        let target = idx * 9;
        let (hits, ms) = ap_recall::match_rules(
            &conn,
            &format!("retry bug on endpoint_{target}"),
            &[format!("pkg/service_{target}/handler.rs")],
            2,
        )
        .unwrap();
        latencies.push(ms);
        if hits.first().map(|h| h.rule.contains(&format!("Service {target} "))).unwrap_or(false) {
            top1_correct += 1;
        }
        let rendered = hits.iter().map(|h| h.rule.as_str()).collect::<Vec<_>>().join("\n");
        injected_toks.push(rendered.len().div_ceil(4));
    }
    for idx in 0..10 {
        let target = idx * 9 + 3;
        let (hits, ms) = ap_recall::match_rules(
            &conn,
            &format!("准备发送客户 {target} 的合同_{target}周报"),
            &[],
            2,
        )
        .unwrap();
        latencies.push(ms);
        if hits.first().map(|h| h.rule.contains(&format!("客户 {target} "))).unwrap_or(false) {
            top1_correct += 1;
        }
        let rendered = hits.iter().map(|h| h.rule.as_str()).collect::<Vec<_>>().join("\n");
        injected_toks.push(rendered.len().div_ceil(4));
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let avg_tok = (injected_toks.iter().sum::<usize>() as f64 / injected_toks.len() as f64 * 10.0).round() / 10.0;
    json!({
        "vault_rules": 200,
        "monolithic_agents_md_tokens": monolith_tokens,
        "avg_injected_tokens_top2": avg_tok,
        "token_reduction_pct": ((1.0 - avg_tok / monolith_tokens as f64) * 1000.0).round() / 10.0,
        "top1_recall_accuracy": format!("{top1_correct}/20 ({}%)", top1_correct * 5),
        "median_query_latency_ms": (latencies[latencies.len() / 2] * 100.0).round() / 100.0,
    })
}

fn bench_atlas_probe_splice(tmp: &Path) -> (Value, Value, Value) {
    // 1. Atlas: 64-module Rust & Python SDK corpus
    let pkg_root = tmp.join("sdk_corpus");
    let subpkgs = ["core", "exporter", "propagator", "metrics", "trace", "resources", "internal", "config"];
    for sp in subpkgs {
        let d = pkg_root.join(sp);
        let _ = fs::create_dir_all(&d);
        let _ = fs::write(d.join("mod.rs"), format!("//! Subpackage {sp}\n"));
    }
    let mut rng = SplitMix64(20261115);
    for idx in 0..56usize {
        let sp = subpkgs[idx % subpkgs.len()];
        let mod_p = pkg_root.join(sp).join(format!("module_{idx:02}.rs"));
        let mut lines = vec![
            "use crate::core;".to_string(),
            format!("pub struct Processor{idx:02} {{ pub timeout_ms: u64 }}"),
            format!("impl Processor{idx:02} {{"),
            "    pub fn start(&self) -> bool { true }".to_string(),
            "    pub fn export_batch(&self, items: &[String]) -> usize { items.len() }".to_string(),
            "}".to_string(),
            format!(
                "pub fn build_pipeline_{idx:02}() -> Processor{idx:02} {{ Processor{idx:02} {{ timeout_ms: {} }} }}",
                rng.range_u32(100, 5000)
            ),
        ];
        for h in 0..32 {
            lines.push(format!(
                "// Internal invariant line {h}: deterministic batch flush & backoff window accounting."
            ));
        }
        let _ = fs::write(mod_p, format!("{}\n", lines.join("\n")));
    }
    let atlas_data = ap_atlas::scan_workspace(&pkg_root, "", 16, 90);
    let atlas_md = ap_atlas::render_atlas_md(&atlas_data);
    let atlas_tok = atlas_md.len().div_ceil(4) as u64;
    let raw_tok = atlas_data.raw_tokens_if_read_all.max(1);
    let atlas_res = json!({
        "scanned_target": "sdk_corpus (64 Rust modules via Tree-sitter)",
        "scanned_files": atlas_data.scanned_files,
        "raw_corpus_tokens_est": raw_tok,
        "atlas_map_tokens_est": atlas_tok,
        "orientation_reduction_pct": ((1.0 - atlas_tok as f64 / raw_tok as f64) * 1000.0).round() / 10.0,
        "scan_latency_ms": atlas_data.elapsed_ms,
    });

    // 2. Probe: 24-module TypeScript repo (9 active, 15 dead distractors)
    let ts_repo = tmp.join("ts_corpus");
    let _ = fs::create_dir_all(ts_repo.join("src"));
    for k in 0..8usize {
        let imp = if k < 7 {
            format!("import {{ step{} }} from \"./mod_{}.ts\";\n", k + 1, k + 1)
        } else {
            String::new()
        };
        let _ = fs::write(
            ts_repo.join("src").join(format!("mod_{k}.ts")),
            format!("{imp}export function step{k}(v: number): number {{\n  return v + {k};\n}}\n"),
        );
    }
    let _ = fs::write(ts_repo.join("src").join("index.ts"), "export { step0 } from \"./mod_0.ts\";\n");
    for d in 0..15usize {
        let _ = fs::write(
            ts_repo.join("src").join(format!("dead_distractor_{d:02}.ts")),
            format!("export function deadFn{d}(x: number): number {{\n  return x * {d};\n}}\n"),
        );
    }
    let sl_payload = ap_probe::slice::compute_slice(&ts_repo, &["src/index.ts".to_string()]);
    let probe_res = json!({
        "total_code_files": sl_payload.total_code_files,
        "reachable_files": sl_payload.reachable_count,
        "pruned_dead_files": sl_payload.pruned_unreferenced_count,
        "dead_code_pruned_pct": ((sl_payload.pruned_unreferenced_count as f64 / sl_payload.total_code_files.max(1) as f64) * 1000.0).round() / 10.0,
        "slice_latency_ms": sl_payload.elapsed_ms,
    });

    // 3. Splice: Tree-sitter symbol view + zero-escaping CRLF-preserving atomic apply + check
    let sp_repo = tmp.join("splice_corpus");
    let _ = fs::create_dir_all(sp_repo.join("src"));
    let mut rs_lines = vec!["pub struct BillingService;".to_string(), "impl BillingService {".to_string()];
    for idx in 0..28usize {
        rs_lines.push(format!("    pub fn helper_step_{idx:02}(&self, cents: i64) -> i64 {{"));
        rs_lines.push(format!("        // UTF-8 accounting invariant #{idx:02}: 账单校验_{idx:02}"));
        rs_lines.push(format!("        cents + {idx}"));
        rs_lines.push("    }".to_string());
    }
    rs_lines.extend([
        "    pub fn commit_invoice(&self, subtotal_cents: i64, tax_bps: i64) -> i64 {".to_string(),
        "        let tax = (subtotal_cents * tax_bps) / 10_000;".to_string(),
        "        subtotal_cents + tax".to_string(),
        "    }".to_string(),
        "}".to_string(),
    ]);
    let svc_file = sp_repo.join("src").join("service.rs");
    let svc_crlf = format!("{}\r\n", rs_lines.join("\r\n"));
    let _ = fs::write(&svc_file, svc_crlf.as_bytes());

    let full_tokens = svc_crlf.len().div_ceil(4);
    let t_v = Instant::now();
    let v_res = ap_splice::view_single_file(
        &svc_file,
        &["BillingService.commit_invoice".to_string()],
        "",
        "",
    )
    .unwrap();
    let view_ms = (t_v.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
    let sym_tokens = v_res.blocks[0].numbered_source.len().div_ceil(4);

    let patch_dsl = "@@ SYMBOL: src/service.rs :: BillingService.commit_invoice\n<<<\npub fn commit_invoice(&self, subtotal_cents: i64, tax_bps: i64) -> i64 {\n    let tax = (subtotal_cents * tax_bps + 5_000) / 10_000;\n    subtotal_cents + tax\n}\n>>>\n";
    let t_a = Instant::now();
    let rc = ap_splice::apply_spec_silent(&sp_repo, patch_dsl);
    let apply_ms = (t_a.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
    let crlf_preserved = fs::read(&svc_file)
        .map(|b| b.windows(2).any(|w| w == b"\r\n"))
        .unwrap_or(false);
    let chk = ap_splice::check::collect_check_payload(&sp_repo, false, "");

    let splice_res = json!({
        "full_module_tokens_est": full_tokens,
        "symbol_view_tokens_est": sym_tokens,
        "symbol_read_reduction_pct": ((1.0 - sym_tokens as f64 / full_tokens.max(1) as f64) * 1000.0).round() / 10.0,
        "view_latency_ms": view_ms,
        "zero_escaping_atomic_apply_ok": rc == 0,
        "atomic_apply_latency_ms": apply_ms,
        "crlf_preserved": crlf_preserved,
        "repo_ast_check_latency_ms": chk.elapsed_ms,
    });

    (atlas_res, probe_res, splice_res)
}

pub fn run_bench(out_path: &Path) -> i32 {
    let tmp = std::env::temp_dir().join(format!("ap_bench_{}", std::process::id()));
    let _ = fs::create_dir_all(&tmp);

    let sieve_res = bench_sieve(&tmp);
    let recall_res = bench_recall(&tmp);
    let (atlas_res, probe_res, splice_res) = bench_atlas_probe_splice(&tmp);
    let _ = fs::remove_dir_all(&tmp);

    let mut idle_tokens = serde_json::Map::new();
    for (name, txt) in all_embedded_skills() {
        let parts: Vec<&str> = txt.split("---").collect();
        let fm = if parts.len() >= 3 {
            parts[1].trim()
        } else {
            &txt[..txt.len().min(300)]
        };
        idle_tokens.insert((*name).to_string(), json!(fm.len().div_ceil(4)));
    }

    let report = json!({
        "generated_at": ap_baton::now_iso_utc(),
        "engine": format!("agent-primitives v{} (Rust, Tree-sitter, bundled SQLite FTS5)", env!("CARGO_PKG_VERSION")),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "idle_frontmatter_tokens_per_folder": idle_tokens,
        "sieve_benchmarks": sieve_res,
        "recall_benchmark_200_rules": recall_res,
        "atlas_real_scan_benchmark": atlas_res,
        "probe_microscope_benchmark": probe_res,
        "splice_surgeon_benchmark": splice_res,
    });

    let pretty = serde_json::to_string_pretty(&report).unwrap_or_default();
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }
    let _ = fs::write(out_path, &pretty);
    println!("{pretty}");
    0
}
