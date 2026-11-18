use crate::read::{decode_any_bytes, extract_office_or_pdf_plain};
use crate::stats::record_event;
use clap::Parser;
use ignore::WalkBuilder;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Instant;

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "__pycache__",
    ".venv",
    "venv",
    ".next",
    ".sieve",
    ".agents",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub file: String,
    pub line: usize,
    pub scope: String,
    pub snippet: String,
}

#[derive(Parser, Debug)]
#[command(
    name = "search",
    about = "Universal .gitignore-aware regex search with enclosing AST symbol spans + .docx/.xlsx/.pdf/GBK support (zero ripgrep needed)"
)]
struct SearchCli {
    /// Regex or literal pattern to search
    pattern: String,
    /// Root file or directory to search (default: '.')
    #[arg(default_value = ".")]
    path: String,
    /// Case-sensitive matching (default is smart case: case-insensitive unless pattern has uppercase)
    #[arg(short = 's', long)]
    case_sensitive: bool,
    /// Optional glob extension filter, e.g. `ts,js` or `xlsx,docx`
    #[arg(short = 'e', long = "ext", default_value = "")]
    ext_filter: String,
    /// Maximum matches to return
    #[arg(long, default_value_t = 50)]
    max_hits: usize,
    /// Emit structured JSON output
    #[arg(long)]
    json: bool,
}

fn build_search_regex(pat: &str, force_case: bool) -> Result<Regex, String> {
    let has_upper = pat.chars().any(|c| c.is_uppercase());
    let case_insens = !force_case && !has_upper;
    RegexBuilder::new(pat)
        .case_insensitive(case_insens)
        .build()
        .or_else(|_| {
            RegexBuilder::new(&regex::escape(pat))
                .case_insensitive(case_insens)
                .build()
        })
        .map_err(|e| format!("invalid pattern `{pat}`: {e}"))
}

pub fn run_cli(args: &[String]) -> i32 {
    let cli = match SearchCli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            return code;
        }
    };
    let re = match build_search_regex(&cli.pattern, cli.case_sensitive) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ap search] {e}");
            return 2;
        }
    };

    let root = Path::new(&cli.path);
    if !root.exists() {
        eprintln!("[ap search] path not found: {}", cli.path);
        return 1;
    }
    let t0 = Instant::now();
    let root_canon = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let ext_allow: HashSet<String> = cli
        .ext_filter
        .split(',')
        .map(|s| s.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    let skip_set: HashSet<&str> = SKIP_DIRS.iter().copied().collect();

    let mut matches: Vec<SearchMatch> = Vec::new();
    let mut scanned_files = 0usize;
    let mut total_bytes = 0u64;

    let walker = WalkBuilder::new(&root_canon)
        .hidden(false)
        .git_ignore(true)
        .filter_entry(move |e| {
            if e.depth() == 0 {
                return true;
            }
            let nm = e.file_name().to_str().unwrap_or("");
            !skip_set.contains(nm)
        })
        .build();

    for entry in walker.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let ext = p
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !ext_allow.is_empty() && !ext_allow.contains(&ext) {
            continue;
        }
        if matches!(
            ext.as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "webp"
                | "ico"
                | "exe"
                | "dll"
                | "so"
                | "dylib"
                | "woff"
                | "woff2"
                | "ttf"
                | "sqlite"
                | "db"
                | "lock"
        ) {
            continue;
        }
        let Ok(meta) = p.metadata() else {
            continue;
        };
        if meta.len() > 16 * 1024 * 1024 {
            continue;
        }
        let Ok(bytes) = fs::read(p) else {
            continue;
        };
        scanned_files += 1;
        total_bytes += bytes.len() as u64;

        let is_office = matches!(ext.as_str(), "docx" | "xlsx" | "pptx" | "pdf");
        let text = if is_office {
            match extract_office_or_pdf_plain(&bytes, &ext) {
                Ok(s) => s,
                Err(_) => continue,
            }
        } else {
            if bytes.iter().take(512).filter(|&&b| b == 0).count() > 4
                && !bytes.starts_with(b"\xFF\xFE")
                && !bytes.starts_with(b"\xFE\xFF")
            {
                continue; // Binary file
            }
            decode_any_bytes(&bytes).0
        };

        if !re.is_match(&text) {
            continue;
        }

        let rel = if root_canon.is_file() {
            p.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        } else {
            p.strip_prefix(&root_canon)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/")
        };

        let ast_spans = if ap_splice::ast::language_for_ext(&ext).is_some() {
            ap_splice::ast::scan_symbols(&text, &ext)
        } else {
            Vec::new()
        };

        for (idx, ln) in text.lines().enumerate() {
            if re.is_match(ln) {
                let ln_no = idx + 1;
                let scope = ast_spans
                    .iter()
                    .filter(|sp| sp.start_line <= ln_no && ln_no <= sp.end_line)
                    .min_by_key(|sp| sp.end_line.saturating_sub(sp.start_line))
                    .map(|sp| {
                        format!(
                            "{} {} L{}-L{}",
                            sp.kind, sp.qual, sp.start_line, sp.end_line
                        )
                    })
                    .unwrap_or_else(|| {
                        if is_office {
                            ext.to_ascii_uppercase()
                        } else {
                            String::new()
                        }
                    });
                let snip: String = ln.trim().chars().take(150).collect();
                matches.push(SearchMatch {
                    file: rel.clone(),
                    line: ln_no,
                    scope,
                    snippet: snip,
                });
                if matches.len() >= cli.max_hits {
                    break;
                }
            }
        }
        if matches.len() >= cli.max_hits {
            break;
        }
    }

    let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0;
    let mut out_lines = vec![format!(
        "# SEARCH `{}` ({} hit(s) across {scanned_files} files in {elapsed_ms}ms)",
        cli.pattern,
        matches.len()
    )];
    for m in &matches {
        let sc = if m.scope.is_empty() {
            String::new()
        } else {
            format!(" [{}]", m.scope)
        };
        out_lines.push(format!("- `{}:{}`{sc} — `{}`", m.file, m.line, m.snippet));
    }
    let rendered = out_lines.join("\n");
    record_event(
        "search",
        total_bytes,
        rendered.len() as u64,
        0,
        0,
        "ast-office-search",
    );

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "elapsed_ms": elapsed_ms,
                "scanned_files": scanned_files,
                "matches": matches,
            }))
            .unwrap_or_default()
        );
    } else {
        println!("{rendered}");
    }
    0
}
