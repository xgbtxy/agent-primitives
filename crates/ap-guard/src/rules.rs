use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRule {
    pub id: String,
    #[serde(default = "default_glob")]
    pub glob: String,
    pub forbid: String,
    pub message: String,
}

fn default_glob() -> String {
    "*".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardConfig {
    #[serde(default = "default_true")]
    pub anti_slop: bool,
    #[serde(default)]
    pub rules: Vec<CodeRule>,
    #[serde(default)]
    pub checks: Vec<String>,
}

fn default_true() -> bool {
    true
}

pub fn config_path(root: &Path) -> PathBuf {
    let agents_cfg = root.join(".agents").join("guard.json");
    if agents_cfg.exists() {
        return agents_cfg;
    }
    let guard_cfg = root.join(".guard").join("rules.json");
    if guard_cfg.exists() {
        return guard_cfg;
    }
    agents_cfg
}

pub fn load_config(root: &Path) -> Option<GuardConfig> {
    let p = config_path(root);
    let txt = fs::read_to_string(p).ok()?;
    serde_json::from_str::<GuardConfig>(&txt).ok()
}

pub fn save_config(root: &Path, cfg: &GuardConfig) -> Result<PathBuf, String> {
    let p = config_path(root);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let js = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&p, format!("{js}\n")).map_err(|e| e.to_string())?;
    Ok(p)
}

pub fn auto_detect_checks(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let pkg = root.join("package.json");
    if pkg.is_file() {
        if let Ok(txt) = fs::read_to_string(&pkg) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                if let Some(scripts) = v.get("scripts").and_then(|s| s.as_object()) {
                    if scripts.contains_key("typecheck") {
                        out.push("npm run typecheck".to_string());
                    } else if root.join("tsconfig.json").is_file()
                        && root.join("node_modules/.bin/tsc").exists()
                    {
                        out.push("npx --no-install tsc --noEmit".to_string());
                    }
                    if scripts.contains_key("lint") {
                        out.push("npm run lint".to_string());
                    } else if root.join("biome.json").is_file()
                        && root.join("node_modules/.bin/biome").exists()
                    {
                        out.push("npx --no-install biome check .".to_string());
                    }
                    if let Some(t) = scripts.get("test").and_then(|x| x.as_str()) {
                        if !t.contains("no test specified") {
                            out.push("npm test".to_string());
                        }
                    }
                }
            }
        }
    } else if root.join("Cargo.toml").is_file() {
        out.push("cargo check --quiet".to_string());
        out.push("cargo test --quiet".to_string());
    } else if root.join("go.mod").is_file() {
        out.push("go vet ./...".to_string());
        out.push("go test ./...".to_string());
    } else if root.join("pyproject.toml").is_file() || root.join("tests").is_dir() {
        out.push("python -m unittest discover".to_string());
    }
    out
}

pub fn default_guard_config(root: &Path) -> GuardConfig {
    GuardConfig {
        anti_slop: true,
        rules: vec![
            CodeRule {
                id: "no-ts-any-or-ignore".to_string(),
                glob: "*.ts,*.tsx".to_string(),
                forbid: r"(:\s*any\b|\bas\s+any\b|@ts-ignore|@ts-nocheck)".to_string(),
                message: "Replace `any` / `@ts-ignore` with concrete domain types.".to_string(),
            },
            CodeRule {
                id: "no-silent-empty-catch".to_string(),
                glob: "*.ts,*.tsx,*.js,*.jsx,*.mjs".to_string(),
                forbid: r"catch\s*(?:\([^)]*\))?\s*\{\s*(?://[^\n]*\s*)?\}".to_string(),
                message: "Never swallow exceptions with an empty `catch {}`; surface or handle the error.".to_string(),
            },
            CodeRule {
                id: "no-focused-test-only".to_string(),
                glob: "*.test.*,*.spec.*".to_string(),
                forbid: r"\b(?:describe|it|test)\.only\s*\(".to_string(),
                message: "Remove `.only(` so the full test suite executes without skipping cases.".to_string(),
            },
        ],
        checks: auto_detect_checks(root),
    }
}

fn matches_glob_list(rel_path: &str, glob_csv: &str) -> bool {
    let fname = rel_path.rsplit('/').next().unwrap_or(rel_path);
    for raw in glob_csv.split(',') {
        let pat = raw.trim();
        if pat.is_empty() || pat == "*" {
            return true;
        }
        if pat.starts_with('*') && pat.ends_with('*') && pat.len() > 2 {
            let mid = &pat[1..pat.len() - 1];
            if fname.contains(mid) || rel_path.contains(mid) {
                return true;
            }
        } else if let Some(ext) = pat.strip_prefix("*.") {
            if fname.ends_with(&format!(".{ext}")) {
                return true;
            }
        } else if let Some(prefix) = pat.strip_suffix("/*") {
            if rel_path.starts_with(&format!("{prefix}/")) {
                return true;
            }
        } else if rel_path.contains(pat) || fname == pat {
            return true;
        }
    }
    false
}

pub fn candidate_changed_files(root: &Path, scan_all: bool) -> Vec<PathBuf> {
    if !scan_all {
        if let Ok(out) = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output()
        {
            if out.status.success() {
                let mut files = Vec::new();
                for ln in String::from_utf8_lossy(&out.stdout).lines() {
                    if ln.len() > 3 {
                        let status_code = &ln[..2];
                        if status_code.contains('D') {
                            continue;
                        }
                        let rel = ln[3..].trim().trim_matches('"');
                        let rel = rel.rsplit(" -> ").next().unwrap_or(rel);
                        let full = root.join(rel);
                        if full.is_file() {
                            files.push(full);
                        } else if full.is_dir() {
                            collect_files_shallow(&full, &mut files);
                        }
                    }
                }
                return files;
            }
        }
    }
    let mut all = Vec::new();
    collect_files_shallow(root, &mut all);
    all
}

fn collect_files_shallow(dir: &Path, out: &mut Vec<PathBuf>) {
    if out.len() >= 250 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        let nm = entry.file_name().to_string_lossy().to_string();
        if nm.starts_with('.')
            || matches!(
                nm.as_str(),
                "node_modules" | "target" | "dist" | "build" | "__pycache__"
            )
        {
            continue;
        }
        if p.is_dir() {
            collect_files_shallow(&p, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

pub fn evaluate_code_rules(root: &Path, cfg: &GuardConfig, scan_all: bool) -> Vec<String> {
    let mut violations = Vec::new();
    let mut active_rules = cfg.rules.clone();
    if cfg.anti_slop && active_rules.is_empty() {
        active_rules = default_guard_config(root).rules;
    }
    let compiled: Vec<(&CodeRule, Regex)> = active_rules
        .iter()
        .filter_map(|r| Regex::new(&r.forbid).ok().map(|re| (r, re)))
        .collect();
    if compiled.is_empty() {
        return violations;
    }

    for fp in candidate_changed_files(root, scan_all) {
        let Ok(meta) = fp.metadata() else { continue };
        if meta.len() > 512_000 {
            continue;
        }
        let rel = fp
            .strip_prefix(root)
            .unwrap_or(&fp)
            .to_string_lossy()
            .replace('\\', "/");
        let is_test_file = rel.starts_with("tests/")
            || rel.starts_with("test/")
            || rel.contains(".test.")
            || rel.contains(".spec.")
            || rel.contains("_test.");
        let Ok(txt) = fs::read_to_string(&fp) else {
            continue;
        };
        for (rule, re) in &compiled {
            if is_test_file && !rule.glob.contains("test") && !rule.glob.contains("spec") {
                continue;
            }
            if !matches_glob_list(&rel, &rule.glob) {
                continue;
            }
            if let Some(m) = re.find(&txt) {
                let line = txt[..m.start()].bytes().filter(|b| *b == b'\n').count() + 1;
                violations.push(
                    RuleViolationFmt {
                        rel: &rel,
                        line,
                        id: &rule.id,
                        matched: m.as_str().trim(),
                        msg: &rule.message,
                    }
                    .render(),
                );
            }
        }
    }
    violations
}

struct RuleViolationFmt<'a> {
    rel: &'a str,
    line: usize,
    id: &'a str,
    matched: &'a str,
    msg: &'a str,
}

impl<'a> RuleViolationFmt<'a> {
    fn render(&self) -> String {
        format!(
            "`{}:{}` violated rule `[{}]` (matched `{}`): {}",
            self.rel, self.line, self.id, self.matched, self.msg
        )
    }
}
