use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Read;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpProbeResult {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub elapsed_ms: f64,
    pub content_type: String,
    pub headers: Vec<String>,
    pub body_preview: String,
    pub error: String,
}

fn is_loopback_url(url: &str) -> bool {
    let low = url.to_ascii_lowercase();
    low.contains("://127.0.0.1") || low.contains("://localhost") || low.contains("://[::1]")
}

pub fn execute_http_probe(
    method: &str,
    url: &str,
    headers: &[String],
    body: Option<&str>,
    timeout_sec: u64,
    insecure: bool,
    follow: bool,
) -> HttpProbeResult {
    let t0 = Instant::now();
    let curl_bin = if cfg!(windows) { "curl.exe" } else { "curl" };
    let mut cmd = Command::new(curl_bin);
    cmd.args([
        "-sS",
        "-i",
        "--max-time",
        &timeout_sec.clamp(1, 120).to_string(),
        "-X",
        method,
    ]);
    if is_loopback_url(url) {
        cmd.args(["--noproxy", "127.0.0.1,localhost,::1"]);
    }
    if insecure {
        cmd.arg("-k");
    }
    if follow {
        cmd.arg("-L");
    }
    for h in headers {
        cmd.args(["-H", h]);
    }
    if let Some(b) = body.filter(|s| !s.is_empty()) {
        let has_ct = headers
            .iter()
            .any(|h| h.to_ascii_lowercase().starts_with("content-type:"));
        if !has_ct && (b.trim_start().starts_with('{') || b.trim_start().starts_with('[')) {
            cmd.args(["-H", "Content-Type: application/json"]);
        }
        cmd.args(["--data-raw", b]);
    }
    cmd.arg(url);

    let out = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return HttpProbeResult {
                method: method.to_string(),
                url: url.to_string(),
                status: 0,
                elapsed_ms: (t0.elapsed().as_secs_f64() * 1000.0).round(),
                content_type: String::new(),
                headers: Vec::new(),
                body_preview: String::new(),
                error: format!("failed to spawn {curl_bin}: {e}"),
            };
        }
    };
    let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
    if !out.status.success() && out.stdout.is_empty() {
        return HttpProbeResult {
            method: method.to_string(),
            url: url.to_string(),
            status: 0,
            elapsed_ms,
            content_type: String::new(),
            headers: Vec::new(),
            body_preview: String::new(),
            error: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        };
    }

    let raw = String::from_utf8_lossy(&out.stdout).to_string();
    // Handle redirect chains / 100 Continue by taking the LAST HTTP header block
    let mut rest = raw.as_str();
    let mut hdr_block = "";
    while rest.starts_with("HTTP/") {
        if let Some(idx) = rest.find("\r\n\r\n") {
            hdr_block = &rest[..idx];
            rest = &rest[idx + 4..];
        } else if let Some(idx) = rest.find("\n\n") {
            hdr_block = &rest[..idx];
            rest = &rest[idx + 2..];
        } else {
            break;
        }
    }
    let body_raw = rest;
    let mut status = 0u16;
    let mut ct = String::new();
    let mut hdrs = Vec::new();
    for (idx, ln) in hdr_block.lines().enumerate() {
        let t = ln.trim();
        if idx == 0 {
            status = t
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);
        } else if !t.is_empty() {
            if let Some((k, v)) = t.split_once(':') {
                if k.eq_ignore_ascii_case("content-type") {
                    ct = v.trim().to_string();
                }
            }
            if hdrs.len() < 24 {
                hdrs.push(t.to_string());
            }
        }
    }

    let preview: String = if body_raw.len() > 2400 {
        format!(
            "{}...\n[truncated: {}B total]",
            body_raw.chars().take(2400).collect::<String>(),
            body_raw.len()
        )
    } else {
        body_raw.to_string()
    };

    HttpProbeResult {
        method: method.to_string(),
        url: url.to_string(),
        status,
        elapsed_ms,
        content_type: ct,
        headers: hdrs,
        body_preview: preview,
        error: String::new(),
    }
}

fn parse_json_http_job(v: &Value) -> Option<(String, String, Vec<String>, Option<String>)> {
    let obj = v.as_object()?;
    let u = obj.get("url").and_then(Value::as_str)?.trim();
    if !(u.starts_with("http://") || u.starts_with("https://")) {
        return None;
    }
    let m = obj
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("GET")
        .trim()
        .to_ascii_uppercase();
    let mut hdrs = Vec::new();
    if let Some(arr) = obj.get("headers").and_then(Value::as_array) {
        for item in arr {
            if let Some(s) = item.as_str() {
                hdrs.push(s.to_string());
            }
        }
    } else if let Some(map) = obj.get("headers").and_then(Value::as_object) {
        for (k, val) in map {
            if let Some(s) = val.as_str() {
                hdrs.push(format!("{k}: {s}"));
            } else {
                hdrs.push(format!("{k}: {val}"));
            }
        }
    }
    let body = obj.get("body").and_then(|b| match b {
        Value::Null => None,
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::String(_) => None,
        other => Some(other.to_string()),
    });
    Some((m, u.to_string(), hdrs, body))
}

#[allow(clippy::too_many_arguments)]
pub fn run_http_cli(
    url_opt: Option<&str>,
    method: &str,
    headers: &[String],
    body: Option<&str>,
    timeout_sec: u64,
    insecure: bool,
    follow: bool,
    batch_stdin: bool,
    as_json: bool,
) -> i32 {
    let mut jobs: Vec<(String, String, Vec<String>, Option<String>)> = Vec::new();
    if let Some(u) = url_opt.filter(|s| !s.trim().is_empty()) {
        jobs.push((
            method.to_ascii_uppercase(),
            u.to_string(),
            Vec::new(),
            body.map(str::to_string),
        ));
    }
    if batch_stdin {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        let trimmed = buf.trim();
        if trimmed.starts_with('[') {
            if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(trimmed) {
                for it in items {
                    if let Some(job) = parse_json_http_job(&it) {
                        jobs.push(job);
                    }
                }
            }
        } else {
            for raw_ln in buf.lines() {
                let ln = raw_ln.trim();
                if ln.is_empty() || ln.starts_with('#') {
                    continue;
                }
                if ln.starts_with('{') {
                    if let Ok(v) = serde_json::from_str::<Value>(ln) {
                        if let Some(job) = parse_json_http_job(&v) {
                            jobs.push(job);
                            continue;
                        }
                    }
                }
                if ln.starts_with("http://") || ln.starts_with("https://") {
                    jobs.push(("GET".to_string(), ln.to_string(), Vec::new(), None));
                    continue;
                }
                let mut p = ln.splitn(3, char::is_whitespace);
                let m = p.next().unwrap_or("GET").to_ascii_uppercase();
                let u = p.next().unwrap_or("").trim();
                let b = p
                    .next()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                if u.starts_with("http://") || u.starts_with("https://") {
                    jobs.push((m, u.to_string(), Vec::new(), b));
                }
            }
        }
    }
    if jobs.is_empty() {
        eprintln!(
            "[probe http] Provide a URL or pass `--batch-stdin` with `METHOD URL [BODY]` lines or JSON/JSONL request objects."
        );
        return 2;
    }

    let mut results = Vec::new();
    for (m, u, extra_hdrs, b) in jobs.into_iter().take(25) {
        let mut merged_hdrs = headers.to_vec();
        merged_hdrs.extend(extra_hdrs);
        results.push(execute_http_probe(
            &m,
            &u,
            &merged_hdrs,
            b.as_deref(),
            timeout_sec,
            insecure,
            follow,
        ));
    }

    let any_err = results.iter().any(|r| !r.error.is_empty() || r.status == 0);
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": !any_err,
                "count": results.len(),
                "results": results,
            }))
            .unwrap_or_default()
        );
    } else if results.len() == 1 {
        let r = &results[0];
        if !r.error.is_empty() {
            eprintln!("[probe http] FAIL {} {} — {}", r.method, r.url, r.error);
        } else {
            println!(
                "[probe http] {} {} -> HTTP {} ({}ms | {})",
                r.method, r.url, r.status, r.elapsed_ms, r.content_type
            );
            for h in r.headers.iter().take(10) {
                println!("  {h}");
            }
            if !r.body_preview.is_empty() {
                println!("\n{}", r.body_preview);
            }
        }
    } else {
        println!("[probe http] Batch {} request(s):", results.len());
        for (i, r) in results.iter().enumerate() {
            if !r.error.is_empty() {
                println!("  {}. {} {} -> ERR ({})", i + 1, r.method, r.url, r.error);
            } else {
                let snip: String = r
                    .body_preview
                    .replace(['\r', '\n'], " ")
                    .chars()
                    .take(120)
                    .collect();
                println!(
                    "  {}. {} {} -> HTTP {} ({}ms) — `{}`",
                    i + 1,
                    r.method,
                    r.url,
                    r.status,
                    r.elapsed_ms,
                    snip
                );
            }
        }
    }
    if any_err {
        1
    } else {
        0
    }
}
