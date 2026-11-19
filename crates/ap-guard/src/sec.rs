use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecFinding {
    pub cwe: String,
    pub file: String,
    pub line: usize,
    pub snippet: String,
    pub fix: String,
}

struct SecPatterns {
    js_sqli: Regex,
    py_sqli: Regex,
    cmd_inj: Regex,
    dom_xss_tpl: Regex,
    tpl_expr: Regex,
    path_trav_sink: Regex,
    path_trav_dyn: Regex,
    debug_tls: Regex,
}

fn sec_re() -> &'static SecPatterns {
    static RE: OnceLock<SecPatterns> = OnceLock::new();
    RE.get_or_init(|| SecPatterns {
        js_sqli: Regex::new(
            r"(?i)\.(?:exec|prepare|query|run|all|get)\s*\(\s*`[^`]*(?:SELECT|INSERT|UPDATE|DELETE)\b[^`]*\$\{",
        )
        .unwrap(),
        py_sqli: Regex::new(
            r#"(?i)\.execute\s*\(\s*(?:f["'][^"']*(?:SELECT|INSERT|UPDATE|DELETE)\b[^"']*\{|["'][^"']*(?:SELECT|INSERT|UPDATE|DELETE)\b[^"']*["']\s*(?:%|\+\s*[A-Za-z_]|\.format\())"#,
        )
        .unwrap(),
        cmd_inj: Regex::new(
            r"(?:execSync|child_process\.exec)\s*\(\s*`[^`]*\$\{|os\.system\s*\(\s*f['\x22]|subprocess\.(?:run|Popen|call)\s*\([^)]*\bshell\s*=\s*True",
        )
        .unwrap(),
        dom_xss_tpl: Regex::new(r"(?s)\.innerHTML\s*(?:\+?=)\s*`([^`]*)`").unwrap(),
        tpl_expr: Regex::new(r"\$\{([^}]+)\}").unwrap(),
        path_trav_sink: Regex::new(
            r"(?:readFileSync|readFile|createReadStream|sendFile)\s*\([^)]*(?:req\.url|pathname|searchParams|params\.|query\.)",
        )
        .unwrap(),
        path_trav_dyn: Regex::new(
            r"(?:readFileSync|readFile|createReadStream|sendFile)\s*\(\s*[A-Za-z_$][A-Za-z0-9_$.]*\s*(?:,|\))",
        )
        .unwrap(),
        debug_tls: Regex::new(
            r"(?:rejectUnauthorized\s*:\s*false|NODE_TLS_REJECT_UNAUTHORIZED\s*=\s*['\x22]0['\x22]|app\.run\s*\([^)]*debug\s*=\s*True)",
        )
        .unwrap(),
    })
}

fn has_escape_wrapper(expr: &str) -> bool {
    let low = expr.to_ascii_lowercase();
    low.contains("esc(")
        || low.contains("escape")
        || low.contains("sanitize")
        || low.contains("encodeuri")
        || low.contains("encodehtml")
        || low.contains("safehtml")
        || low.contains("number(")
        || low.contains("math.")
        || low.ends_with(".length")
        || low.ends_with("cents")
        || low.ends_with("count")
        || low.ends_with("stock")
        || low.ends_with("price")
}

fn walk_code_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if out.len() >= 300 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        let nm = entry.file_name().to_string_lossy().to_string();
        if nm.starts_with('.')
            || matches!(
                nm.as_str(),
                "node_modules" | "target" | "dist" | "build" | "__pycache__" | "vendor"
            )
        {
            continue;
        }
        if p.is_dir() {
            walk_code_files(&p, out);
        } else if p.is_file() {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(
                ext,
                "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx" | "py" | "go" | "rs"
            ) {
                out.push(p);
            }
        }
    }
}

pub fn scan_security_issues(root: &Path) -> Vec<SecFinding> {
    let re = sec_re();
    let mut files = Vec::new();
    walk_code_files(root, &mut files);
    let mut findings = Vec::new();

    for fp in files {
        let rel = fp
            .strip_prefix(root)
            .unwrap_or(&fp)
            .to_string_lossy()
            .replace('\\', "/");
        let low_rel = rel.to_ascii_lowercase();
        if low_rel.starts_with("tests/")
            || low_rel.starts_with("test/")
            || low_rel.contains(".test.")
            || low_rel.contains(".spec.")
            || low_rel.contains("_test.")
        {
            continue;
        }
        let Ok(txt) = fs::read_to_string(&fp) else {
            continue;
        };
        let has_path_guard = txt.contains(".startsWith(")
            || txt.contains("path.basename(")
            || txt.contains("path.relative(")
            || txt.contains("os.path.commonpath")
            || txt.contains("includes('..')")
            || txt.contains("includes(\"..\")");
        let has_req_path_taint =
            txt.contains("searchParams") || txt.contains("req.query") || txt.contains("req.params");

        // Multi-line & multi-expression CWE-79 DOM XSS audit across `.innerHTML = \`...\``
        for cap in re.dom_xss_tpl.captures_iter(&txt) {
            let Some(m_full) = cap.get(0) else { continue };
            let tpl_body = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let mut unsafe_expr = None;
            for ec in re.tpl_expr.captures_iter(tpl_body) {
                let expr = ec.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                if !expr.is_empty() && !has_escape_wrapper(expr) {
                    unsafe_expr = Some(expr.to_string());
                    break;
                }
            }
            if let Some(expr) = unsafe_expr {
                let lineno = txt[..m_full.start()]
                    .bytes()
                    .filter(|&b| b == b'\n')
                    .count()
                    + 1;
                findings.push(SecFinding {
                    cwe: "CWE-79 DOM XSS".to_string(),
                    file: rel.clone(),
                    line: lineno,
                    snippet: format!("innerHTML interpolates `${{{expr}}}` without escaping"),
                    fix: "Use `textContent` / DOM nodes or wrap interpolated data in an `escapeHtml(...)` helper before assigning `innerHTML`.".to_string(),
                });
            }
        }

        let mut flagged_trav = false;
        for (idx, ln) in txt.lines().enumerate() {
            let trim = ln.trim();
            if trim.starts_with("//") || trim.starts_with('#') || trim.starts_with('*') {
                continue;
            }
            let lineno = idx + 1;

            if re.js_sqli.is_match(ln) || re.py_sqli.is_match(ln) {
                findings.push(SecFinding {
                    cwe: "CWE-89 SQL Injection".to_string(),
                    file: rel.clone(),
                    line: lineno,
                    snippet: trim.chars().take(120).collect(),
                    fix: "Use parameterized placeholders (`?` / `$1`) instead of interpolating variables into SQL strings.".to_string(),
                });
            }
            if re.cmd_inj.is_match(ln) {
                findings.push(SecFinding {
                    cwe: "CWE-78 Command Injection".to_string(),
                    file: rel.clone(),
                    line: lineno,
                    snippet: trim.chars().take(120).collect(),
                    fix: "Pass explicit argv array to `execFileSync`/`spawnSync`/`subprocess.run(..., shell=False)` instead of shell string interpolation.".to_string(),
                });
            }
            if !has_path_guard
                && !flagged_trav
                && (re.path_trav_sink.is_match(ln)
                    || (has_req_path_taint && re.path_trav_dyn.is_match(ln)))
            {
                flagged_trav = true;
                findings.push(SecFinding {
                    cwe: "CWE-22 Path Traversal".to_string(),
                    file: rel.clone(),
                    line: lineno,
                    snippet: trim.chars().take(120).collect(),
                    fix: "Resolve canonical path (`path.resolve(baseDir, rel)`) and verify `resolved.startsWith(baseDir + path.sep)` before reading files.".to_string(),
                });
            }
            if re.debug_tls.is_match(ln) {
                findings.push(SecFinding {
                    cwe: "CWE-295/489 Insecure Debug/TLS".to_string(),
                    file: rel.clone(),
                    line: lineno,
                    snippet: trim.chars().take(120).collect(),
                    fix: "Do not disable TLS verification or leave `debug=True` enabled in server entrypoints.".to_string(),
                });
            }
        }
    }
    findings
}
