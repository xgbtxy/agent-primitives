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
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub pattern: String,
    pub file: String,
    pub line: usize,
    pub scope: String,
    pub snippet: String,
}

#[derive(Parser, Debug)]
#[command(
    name = "search",
    about = "Universal .gitignore-aware regex search with enclosing AST symbol spans + batch multi-pattern + .map/.asar/.vsix/Office/minified support"
)]
struct SearchCli {
    /// Regex or literal pattern to search (or pass `--stdin` for 1 pattern per line)
    #[arg(default_value = "")]
    pattern: String,
    /// Root file or directory to search (default: '.')
    #[arg(default_value = ".")]
    path: String,
    /// Additional patterns to search in the SAME single pass (repeatable: `--pat A --pat B`)
    #[arg(long = "pat")]
    extra_pats: Vec<String>,
    /// Read additional search patterns from stdin (1 regex per line)
    #[arg(long)]
    stdin: bool,
    /// Case-sensitive matching (default is smart case: case-insensitive unless pattern has uppercase)
    #[arg(short = 's', long)]
    case_sensitive: bool,
    /// Optional glob extension filter, e.g. `ts,js` or `xlsx,docx`
    #[arg(short = 'e', long = "ext", default_value = "")]
    ext_filter: String,
    /// Maximum matches to return
    #[arg(long, default_value_t = 50)]
    max_hits: usize,
    /// Context characters on each side when slicing matches inside giant/minified single-line bundles
    #[arg(long, default_value_t = 160)]
    around: usize,
    /// Pretty-format minified JS/JSON match slices
    #[arg(long)]
    pretty: bool,
    /// Enable cross-line multiline regex search (`(?s)` / `\n` across lines)
    #[arg(short = 'm', long)]
    multiline: bool,
    /// Emit full enclosing Tree-sitter AST function/class block around each code match
    #[arg(long)]
    block: bool,
    /// Emit structured JSON output
    #[arg(long)]
    json: bool,
}

#[derive(Clone, Copy)]
struct SearchOpts {
    around: usize,
    pretty: bool,
    multiline: bool,
    block: bool,
    max_hits: usize,
}

fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    idx = idx.min(s.len());
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

fn ceil_char_boundary(s: &str, mut idx: usize) -> usize {
    idx = idx.min(s.len());
    while idx < s.len() && !s.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

#[allow(clippy::too_many_arguments)]
fn push_text_matches(
    pat_label: &str,
    text: &str,
    rel: &str,
    ext: &str,
    is_office: bool,
    re: &Regex,
    opts: SearchOpts,
    matches: &mut Vec<SearchMatch>,
) {
    if !re.is_match(text) {
        return;
    }
    let ast_spans = if !is_office
        && text.len() <= 2 * 1024 * 1024
        && ap_splice::ast::language_for_ext(ext).is_some()
    {
        ap_splice::ast::scan_symbols(text, ext)
    } else {
        Vec::new()
    };
    let all_lines: Vec<&str> = text.lines().collect();

    if opts.multiline {
        for m in re.find_iter(text) {
            let ln_no = text[..m.start()].bytes().filter(|&b| b == b'\n').count() + 1;
            let enc = ast_spans
                .iter()
                .filter(|sp| sp.start_line <= ln_no && ln_no <= sp.end_line)
                .min_by_key(|sp| sp.end_line.saturating_sub(sp.start_line));
            let scope = enc
                .map(|sp| {
                    format!(
                        "{} {} L{}-L{}",
                        sp.kind, sp.qual, sp.start_line, sp.end_line
                    )
                })
                .unwrap_or_default();
            let snip = if opts.block {
                if let Some(sp) = enc {
                    let s = sp.start_line.saturating_sub(1);
                    let e = sp.end_line.min(all_lines.len()).min(s + 30);
                    all_lines[s..e].join("\n")
                } else {
                    m.as_str().chars().take(300).collect()
                }
            } else {
                m.as_str().trim().chars().take(240).collect()
            };
            matches.push(SearchMatch {
                pattern: pat_label.to_string(),
                file: rel.to_string(),
                line: ln_no,
                scope,
                snippet: snip,
            });
            if matches.len() >= opts.max_hits {
                return;
            }
        }
        return;
    }

    let mut line_byte_start = 0usize;
    for (idx, ln) in text.split('\n').enumerate() {
        let ln_clean = ln.strip_suffix('\r').unwrap_or(ln);
        let ln_no = idx + 1;
        if re.is_match(ln_clean) {
            if ln_clean.len() > 300 {
                // Minified / giant single-line bundle: emit every match as a byte-window slice!
                for m in re.find_iter(ln_clean) {
                    let s = floor_char_boundary(ln_clean, m.start().saturating_sub(opts.around));
                    let e = ceil_char_boundary(ln_clean, m.end().saturating_add(opts.around));
                    let raw_snip = &ln_clean[s..e];
                    let snip = if opts.pretty {
                        crate::binary::pretty_format_js_snippet(raw_snip)
                    } else {
                        raw_snip.to_string()
                    };
                    let abs_off = line_byte_start + m.start();
                    matches.push(SearchMatch {
                        pattern: pat_label.to_string(),
                        file: rel.to_string(),
                        line: ln_no,
                        scope: format!("minified @off={abs_off}"),
                        snippet: snip,
                    });
                    if matches.len() >= opts.max_hits {
                        return;
                    }
                }
            } else {
                let enc = ast_spans
                    .iter()
                    .filter(|sp| sp.start_line <= ln_no && ln_no <= sp.end_line)
                    .min_by_key(|sp| sp.end_line.saturating_sub(sp.start_line));
                let scope = enc
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
                let snip: String = if opts.block {
                    if let Some(sp) = enc {
                        let s = sp.start_line.saturating_sub(1);
                        let e = sp.end_line.min(all_lines.len()).min(s + 30);
                        all_lines[s..e].join("\n")
                    } else {
                        ln_clean.trim().chars().take(180).collect()
                    }
                } else {
                    ln_clean.trim().chars().take(180).collect()
                };
                matches.push(SearchMatch {
                    pattern: pat_label.to_string(),
                    file: rel.to_string(),
                    line: ln_no,
                    scope,
                    snippet: snip,
                });
                if matches.len() >= opts.max_hits {
                    return;
                }
            }
        }
        line_byte_start += ln.len() + 1;
    }
}

fn build_search_regex(pat: &str, force_case: bool, multiline: bool) -> Result<Regex, String> {
    let has_upper = pat.chars().any(|c| c.is_uppercase());
    let case_insens = !force_case && !has_upper;
    RegexBuilder::new(pat)
        .case_insensitive(case_insens)
        .multi_line(multiline)
        .dot_matches_new_line(multiline)
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
    let mut raw_pats: Vec<String> = Vec::new();
    if !cli.pattern.trim().is_empty() {
        raw_pats.push(cli.pattern.clone());
    }
    for ep in &cli.extra_pats {
        if !ep.trim().is_empty() && !raw_pats.contains(ep) {
            raw_pats.push(ep.clone());
        }
    }
    if cli.stdin {
        let mut buf = String::new();
        let _ = std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf);
        for ln in buf.lines() {
            let t = ln.trim();
            if !t.is_empty() && !t.starts_with('#') && !raw_pats.iter().any(|p| p == t) {
                raw_pats.push(t.to_string());
            }
        }
    }
    if raw_pats.is_empty() {
        eprintln!("[ap search] provide at least one pattern (positional, --pat, or --stdin)");
        return 2;
    }
    let multi_mode = raw_pats.len() > 1;
    let mut compiled: Vec<(String, Regex)> = Vec::new();
    for p in raw_pats.into_iter().take(20) {
        match build_search_regex(&p, cli.case_sensitive, cli.multiline) {
            Ok(r) => compiled.push((p, r)),
            Err(e) => {
                eprintln!("[ap search] {e}");
                return 2;
            }
        }
    }
    let combined_label = compiled
        .iter()
        .map(|(p, _)| p.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    let s_opts = SearchOpts {
        around: cli.around,
        pretty: cli.pretty,
        multiline: cli.multiline,
        block: cli.block,
        max_hits: cli.max_hits,
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
        let explicit_file = root_canon.is_file();
        if matches!(
            ext.as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "webp"
                | "ico"
                | "woff"
                | "woff2"
                | "ttf"
                | "sqlite"
                | "db"
                | "lock"
        ) {
            continue;
        }
        let is_bin_ext = matches!(
            ext.as_str(),
            "exe" | "dll" | "so" | "dylib" | "node" | "wasm" | "bin"
        );
        if is_bin_ext && !explicit_file && ext_allow.is_empty() {
            continue;
        }
        let Ok(meta) = p.metadata() else {
            continue;
        };
        let size_cap = if explicit_file {
            64 * 1024 * 1024
        } else {
            16 * 1024 * 1024
        };
        if meta.len() > size_cap {
            continue;
        }
        let Ok(bytes) = fs::read(p) else {
            continue;
        };
        scanned_files += 1;
        total_bytes += bytes.len() as u64;

        let rel = if explicit_file {
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

        // SourceMap v3 embedded original sources search (`.js.map` / `.map`)
        if ext == "map" {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                if crate::inspect::is_sourcemap_v3(&v) {
                    if let (Some(srcs), Some(contents)) = (
                        v.get("sources").and_then(|x| x.as_array()),
                        v.get("sourcesContent").and_then(|x| x.as_array()),
                    ) {
                        for (idx, s_val) in srcs.iter().enumerate() {
                            let Some(sname) = s_val.as_str() else {
                                continue;
                            };
                            let Some(body) = contents.get(idx).and_then(|x| x.as_str()) else {
                                continue;
                            };
                            let eext = Path::new(sname)
                                .extension()
                                .and_then(|s| s.to_str())
                                .unwrap_or("js")
                                .to_ascii_lowercase();
                            let virt_rel = format!("{rel}::{sname}");
                            for (pl, r) in &compiled {
                                let lbl = if multi_mode { pl.as_str() } else { "" };
                                push_text_matches(
                                    lbl,
                                    body,
                                    &virt_rel,
                                    &eext,
                                    false,
                                    r,
                                    s_opts,
                                    &mut matches,
                                );
                                if matches.len() >= cli.max_hits {
                                    break;
                                }
                            }
                            if matches.len() >= cli.max_hits {
                                break;
                            }
                        }
                    }
                    if matches.len() >= cli.max_hits {
                        break;
                    }
                    continue;
                }
            }
        }

        // Virtual Electron .asar archive search (zero extraction!)
        if ext == "asar" {
            if let Ok((base, entries)) = crate::binary::parse_asar_index(&bytes) {
                for ent in entries {
                    if ent.unpacked || ent.size == 0 || ent.size > 12 * 1024 * 1024 {
                        continue;
                    }
                    let eext = Path::new(&ent.path)
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if !matches!(
                        eext.as_str(),
                        "js" | "mjs"
                            | "cjs"
                            | "ts"
                            | "tsx"
                            | "jsx"
                            | "json"
                            | "html"
                            | "htm"
                            | "css"
                            | "xml"
                            | "txt"
                            | "md"
                            | "yml"
                            | "yaml"
                            | "map"
                    ) {
                        continue;
                    }
                    let st = base.saturating_add(ent.offset);
                    let en = st.saturating_add(ent.size);
                    if en <= bytes.len() {
                        let txt = decode_any_bytes(&bytes[st..en]).0;
                        let virt_rel = format!("{rel}::{}", ent.path);
                        for (pl, r) in &compiled {
                            let lbl = if multi_mode { pl.as_str() } else { "" };
                            push_text_matches(
                                lbl,
                                &txt,
                                &virt_rel,
                                &eext,
                                false,
                                r,
                                s_opts,
                                &mut matches,
                            );
                            if matches.len() >= cli.max_hits {
                                break;
                            }
                        }
                        if matches.len() >= cli.max_hits {
                            break;
                        }
                    }
                }
            }
            if matches.len() >= cli.max_hits {
                break;
            }
            continue;
        }

        // Virtual .zip / .vsix / .jar / .whl archive search (when explicitly targeted or via --ext)
        if matches!(ext.as_str(), "zip" | "vsix" | "jar" | "whl")
            && (explicit_file || !ext_allow.is_empty())
        {
            if let Ok(mut za) = zip::ZipArchive::new(std::io::Cursor::new(&bytes)) {
                for i in 0..za.len() {
                    if let Ok(mut zf) = za.by_index(i) {
                        if zf.is_dir() || zf.size() > 8 * 1024 * 1024 {
                            continue;
                        }
                        let nm = zf.name().replace('\\', "/");
                        let eext = Path::new(&nm)
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        if !matches!(
                            eext.as_str(),
                            "js" | "mjs"
                                | "cjs"
                                | "ts"
                                | "json"
                                | "html"
                                | "xml"
                                | "py"
                                | "txt"
                                | "md"
                                | "yml"
                                | "yaml"
                                | "properties"
                        ) {
                            continue;
                        }
                        let mut eb = Vec::new();
                        if std::io::Read::read_to_end(&mut zf, &mut eb).is_ok() {
                            let txt = decode_any_bytes(&eb).0;
                            let virt_rel = format!("{rel}::{nm}");
                            for (pl, r) in &compiled {
                                let lbl = if multi_mode { pl.as_str() } else { "" };
                                push_text_matches(
                                    lbl,
                                    &txt,
                                    &virt_rel,
                                    &eext,
                                    false,
                                    r,
                                    s_opts,
                                    &mut matches,
                                );
                                if matches.len() >= cli.max_hits {
                                    break;
                                }
                            }
                            if matches.len() >= cli.max_hits {
                                break;
                            }
                        }
                    }
                }
            }
            if matches.len() >= cli.max_hits {
                break;
            }
            continue;
        }

        // Binary ASCII + Windows UTF-16LE strings search when explicitly targeted
        if is_bin_ext || (explicit_file && crate::binary::is_binary_magic(&bytes)) {
            for (pl, _) in &compiled {
                let lbl = if multi_mode {
                    pl.clone()
                } else {
                    String::new()
                };
                let md = crate::binary::inspect_binary_md(&bytes, Some(pl), cli.max_hits);
                for (idx, ln) in md.lines().enumerate() {
                    if ln.starts_with("[ascii ") || ln.starts_with("[utf16le ") {
                        matches.push(SearchMatch {
                            pattern: lbl.clone(),
                            file: rel.clone(),
                            line: idx + 1,
                            scope: "BINARY".to_string(),
                            snippet: ln.chars().take(200).collect(),
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
            if matches.len() >= cli.max_hits {
                break;
            }
            continue;
        }

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

        for (pl, r) in &compiled {
            let lbl = if multi_mode { pl.as_str() } else { "" };
            push_text_matches(lbl, &text, &rel, &ext, is_office, r, s_opts, &mut matches);
            if matches.len() >= cli.max_hits {
                break;
            }
        }
        if matches.len() >= cli.max_hits {
            break;
        }
    }

    let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0;
    let mut out_lines = vec![format!(
        "# SEARCH `{combined_label}` ({} hit(s) across {scanned_files} files in {elapsed_ms}ms)",
        matches.len()
    )];
    for m in &matches {
        let ptag = if m.pattern.is_empty() {
            String::new()
        } else {
            format!("[pat:`{}`] ", m.pattern)
        };
        let sc = if m.scope.is_empty() {
            String::new()
        } else {
            format!(" [{}]", m.scope)
        };
        out_lines.push(format!(
            "- {ptag}`{}:{}`{sc} — `{}`",
            m.file, m.line, m.snippet
        ));
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
