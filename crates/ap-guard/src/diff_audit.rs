use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFinding {
    pub kind: String,
    pub file: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffAuditReport {
    pub is_git_repo: bool,
    pub files_changed: usize,
    pub lines_added: usize,
    pub lines_deleted: usize,
    pub findings: Vec<DiffFinding>,
}

fn is_test_path(p: &str) -> bool {
    let low = p.to_ascii_lowercase();
    low.starts_with("test/")
        || low.starts_with("tests/")
        || low.contains(".test.")
        || low.contains(".spec.")
        || low.contains("/test_")
        || low.starts_with("test_")
        || low.ends_with("_test.go")
        || low.ends_with("_test.rs")
}

fn is_assertion_line(s: &str) -> bool {
    let low = s.to_ascii_lowercase();
    low.contains("assert")
        || low.contains("expect(")
        || low.contains(".should")
        || low.contains("t.error")
        || low.contains("t.fatal")
}

fn has_debug_leftover(s: &str) -> Option<&'static str> {
    let trim = s.trim();
    if trim.starts_with("//") || trim.starts_with('#') || trim.starts_with('*') {
        return None;
    }
    if trim == "debugger;" || trim == "debugger" || trim.contains(" debugger;") {
        return Some("debugger;");
    }
    if trim.contains("breakpoint()") {
        return Some("breakpoint()");
    }
    if trim.contains("pdb.set_trace()") {
        return Some("pdb.set_trace()");
    }
    if trim.contains("dbg!(") {
        return Some("dbg!(");
    }
    None
}

fn has_test_skip_marker(s: &str) -> Option<&'static str> {
    let trim = s.trim();
    if trim.starts_with("//") || trim.starts_with('#') && !trim.starts_with("#[ignore") {
        return None;
    }
    if trim.contains(".only(") {
        return Some(".only(");
    }
    if trim.contains(".skip(") {
        return Some(".skip(");
    }
    if trim.contains("#[ignore") {
        return Some("#[ignore]");
    }
    if trim.contains("@pytest.mark.skip") {
        return Some("@pytest.mark.skip");
    }
    if trim.contains("t.Skip(") || trim.contains("t.SkipNow(") {
        return Some("t.Skip()");
    }
    None
}

fn has_incomplete_stub(s: &str) -> Option<&'static str> {
    let trim = s.trim();
    if trim.starts_with("//") || trim.starts_with('#') || trim.starts_with('*') {
        return None;
    }
    if trim.contains("todo!(") {
        return Some("todo!()");
    }
    if trim.contains("unimplemented!(") {
        return Some("unimplemented!()");
    }
    if trim.contains("raise NotImplementedError") {
        return Some("raise NotImplementedError");
    }
    let low = trim.to_ascii_lowercase();
    if low.contains("throw new error(\"not implemented")
        || low.contains("throw new error('not implemented")
    {
        return Some("throw new Error('not implemented')");
    }
    None
}

pub fn audit_git_diff(root: &Path) -> DiffAuditReport {
    let check_git = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(root)
        .output();
    let is_git = check_git.map(|o| o.status.success()).unwrap_or(false);
    if !is_git {
        return DiffAuditReport {
            is_git_repo: false,
            files_changed: 0,
            lines_added: 0,
            lines_deleted: 0,
            findings: Vec::new(),
        };
    }

    let diff_out = Command::new("git")
        .args(["diff", "-U0", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .or_else(|| {
            Command::new("git")
                .args(["diff", "-U0"])
                .current_dir(root)
                .output()
                .ok()
        });
    let diff_txt = diff_out
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut cur_file = String::new();
    let mut per_file_add: HashMap<String, usize> = HashMap::new();
    let mut per_file_del: HashMap<String, usize> = HashMap::new();
    let mut per_file_assert_add: HashMap<String, usize> = HashMap::new();
    let mut per_file_assert_del: HashMap<String, usize> = HashMap::new();
    let mut findings: Vec<DiffFinding> = Vec::new();
    let mut total_add = 0usize;
    let mut total_del = 0usize;

    for ln in diff_txt.lines() {
        if let Some(rest) = ln.strip_prefix("+++ b/") {
            cur_file = rest.trim().to_string();
            per_file_add.entry(cur_file.clone()).or_insert(0);
            per_file_del.entry(cur_file.clone()).or_insert(0);
            continue;
        }
        if cur_file.is_empty()
            || ln.starts_with("+++")
            || ln.starts_with("---")
            || ln.starts_with("@@")
        {
            continue;
        }
        if let Some(added) = ln.strip_prefix('+') {
            total_add += 1;
            *per_file_add.entry(cur_file.clone()).or_insert(0) += 1;
            if is_test_path(&cur_file) {
                if is_assertion_line(added) {
                    *per_file_assert_add.entry(cur_file.clone()).or_insert(0) += 1;
                }
                if let Some(tok) = has_test_skip_marker(added) {
                    findings.push(DiffFinding {
                        kind: "TEST-SKIPPED".to_string(),
                        file: cur_file.clone(),
                        detail: format!(
                            "Added test focus/skip `{tok}` in `{}`: `{}`",
                            cur_file,
                            added.trim().chars().take(100).collect::<String>()
                        ),
                    });
                }
            } else {
                if let Some(tok) = has_debug_leftover(added) {
                    findings.push(DiffFinding {
                        kind: "DEBUG-LEFTOVER".to_string(),
                        file: cur_file.clone(),
                        detail: format!(
                            "Added `{tok}` in `{}`: `{}`",
                            cur_file,
                            added.trim().chars().take(100).collect::<String>()
                        ),
                    });
                }
                if let Some(tok) = has_incomplete_stub(added) {
                    findings.push(DiffFinding {
                        kind: "INCOMPLETE-STUB".to_string(),
                        file: cur_file.clone(),
                        detail: format!(
                            "Added unfinished stub `{tok}` in `{}`: `{}`",
                            cur_file,
                            added.trim().chars().take(100).collect::<String>()
                        ),
                    });
                }
                for sec_hit in crate::inspect_text_safety(added) {
                    if sec_hit.starts_with("Credential/secret") {
                        findings.push(DiffFinding {
                            kind: "SECRET-ADDED".to_string(),
                            file: cur_file.clone(),
                            detail: format!("{sec_hit} in `{cur_file}`"),
                        });
                    }
                }
            }
        } else if let Some(deleted) = ln.strip_prefix('-') {
            total_del += 1;
            *per_file_del.entry(cur_file.clone()).or_insert(0) += 1;
            if is_test_path(&cur_file) && is_assertion_line(deleted) {
                *per_file_assert_del.entry(cur_file.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut files: Vec<String> = per_file_add.keys().cloned().collect();
    files.sort();
    for f in &files {
        let a = *per_file_add.get(f).unwrap_or(&0);
        let d = *per_file_del.get(f).unwrap_or(&0);
        let aa = *per_file_assert_add.get(f).unwrap_or(&0);
        let ad = *per_file_assert_del.get(f).unwrap_or(&0);
        if is_test_path(f) && ad > aa {
            findings.push(DiffFinding {
                kind: "TEST-WEAKENING".to_string(),
                file: f.clone(),
                detail: format!(
                    "Removed {ad} assertion(s) in `{f}` while adding only {aa} — verify you did not delete tests to force a green build."
                ),
            });
        }
        if d >= 25 && d > a.saturating_mul(3) {
            findings.push(DiffFinding {
                kind: "MASS-DELETION".to_string(),
                file: f.clone(),
                detail: format!(
                    "Deleted {d} lines vs +{a} added in `{f}` — verify no accidental truncation or unintended action-bias deletion."
                ),
            });
        }
    }

    DiffAuditReport {
        is_git_repo: true,
        files_changed: files.len(),
        lines_added: total_add,
        lines_deleted: total_del,
        findings,
    }
}
