use clap::{Parser, Subcommand};
use regex::Regex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub const DEFAULT_MAX_STORE_BYTES: u64 = 50 * 1024 * 1024;
pub const PASSTHROUGH_BYTES: usize = 1200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SieveResult {
    pub id: String,
    pub mode: String,
    pub exit_code: i32,
    pub raw_bytes: usize,
    pub digest_bytes: usize,
    #[serde(default)]
    pub saved_pct: f64,
    pub raw_tokens_est: usize,
    pub digest_tokens_est: usize,
    pub rendered: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub label: String,
    pub snippet: String,
}

#[derive(Parser, Debug)]
#[command(name = "sieve", about = "Lossless output compressor & SQLite FTS5 flight recorder")]
struct Cli {
    #[command(subcommand)]
    subcmd: SubCmd,
}

#[derive(Subcommand, Debug)]
enum SubCmd {
    /// Run a command and compress its output losslessly
    Run {
        #[arg(long)]
        json: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    /// Read stdin and compress losslessly
    Pipe {
        #[arg(long, default_value = "stdin")]
        label: String,
        #[arg(long)]
        json: bool,
    },
    /// Retrieve exact raw slices by archive id
    Peek {
        id: String,
        #[arg(long)]
        lines: Option<String>,
        #[arg(long)]
        grep: Option<String>,
        #[arg(short = 'C', long, default_value_t = 3)]
        context: usize,
        #[arg(long, default_value_t = 10)]
        max_hits: usize,
        #[arg(long)]
        json_path: Option<String>,
    },
    /// Sub-millisecond SQLite FTS5 full-text search across all archived outputs
    Search {
        query: String,
        #[arg(long, default_value_t = 8)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
}

fn log_regexes() -> &'static (Regex, Regex, Regex) {
    static RE: OnceLock<(Regex, Regex, Regex)> = OnceLock::new();
    RE.get_or_init(|| {
        let err_re = Regex::new(
            r#"(?i)(traceback \(most recent call last\)|^\s*file ".*", line \d+|\b(?:error|exception|failed|failure|fatal|panic|assertionerror|typeerror|valueerror|keyerror|syntaxerror|segfault)\b|^E\s+|^FAIL[:\s]|^ERROR[:\s])"#,
        )
        .unwrap();
        let ts_re = Regex::new(r"\b\d{2,4}[-/:]\d{2}[-/:]\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?\b").unwrap();
        let num_re = Regex::new(r"\d+").unwrap();
        (err_re, ts_re, num_re)
    })
}

fn trunc_chars(s: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max_chars {
            break;
        }
        out.push(ch);
    }
    out
}

fn est_tokens(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.len().div_ceil(4).max(1)
    }
}

pub fn store_dir(cwd: &Path) -> PathBuf {
    let base = match std::env::var("SIEVE_HOME") {
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v),
        _ => cwd.join(".sieve"),
    };
    let _ = fs::create_dir_all(&base);
    let gi = base.join(".gitignore");
    if !gi.exists() {
        let _ = fs::write(&gi, "*\n");
    }
    base
}

fn index_archive(store: &Path, item_id: &str, label: &str, raw_text: &str) {
    let db_path = store.join("index.sqlite");
    if let Ok(conn) = Connection::open(db_path) {
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        let _ = conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS chunks USING fts5(id UNINDEXED, label, content);",
        );
        let _ = conn.execute("DELETE FROM chunks WHERE id = ?1", params![item_id]);
        let _ = conn.execute(
            "INSERT INTO chunks(id, label, content) VALUES (?1, ?2, ?3)",
            params![item_id, label, raw_text],
        );
    }
}

fn prune_ring_buffer(store: &Path, max_bytes: u64) {
    let Ok(rd) = fs::read_dir(store) else { return };
    let mut raw_files: Vec<(u64, u64, PathBuf)> = Vec::new();
    let mut total = 0u64;
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("raw") {
            if let Ok(meta) = entry.metadata() {
                let sz = meta.len();
                let mtime = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                total += sz;
                raw_files.push((mtime, sz, p));
            }
        }
    }
    raw_files.sort_by_key(|x| x.0);
    let mut idx = 0usize;
    while total > max_bytes && raw_files.len().saturating_sub(idx) > 1 {
        let (_, sz, ref victim) = raw_files[idx];
        let _ = fs::remove_file(victim);
        let _ = fs::remove_file(victim.with_extension("meta.json"));
        total = total.saturating_sub(sz);
        idx += 1;
    }
}

fn archive_stream(store: &Path, raw_text: &str, label: &str, exit_code: i32, elapsed_ms: f64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_text.as_bytes());
    let id = format!("{:x}", hasher.finalize())[..10].to_string();
    let raw_path = store.join(format!("{id}.raw"));
    let meta_path = store.join(format!("{id}.meta.json"));
    let _ = fs::write(&raw_path, raw_text);
    let raw_lines = raw_text.lines().count();
    let meta = json!({
        "id": id,
        "label": label,
        "exit_code": exit_code,
        "elapsed_ms": (elapsed_ms * 10.0).round() / 10.0,
        "raw_bytes": raw_text.len(),
        "raw_lines": raw_lines,
        "created_epoch": SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
    });
    if let Ok(ms) = serde_json::to_string_pretty(&meta) {
        let _ = fs::write(&meta_path, ms);
    }
    index_archive(store, &id, label, raw_text);
    prune_ring_buffer(store, DEFAULT_MAX_STORE_BYTES);
    id
}

fn format_pipe_table(rows: &[Vec<String>], max_rows: usize) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let sample_n = rows.len().min(max_rows + 2);
    let ncols = rows[..sample_n].iter().map(|r| r.len()).max().unwrap_or(0).min(10);
    if ncols == 0 {
        return String::new();
    }
    let fmt_row = |r: &[String]| -> String {
        let cells: Vec<String> = (0..ncols)
            .map(|i| {
                let raw = r.get(i).map(String::as_str).unwrap_or("");
                trunc_chars(raw.replace('|', "\\|").trim(), 48)
            })
            .collect();
        format!("| {} |", cells.join(" | "))
    };
    let mut out = vec![fmt_row(&rows[0]), format!("| {} |", vec!["---"; ncols].join(" | "))];
    let body = &rows[1..];
    if body.len() <= max_rows {
        for r in body {
            out.push(fmt_row(r));
        }
    } else {
        let head_n = max_rows.saturating_sub(2).max(1);
        for r in &body[..head_n] {
            out.push(fmt_row(r));
        }
        let empty_tail = if ncols > 1 {
            format!(" | {}", vec![""; ncols - 1].join(" | "))
        } else {
            String::new()
        };
        out.push(format!(
            "| ... ({} middle rows omitted){empty_tail} |",
            body.len() - max_rows
        ));
        for r in &body[body.len() - 2..] {
            out.push(fmt_row(r));
        }
    }
    out.join("\n")
}

fn summarize_json(val: &Value) -> String {
    let mut lines = Vec::new();
    match val {
        Value::Object(map) => {
            let keys: Vec<&String> = map.keys().collect();
            let top_preview: Vec<&str> = keys.iter().take(15).map(|s| s.as_str()).collect();
            lines.push(format!(
                "JSON Object ({} top-level keys): {}",
                keys.len(),
                top_preview.join(", ")
            ));
            for k in keys.iter().take(10) {
                let v = &map[*k];
                match v {
                    Value::Array(arr) => {
                        let schema = match arr.first() {
                            Some(Value::Object(o)) => format!(
                                "{:?}",
                                o.keys().take(8).collect::<Vec<_>>()
                            ),
                            Some(other) => value_type_name(other).to_string(),
                            None => "empty".to_string(),
                        };
                        lines.push(format!("  - {k}: Array[{}] (item schema: {schema})", arr.len()));
                    }
                    Value::Object(sub) => {
                        let sk: Vec<&str> = sub.keys().take(8).map(|s| s.as_str()).collect();
                        lines.push(format!("  - {k}: Object({} keys: {})", sub.len(), sk.join(", ")));
                    }
                    _ => {
                        let s = serde_json::to_string(v).unwrap_or_default();
                        lines.push(format!("  - {k}: {}", trunc_chars(&s, 100)));
                    }
                }
            }
            for alert_k in ["error", "errors", "message", "status", "detail", "warnings"] {
                if let Some(av) = map.get(alert_k) {
                    if !av.is_null() && av != &Value::Bool(false) && av != &json!("") && av != &json!([]) {
                        let s = serde_json::to_string(av).unwrap_or_default();
                        lines.push(format!("  ! {alert_k}: {}", trunc_chars(&s, 240)));
                    }
                }
            }
        }
        Value::Array(arr) => {
            let schema = match arr.first() {
                Some(Value::Object(o)) => format!("{:?}", o.keys().take(10).collect::<Vec<_>>()),
                Some(other) => value_type_name(other).to_string(),
                None => "empty".to_string(),
            };
            lines.push(format!("JSON Array ({} items, item schema: {schema})", arr.len()));
            if let Some(first) = arr.first() {
                let s = serde_json::to_string(first).unwrap_or_default();
                lines.push(format!("  - [0] preview: {}", trunc_chars(&s, 260)));
            }
        }
        _ => {
            let s = serde_json::to_string(val).unwrap_or_default();
            lines.push(format!("JSON Scalar: {}", trunc_chars(&s, 200)));
        }
    }
    lines.join("\n")
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn try_summarize_jsonl(raw: &str) -> Option<String> {
    let all_lines: Vec<&str> = raw.lines().collect();
    let non_empty: Vec<(usize, &str)> = all_lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let t = l.trim();
            if t.is_empty() {
                None
            } else {
                Some((i + 1, t))
            }
        })
        .collect();
    if non_empty.len() < 2 {
        return None;
    }
    let sample_n = non_empty.len().min(40);
    let mut valid = 0usize;
    for &(_, ln) in &non_empty[..sample_n] {
        if ln.starts_with('{') && ln.ends_with('}') {
            if let Ok(Value::Object(_)) = serde_json::from_str::<Value>(ln) {
                valid += 1;
            }
        }
    }
    if valid < ((sample_n * 3) / 4).max(2) {
        return None;
    }

    let mut key_counts: HashMap<String, usize> = HashMap::new();
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut err_records: Vec<String> = Vec::new();
    let mut parsed_count = 0usize;
    let mut first_preview = String::new();
    let mut last_preview = String::new();

    for &(lineno, ln) in &non_empty {
        let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(ln) else {
            continue;
        };
        parsed_count += 1;
        if first_preview.is_empty() {
            first_preview = trunc_chars(ln, 200);
        }
        last_preview = trunc_chars(ln, 200);
        for k in obj.keys() {
            *key_counts.entry(k.clone()).or_insert(0) += 1;
        }
        for tk in ["type", "event", "kind", "level"] {
            if let Some(tv) = obj.get(tk) {
                let tag = match tv {
                    Value::String(s) => Some(format!("{tk}={s}")),
                    Value::Number(n) => Some(format!("{tk}={n}")),
                    _ => None,
                };
                if let Some(t) = tag {
                    *type_counts.entry(t).or_insert(0) += 1;
                    break;
                }
            }
        }
        let is_err = obj.get("error").map(|v| !v.is_null() && v != &Value::Bool(false) && v != &json!("")).unwrap_or(false)
            || obj.get("errors").map(|v| !v.is_null() && v != &json!([])).unwrap_or(false)
            || obj.get("isError") == Some(&Value::Bool(true))
            || obj.get("ok") == Some(&Value::Bool(false))
            || obj
                .get("level")
                .and_then(Value::as_str)
                .map(|l| matches!(l.to_ascii_lowercase().as_str(), "error" | "fatal" | "critical"))
                .unwrap_or(false)
            || obj.get("status").and_then(Value::as_i64).map(|s| s >= 400).unwrap_or(false);
        if is_err && err_records.len() < 5 {
            err_records.push(format!("  - L{lineno}: {}", trunc_chars(ln, 220)));
        }
    }

    let mut kc_vec: Vec<(String, usize)> = key_counts.into_iter().collect();
    kc_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top_keys = kc_vec
        .into_iter()
        .take(12)
        .map(|(k, c)| format!("{k}({c})"))
        .collect::<Vec<_>>()
        .join(", ");

    let mut out = vec![format!(
        "JSONL Stream ({parsed_count} records across {} lines) | Schema: {top_keys}",
        all_lines.len()
    )];
    if !type_counts.is_empty() {
        let mut tc_vec: Vec<(String, usize)> = type_counts.into_iter().collect();
        tc_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let dist = tc_vec
            .into_iter()
            .take(8)
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push(format!("Distribution: {dist}"));
    }
    if !err_records.is_empty() {
        out.push(format!(
            "High-Signal Error/Alert Records (showing {}):",
            err_records.len()
        ));
        out.extend(err_records);
    }
    if !first_preview.is_empty() {
        out.push(format!("First record (L{}): {first_preview}", non_empty[0].0));
    }
    if !last_preview.is_empty() && non_empty.len() > 1 {
        out.push(format!(
            "Last record (L{}): {last_preview}",
            non_empty[non_empty.len() - 1].0
        ));
    }
    Some(out.join("\n"))
}

fn try_summarize_tabular(raw: &str) -> Option<(String, String)> {
    let non_empty: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.len() < 4 {
        return None;
    }
    let sample_txt = non_empty[..non_empty.len().min(15)].join("\n");
    for (delim, kind_name) in [(b'\t', "tsv"), (b',', "csv")] {
        if non_empty[0].as_bytes().iter().filter(|&&b| b == delim).count() < 2 {
            continue;
        }
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(delim)
            .flexible(true)
            .from_reader(sample_txt.as_bytes());
        let sample_rows: Vec<Vec<String>> = rdr
            .records()
            .flatten()
            .map(|rec| rec.iter().map(String::from).collect())
            .collect();
        if sample_rows.len() < 4 {
            continue;
        }
        let col_n = sample_rows[0].len();
        if col_n < 3
            || sample_rows[0].iter().take(2).any(|c| c.trim().is_empty())
            || sample_rows[0].iter().any(|c| c.chars().count() > 64)
        {
            continue;
        }
        let consistent = sample_rows[1..].iter().filter(|r| r.len() == col_n).count();
        if consistent * 5 < (sample_rows.len() - 1) * 4 {
            continue;
        }
        let mut full_rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(delim)
            .flexible(true)
            .from_reader(raw.as_bytes());
        let all_rows: Vec<Vec<String>> = full_rdr
            .records()
            .flatten()
            .map(|rec| rec.iter().map(String::from).collect::<Vec<_>>())
            .filter(|r| r.iter().any(|c| !c.trim().is_empty()))
            .collect();
        let hdr_cols: Vec<&str> = sample_rows[0].iter().take(10).map(String::as_str).collect();
        let summary = format!(
            "{} Table ({} data rows x {col_n} columns): {}\n{}",
            kind_name.to_uppercase(),
            all_rows.len().saturating_sub(1),
            hdr_cols.join(", "),
            format_pipe_table(&all_rows, 6)
        );
        return Some((kind_name.to_string(), summary));
    }
    None
}

fn decode_basic_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn summarize_html(raw: &str) -> String {
    let mut skip_depth = 0usize;
    let mut in_title = false;
    let mut heading_tag: Option<String> = None;
    let mut title_buf = String::new();
    let mut cur_heading = String::new();
    let mut headings: Vec<String> = Vec::new();
    let mut text_chunks: Vec<String> = Vec::new();
    let mut link_count = 0usize;
    let mut tables: Vec<Vec<Vec<String>>> = Vec::new();
    let mut cur_table: Option<Vec<Vec<String>>> = None;
    let mut cur_row: Option<Vec<String>> = None;
    let mut cur_cell: Option<String> = None;

    let bytes = raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if raw[i..].starts_with("<!--") {
                if let Some(end_c) = raw[i + 4..].find("-->") {
                    i += 4 + end_c + 3;
                    continue;
                }
            }
            if let Some(gt_rel) = raw[i..].find('>') {
                let tag_inner = raw[i + 1..i + gt_rel].trim();
                let is_end = tag_inner.starts_with('/');
                let is_self_close = tag_inner.ends_with('/');
                let clean = tag_inner.trim_start_matches('/').trim_end_matches('/').trim();
                let tag_name: String = clean
                    .split(|c: char| c.is_whitespace() || c == '/')
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();

                if !is_end {
                    if matches!(tag_name.as_str(), "script" | "style" | "svg" | "noscript") && !is_self_close {
                        skip_depth += 1;
                    } else if skip_depth == 0 {
                        match tag_name.as_str() {
                            "title" => in_title = true,
                            "h1" | "h2" | "h3" => {
                                heading_tag = Some(tag_name.to_uppercase());
                                cur_heading.clear();
                            }
                            "a" => link_count += 1,
                            "table" => cur_table = Some(Vec::new()),
                            "tr" if cur_table.is_some() => cur_row = Some(Vec::new()),
                            "th" | "td" if cur_row.is_some() => cur_cell = Some(String::new()),
                            _ => {}
                        }
                    }
                } else {
                    if matches!(tag_name.as_str(), "script" | "style" | "svg" | "noscript") {
                        skip_depth = skip_depth.saturating_sub(1);
                    } else if skip_depth == 0 {
                        match tag_name.as_str() {
                            "title" => in_title = false,
                            "h1" | "h2" | "h3" => {
                                if let Some(ht) = heading_tag.take() {
                                    let h = cur_heading.split_whitespace().collect::<Vec<_>>().join(" ");
                                    if !h.is_empty() {
                                        headings.push(format!("{ht}: {}", trunc_chars(&h, 100)));
                                    }
                                }
                            }
                            "th" | "td" => {
                                if let (Some(cell), Some(row)) = (cur_cell.take(), cur_row.as_mut()) {
                                    let c = cell.split_whitespace().collect::<Vec<_>>().join(" ");
                                    row.push(trunc_chars(&c, 80));
                                }
                            }
                            "tr" => {
                                if let (Some(row), Some(tbl)) = (cur_row.take(), cur_table.as_mut()) {
                                    if row.iter().any(|c| !c.is_empty()) {
                                        tbl.push(row);
                                    }
                                }
                            }
                            "table" => {
                                if let Some(tbl) = cur_table.take() {
                                    if !tbl.is_empty() {
                                        tables.push(tbl);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                i += gt_rel + 1;
                continue;
            }
        }
        let next_lt = raw[i..].find('<').map(|off| i + off).unwrap_or(raw.len());
        if skip_depth == 0 && next_lt > i {
            let text = decode_basic_entities(&raw[i..next_lt]);
            if in_title {
                title_buf.push_str(&text);
            }
            if heading_tag.is_some() {
                cur_heading.push_str(&text);
            }
            if let Some(cell) = cur_cell.as_mut() {
                cell.push_str(&text);
            } else {
                let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if !cleaned.is_empty() {
                    text_chunks.push(cleaned);
                }
            }
        }
        i = next_lt.max(i + 1);
    }

    let title_clean = title_buf.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = if title_clean.is_empty() {
        "(untitled)".to_string()
    } else {
        trunc_chars(&title_clean, 140)
    };
    let body = text_chunks.join(" ");
    let mut out = vec![format!(
        "HTML Document | Title: {title} | Links: {link_count} | Tables: {}",
        tables.len()
    )];
    if !headings.is_empty() {
        out.push("Outline:".to_string());
        for h in headings.iter().take(12) {
            out.push(format!("  - {h}"));
        }
    }
    for (idx, tbl) in tables.iter().take(2).enumerate() {
        out.push(format!(
            "Table #{} ({} rows):\n{}",
            idx + 1,
            tbl.len(),
            format_pipe_table(tbl, 5)
        ));
    }
    let body_lead = trunc_chars(&body, 450);
    let ellipsis = if body.chars().count() > 450 { "..." } else { "" };
    out.push(format!("Lead text: {body_lead}{ellipsis}"));
    out.join("\n")
}

fn summarize_log(raw: &str) -> String {
    let (err_re, ts_re, num_re) = log_regexes();
    let lines: Vec<&str> = raw.lines().collect();
    let n = lines.len();
    let mut norm_counts: HashMap<String, usize> = HashMap::new();
    let mut error_windows: Vec<(usize, usize)> = Vec::new();

    for (idx, &ln) in lines.iter().enumerate() {
        let norm = ts_re.replace_all(ln, "<TS>").trim().to_string();
        if !norm.is_empty() {
            *norm_counts.entry(norm).or_insert(0) += 1;
        }
        if err_re.is_match(ln) {
            let start = idx.saturating_sub(1);
            let end = (idx + 3).min(n.saturating_sub(1));
            if let Some(last) = error_windows.last_mut() {
                if start <= last.1 + 2 {
                    last.1 = last.1.max(end);
                    continue;
                }
            }
            error_windows.push((start, end));
        }
    }

    let mut sections: Vec<String> = Vec::new();
    let head: Vec<String> = lines
        .iter()
        .enumerate()
        .take(5)
        .filter(|(_, l)| !l.trim().is_empty())
        .take(4)
        .map(|(i, l)| format!("L{}: {}", i + 1, trunc_chars(l, 140)))
        .collect();
    if !head.is_empty() {
        sections.push(format!("Head:\n  {}", head.join("\n  ")));
    }

    if !error_windows.is_empty() {
        let mut seen_sigs: HashSet<String> = HashSet::new();
        let mut kept_blocks: Vec<String> = Vec::new();
        for &(s, e) in &error_windows {
            let sig_end = (e + 1).min(s + 3);
            let sig = (s..sig_end)
                .map(|k| trunc_chars(&num_re.replace_all(lines[k], "#"), 80))
                .collect::<Vec<_>>()
                .join("\n");
            if !seen_sigs.insert(sig) {
                continue;
            }
            let snippet = (s..=e)
                .map(|k| format!("L{}: {}", k + 1, trunc_chars(lines[k], 160)))
                .collect::<Vec<_>>()
                .join("\n    ");
            kept_blocks.push(format!("  [L{}-L{}]\n    {snippet}", s + 1, e + 1));
            if kept_blocks.len() >= 4 {
                break;
            }
        }
        let omitted = error_windows.len().saturating_sub(kept_blocks.len());
        let hdr = if omitted > 0 {
            format!(
                "High-Signal Failure/Error Blocks ({} total, {omitted} similar omitted):",
                error_windows.len()
            )
        } else {
            format!("High-Signal Failure/Error Blocks ({} total):", error_windows.len())
        };
        sections.push(format!("{hdr}\n{}", kept_blocks.join("\n")));
    }

    let mut noisy: Vec<(usize, String)> = norm_counts
        .into_iter()
        .filter(|(_, cnt)| *cnt >= 5)
        .map(|(txt, cnt)| (cnt, txt))
        .collect();
    noisy.sort_by_key(|a| std::cmp::Reverse(a.0));
    if !noisy.is_empty() {
        let rep = noisy
            .into_iter()
            .take(3)
            .map(|(cnt, txt)| format!("[x{cnt}] {}", trunc_chars(&txt, 110)))
            .collect::<Vec<_>>()
            .join("\n  ");
        sections.push(format!("Collapsed Repetitions:\n  {rep}"));
    }

    if n > 6 {
        let tail_start = 5.max(n.saturating_sub(5));
        let tail: Vec<String> = (tail_start..n)
            .filter(|&i| !lines[i].trim().is_empty())
            .map(|i| format!("L{}: {}", i + 1, trunc_chars(lines[i], 140)))
            .collect();
        if !tail.is_empty() {
            sections.push(format!("Tail:\n  {}", tail.join("\n  ")));
        }
    }
    sections.join("\n")
}

pub fn compress_stream_in(
    store: &Path,
    raw_text: &str,
    label: &str,
    exit_code: i32,
    elapsed_ms: f64,
) -> SieveResult {
    let raw_bytes = raw_text.len();
    let item_id = archive_stream(store, raw_text, label, exit_code, elapsed_ms);

    if raw_bytes <= PASSTHROUGH_BYTES {
        let rendered = format!(
            "{}\n[sieve id={item_id} | exit={exit_code} | {raw_bytes}B passthrough]",
            raw_text.trim_end()
        );
        let digest_bytes = rendered.len();
        return SieveResult {
            id: item_id,
            mode: "passthrough".to_string(),
            exit_code,
            raw_bytes,
            digest_bytes,
            saved_pct: 0.0,
            raw_tokens_est: est_tokens(raw_text),
            digest_tokens_est: est_tokens(&rendered),
            rendered,
        };
    }

    let stripped = raw_text.trim_start();
    let (kind, body_summary) = if stripped.starts_with('{') || stripped.starts_with('[') {
        if let Ok(parsed) = serde_json::from_str::<Value>(stripped) {
            ("json".to_string(), summarize_json(&parsed))
        } else if let Some(jl) = try_summarize_jsonl(raw_text) {
            ("jsonl".to_string(), jl)
        } else {
            ("log".to_string(), summarize_log(raw_text))
        }
    } else {
        let head_low = trunc_chars(stripped, 400).to_ascii_lowercase();
        if head_low.starts_with("<!doctype html") || head_low.contains("<html") || head_low.contains("<table") {
            ("html".to_string(), summarize_html(raw_text))
        } else if let Some((tk, ts)) = try_summarize_tabular(raw_text) {
            (tk, ts)
        } else {
            ("log".to_string(), summarize_log(raw_text))
        }
    };

    let raw_tok = est_tokens(raw_text);
    let footer = format!(
        "[sieve id={item_id} | kind={kind} | exit={exit_code} | raw={raw_bytes}B (~{raw_tok} tok) | peek: ap sieve peek {item_id} --lines A:B | --grep REGEX]"
    );
    let rendered = format!("{body_summary}\n{footer}");
    let dig_bytes = rendered.len();
    let dig_tok = est_tokens(&rendered);
    let saved_pct = ((1.0 - (dig_bytes as f64) / (raw_bytes.max(1) as f64)) * 1000.0).round() / 10.0;

    SieveResult {
        id: item_id,
        mode: kind,
        exit_code,
        raw_bytes,
        digest_bytes: dig_bytes,
        saved_pct,
        raw_tokens_est: raw_tok,
        digest_tokens_est: dig_tok,
        rendered,
    }
}

fn resolve_json_path<'a>(mut cur: &'a Value, path: &str) -> Option<&'a Value> {
    for part in path.split('.').filter(|p| !p.is_empty()) {
        match cur {
            Value::Array(arr) => {
                let idx: usize = part.parse().ok()?;
                cur = arr.get(idx)?;
            }
            Value::Object(map) => {
                cur = map.get(part)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

fn ensure_backfilled(store: &Path) {
    let db_path = store.join("index.sqlite");
    let mut existing: HashSet<String> = HashSet::new();
    if db_path.exists() {
        if let Ok(conn) = Connection::open(&db_path) {
            if let Ok(mut st) = conn.prepare("SELECT id FROM chunks") {
                if let Ok(rows) = st.query_map([], |r| r.get::<_, String>(0)) {
                    existing.extend(rows.flatten());
                }
            }
        }
    }
    let Ok(rd) = fs::read_dir(store) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("raw") {
            let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if !existing.contains(stem) {
                let meta_p = p.with_extension("meta.json");
                let lbl = fs::read_to_string(&meta_p)
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .and_then(|v| v.get("label").and_then(Value::as_str).map(String::from))
                    .unwrap_or_else(|| "stream".to_string());
                if let Ok(bytes) = fs::read(&p) {
                    index_archive(store, stem, &lbl, &String::from_utf8_lossy(&bytes));
                }
            }
        }
    }
}

pub fn search_archives(store: &Path, query: &str, limit: usize) -> (Vec<SearchHit>, f64) {
    let t0 = Instant::now();
    ensure_backfilled(store);
    let db_path = store.join("index.sqlite");
    let mut hits = Vec::new();
    let q = query.trim();
    if db_path.exists() && !q.is_empty() {
        if let Ok(conn) = Connection::open(db_path) {
            let fts_q = q
                .split_whitespace()
                .filter(|t| !t.is_empty())
                .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(" ");
            if let Ok(mut st) = conn.prepare(
                "SELECT id, label, snippet(chunks, 2, '[', ']', ' ... ', 20) FROM chunks WHERE chunks MATCH ?1 LIMIT ?2",
            ) {
                if let Ok(rows) = st.query_map(params![fts_q, limit as i64], |r| {
                    Ok(SearchHit {
                        id: r.get(0)?,
                        label: r.get(1)?,
                        snippet: r
                            .get::<_, String>(2)?
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" "),
                    })
                }) {
                    hits.extend(rows.flatten());
                }
            }
        }
    }
    let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
    (hits, elapsed_ms)
}

pub fn run_cli(args: &[String]) -> i32 {
    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            return code;
        }
    };
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let store = store_dir(&cwd);

    match cli.subcmd {
        SubCmd::Run { json: as_json, cmd } => {
            let clean_cmd: Vec<String> = if cmd.first().map(|s| s.as_str()) == Some("--") {
                cmd[1..].to_vec()
            } else {
                cmd
            };
            if clean_cmd.is_empty() {
                eprintln!("[sieve] usage: ap sieve run -- <command...>");
                return 2;
            }
            let t0 = Instant::now();
            let out = match Command::new(&clean_cmd[0]).args(&clean_cmd[1..]).current_dir(&cwd).output() {
                Ok(o) => o,
                #[cfg(windows)]
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    match Command::new("cmd").arg("/C").args(&clean_cmd).current_dir(&cwd).output() {
                        Ok(o) => o,
                        Err(e2) => {
                            eprintln!("[sieve] failed to execute `{}`: {e2}", clean_cmd.join(" "));
                            return 127;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[sieve] failed to execute `{}`: {e}", clean_cmd.join(" "));
                    return 127;
                }
            };
            let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
            let mut combined = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr_s = String::from_utf8_lossy(&out.stderr);
            if !stderr_s.is_empty() {
                if !combined.is_empty() && !combined.ends_with('\n') {
                    combined.push('\n');
                }
                combined.push_str(&stderr_s);
            }
            let code = out.status.code().unwrap_or(1);
            let label = clean_cmd.iter().take(3).cloned().collect::<Vec<_>>().join(" ");
            let res = compress_stream_in(&store, &combined, &label, code, elapsed_ms);
            if as_json {
                println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
            } else {
                println!("{}", res.rendered);
            }
            code
        }
        SubCmd::Pipe {
            label,
            json: as_json,
        } => {
            let mut raw = String::new();
            let _ = std::io::stdin().read_to_string(&mut raw);
            let res = compress_stream_in(&store, &raw, &label, 0, 0.0);
            if as_json {
                println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
            } else {
                println!("{}", res.rendered);
            }
            0
        }
        SubCmd::Peek {
            id,
            lines: line_spec,
            grep,
            context,
            max_hits,
            json_path,
        } => {
            let mut match_file: Option<PathBuf> = None;
            if let Ok(rd) = fs::read_dir(&store) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.extension().and_then(|e| e.to_str()) == Some("raw") {
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            if stem.starts_with(&id) {
                                match_file = Some(p);
                                break;
                            }
                        }
                    }
                }
            }
            let Some(raw_path) = match_file else {
                eprintln!("[sieve] error: unknown archive id '{id}' in {}", store.display());
                return 1;
            };
            let raw = String::from_utf8_lossy(&fs::read(&raw_path).unwrap_or_default()).to_string();

            if let Some(jpath) = json_path {
                let parsed_val: Option<Value> = if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                    Some(v)
                } else {
                    let raw_lines: Vec<&str> = raw.lines().collect();
                    if let Some(ls) = &line_spec {
                        let idx = ls
                            .split(':')
                            .next()
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(1)
                            .saturating_sub(1);
                        raw_lines.get(idx).and_then(|ln| serde_json::from_str::<Value>(ln).ok())
                    } else {
                        let arr: Vec<Value> = raw_lines
                            .iter()
                            .filter(|l| !l.trim().is_empty())
                            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
                            .collect();
                        Some(Value::Array(arr))
                    }
                };
                let Some(root_v) = parsed_val else {
                    eprintln!("[sieve] archive '{id}' is not valid JSON/JSONL");
                    return 1;
                };
                let Some(sub) = resolve_json_path(&root_v, &jpath) else {
                    eprintln!("[sieve] json-path '{jpath}' not found in archive '{id}'");
                    return 1;
                };
                let pretty = serde_json::to_string_pretty(sub).unwrap_or_default();
                println!("{}", trunc_chars(&pretty, 8000));
                return 0;
            }

            let all_lines: Vec<&str> = raw.lines().collect();
            if let Some(ls) = line_spec {
                let parts: Vec<&str> = ls.split(':').collect();
                let start = parts
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1)
                    .max(1);
                let end = parts
                    .get(1)
                    .filter(|s| !s.is_empty())
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(start + 40)
                    .min(all_lines.len());
                if start <= end && start <= all_lines.len() {
                    for (idx, line_txt) in all_lines.iter().enumerate().take(end).skip(start - 1) {
                        println!("L{}: {line_txt}", idx + 1);
                    }
                }
                return 0;
            }

            if let Some(pat_str) = grep {
                let pat = match Regex::new(&format!("(?i){pat_str}")) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("[sieve] invalid regex '{pat_str}': {e}");
                        return 2;
                    }
                };
                let mut shown: HashSet<usize> = HashSet::new();
                let mut hits = 0usize;
                for (i, &ln) in all_lines.iter().enumerate() {
                    if pat.is_match(ln) {
                        hits += 1;
                        let s = i.saturating_sub(context);
                        let e = (i + context + 1).min(all_lines.len());
                        for (j, j_line) in all_lines.iter().enumerate().take(e).skip(s) {
                            if shown.insert(j) {
                                let marker = if j == i { '>' } else { ' ' };
                                println!("{marker}L{}: {j_line}", j + 1);
                            }
                        }
                        if hits >= max_hits {
                            println!("... (capped at {max_hits} matches)");
                            break;
                        }
                    }
                }
                if hits == 0 {
                    println!("[sieve] 0 matches for /{pat_str}/ in {} lines.", all_lines.len());
                }
                return 0;
            }

            for (idx, &ln) in all_lines.iter().take(40).enumerate() {
                println!("L{}: {ln}", idx + 1);
            }
            0
        }
        SubCmd::Search {
            query,
            limit,
            json: as_json,
        } => {
            let (hits, elapsed_ms) = search_archives(&store, &query, limit);
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "query": query.trim(),
                        "elapsed_ms": elapsed_ms,
                        "hits": hits,
                    }))
                    .unwrap_or_default()
                );
                return 0;
            }
            if hits.is_empty() {
                println!(
                    "[sieve search] 0 matches for {:?} across {} ({elapsed_ms} ms)",
                    query.trim(),
                    store.display()
                );
                return 0;
            }
            println!(
                "[sieve search] {} archive hit(s) for {:?} in {elapsed_ms} ms:",
                hits.len(),
                query.trim()
            );
            for h in hits {
                println!("  - id={} ({}): {}", h.id, h.label, h.snippet);
            }
            0
        }
    }
}
