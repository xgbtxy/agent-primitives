use crate::ast::{language_for_ext, scan_symbols};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};

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

const GENERIC_SYMS: &[&str] = &[
    "new", "main", "run", "get", "set", "init", "default", "drop", "fmt", "from", "into", "test",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastHit {
    pub symbol: String,
    pub file: String,
    pub line: usize,
    pub snippet: String,
}

fn extract_sig_map(src: &str, ext: &str) -> HashMap<String, String> {
    let lines: Vec<&str> = src.lines().collect();
    let mut map = HashMap::new();
    for sp in scan_symbols(src, ext) {
        if sp.short.len() < 3 || GENERIC_SYMS.contains(&sp.short.as_str()) {
            continue;
        }
        let end = (sp.start_line + 3).min(sp.end_line).min(lines.len());
        if sp.start_line == 0 || sp.start_line > end {
            continue;
        }
        let raw_hdr = lines[sp.start_line - 1..end].join(" ");
        let head_part = raw_hdr
            .split('{')
            .next()
            .unwrap_or(&raw_hdr)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        map.insert(sp.short, head_part);
    }
    map
}

fn collect_non_import_ast_refs(
    node: Node,
    src: &[u8],
    lines: &[&str],
    targets: &HashSet<&str>,
    rel_file: &str,
    seen: &mut HashSet<(usize, String)>,
    out: &mut Vec<BlastHit>,
) {
    let k = node.kind();
    if k.contains("comment")
        || k.contains("string")
        || k.contains("import")
        || k == "use_declaration"
    {
        return;
    }
    if k.contains("identifier") && node.child_count() == 0 {
        if let Ok(txt) = node.utf8_text(src) {
            if targets.contains(txt) {
                let ln = node.start_position().row + 1;
                if seen.insert((ln, txt.to_string())) {
                    let raw_ln = lines.get(ln - 1).copied().unwrap_or("").trim();
                    let snip: String = raw_ln.chars().take(130).collect();
                    out.push(BlastHit {
                        symbol: txt.to_string(),
                        file: rel_file.to_string(),
                        line: ln,
                        snippet: snip,
                    });
                }
            }
        }
        return;
    }
    let mut walk = node.walk();
    for ch in node.children(&mut walk) {
        collect_non_import_ast_refs(ch, src, lines, targets, rel_file, seen, out);
    }
}

pub fn detect_untouched_blast_radius(
    cwd_canon: &Path,
    touched_paths: &[PathBuf],
    backups: &HashMap<PathBuf, Option<Vec<u8>>>,
    working_text: &HashMap<PathBuf, (String, &'static str)>,
) -> Vec<BlastHit> {
    let mut changed_syms: Vec<String> = Vec::new();
    for p in touched_paths {
        let Some(Some(orig_bytes)) = backups.get(p) else {
            continue;
        };
        let Some((new_lf, _)) = working_text.get(p) else {
            continue;
        };
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if language_for_ext(ext).is_none() {
            continue;
        }
        let old_lf = String::from_utf8_lossy(orig_bytes)
            .trim_start_matches('\u{FEFF}')
            .replace("\r\n", "\n");
        let old_sigs = extract_sig_map(&old_lf, ext);
        let new_sigs = extract_sig_map(new_lf, ext);
        for (sym, old_hdr) in old_sigs {
            let sig_changed = match new_sigs.get(&sym) {
                None => true,
                Some(new_hdr) => new_hdr != &old_hdr,
            };
            if sig_changed && !changed_syms.contains(&sym) {
                changed_syms.push(sym);
            }
        }
    }

    if changed_syms.is_empty() {
        return Vec::new();
    }

    let touched_canon: HashSet<PathBuf> = touched_paths
        .iter()
        .map(|p| fs::canonicalize(p).unwrap_or_else(|_| p.clone()))
        .collect();
    let target_set: HashSet<&str> = changed_syms.iter().map(String::as_str).collect();
    let skip_set: HashSet<&str> = SKIP_DIRS.iter().copied().collect();

    let mut hits = Vec::new();
    let walker = WalkBuilder::new(cwd_canon)
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
        let p_canon = fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        if touched_canon.contains(&p_canon) {
            continue;
        }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        let Some(lang) = language_for_ext(ext) else {
            continue;
        };
        let Ok(bytes) = fs::read(p) else {
            continue;
        };
        let body = String::from_utf8_lossy(&bytes);
        if !changed_syms.iter().any(|s| body.contains(s.as_str())) {
            continue;
        }
        let mut parser = Parser::new();
        let _ = parser.set_language(&lang);
        let Some(tree) = parser.parse(body.as_ref(), None) else {
            continue;
        };
        let rel = p_canon
            .strip_prefix(cwd_canon)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        let lines: Vec<&str> = body.lines().collect();
        let mut seen = HashSet::new();
        collect_non_import_ast_refs(
            tree.root_node(),
            body.as_bytes(),
            &lines,
            &target_set,
            &rel,
            &mut seen,
            &mut hits,
        );
        if hits.len() >= 12 {
            break;
        }
    }
    hits.truncate(10);
    hits
}
