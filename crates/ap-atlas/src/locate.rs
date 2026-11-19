use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tree_sitter::{Node, Parser};

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
    "target",
    ".next",
    ".agents",
    ".claude",
    ".codex",
    ".cursor",
    "vendor",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatedUnit {
    pub file: String,
    pub symbol: String,
    pub start_line: usize,
    pub end_line: usize,
    pub score: f64,
    pub matched_terms: Vec<String>,
    pub numbered_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocateReport {
    pub query: String,
    pub elapsed_ms: f64,
    pub scanned_units: usize,
    pub hits: Vec<LocatedUnit>,
}

struct RawUnit {
    file: String,
    symbol: String,
    start_line: usize,
    end_line: usize,
    name_tokens: Vec<String>,
    body_tokens: Vec<String>,
    lines: Vec<String>,
}

fn tokenize_code_and_query(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let flush_word = |w: &mut String, dst: &mut Vec<String>| {
        if w.len() >= 2 {
            let low = w.to_ascii_lowercase();
            if !is_stopword(&low) {
                dst.push(low);
            }
        }
        w.clear();
    };
    let chars: Vec<char> = text.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ('\u{4E00}'..='\u{9FFF}').contains(&ch) {
            flush_word(&mut word, &mut out);
            if i + 1 < chars.len() && ('\u{4E00}'..='\u{9FFF}').contains(&chars[i + 1]) {
                out.push(format!("{}{}", ch, chars[i + 1]));
            } else {
                out.push(ch.to_string());
            }
        } else if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !word.is_empty() {
                let prev_lower = word
                    .chars()
                    .last()
                    .map(|c| c.is_ascii_lowercase())
                    .unwrap_or(false);
                if prev_lower {
                    flush_word(&mut word, &mut out);
                }
            }
            word.push(ch);
        } else {
            flush_word(&mut word, &mut out);
        }
    }
    flush_word(&mut word, &mut out);
    out
}

fn is_stopword(s: &str) -> bool {
    matches!(
        s,
        "the"
            | "and"
            | "for"
            | "with"
            | "from"
            | "this"
            | "that"
            | "into"
            | "when"
            | "where"
            | "return"
            | "const"
            | "let"
            | "var"
            | "function"
            | "export"
            | "import"
            | "async"
            | "await"
            | "true"
            | "false"
            | "null"
            | "none"
            | "self"
            | "pub"
            | "def"
            | "class"
            | "error"
            | "bug"
            | "fix"
            | "issue"
    )
}

fn parser_for_ext(ext: &str) -> Option<Parser> {
    let mut p = Parser::new();
    let lang = match ext {
        "py" => tree_sitter_python::LANGUAGE.into(),
        "ts" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "tsx" => tree_sitter_typescript::LANGUAGE_TSX.into(),
        "js" | "jsx" | "mjs" | "cjs" => tree_sitter_javascript::LANGUAGE.into(),
        "go" => tree_sitter_go::LANGUAGE.into(),
        "rs" => tree_sitter_rust::LANGUAGE.into(),
        "c" | "h" => tree_sitter_c::LANGUAGE.into(),
        _ => return None,
    };
    p.set_language(&lang).ok()?;
    Some(p)
}

fn node_name(node: Node, src: &[u8]) -> Option<String> {
    if let Some(n) = node.child_by_field_name("name") {
        if let Ok(s) = n.utf8_text(src) {
            return Some(s.trim().to_string());
        }
    }
    if let Some(d) = node.child_by_field_name("declarator") {
        return node_name(d, src).or_else(|| d.utf8_text(src).ok().map(str::to_string));
    }
    None
}

fn extract_units_from_tree(
    node: Node,
    src: &str,
    rel: &str,
    src_lines: &[&str],
    out: &mut Vec<RawUnit>,
) {
    let kind = node.kind();
    let is_fn = matches!(
        kind,
        "function_definition"
            | "function_declaration"
            | "method_definition"
            | "function_item"
            | "method_declaration"
    );
    if is_fn {
        if let Some(name) = node_name(node, src.as_bytes()) {
            let s_line = node.start_position().row + 1;
            let e_line = (node.end_position().row + 1).min(src_lines.len());
            if e_line >= s_line {
                let body = &src[node.start_byte()..node.end_byte().min(src.len())];
                let mut name_tokens = tokenize_code_and_query(&name);
                name_tokens.extend(tokenize_code_and_query(rel));
                let body_tokens = tokenize_code_and_query(body);
                let lines = src_lines[s_line - 1..e_line]
                    .iter()
                    .map(|l| (*l).to_string())
                    .collect();
                out.push(RawUnit {
                    file: rel.to_string(),
                    symbol: name,
                    start_line: s_line,
                    end_line: e_line,
                    name_tokens,
                    body_tokens,
                    lines,
                });
            }
        }
    }
    let mut cur = node.walk();
    for ch in node.children(&mut cur) {
        extract_units_from_tree(ch, src, rel, src_lines, out);
    }
}

pub fn locate_faults(root: &Path, query: &str, top_k: usize) -> LocateReport {
    let t0 = Instant::now();
    let q_tokens_raw = tokenize_code_and_query(query);
    let q_set: HashSet<String> = q_tokens_raw.into_iter().collect();
    let q_low = query.to_ascii_lowercase();
    let wants_tests = q_low.contains("test") || q_low.contains("spec");

    let mut units = Vec::new();
    let walker = WalkBuilder::new(root)
        .git_ignore(true)
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let nm = e.file_name().to_string_lossy();
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                return !SKIP_DIRS.contains(&nm.as_ref());
            }
            true
        })
        .build();

    for entry in walker.flatten() {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let p = entry.path();
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
        let Some(mut parser) = parser_for_ext(ext) else {
            continue;
        };
        let rel = p
            .strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        let rel_low = rel.to_ascii_lowercase();
        if !wants_tests
            && (rel_low.starts_with("test/")
                || rel_low.starts_with("tests/")
                || rel_low.contains(".test.")
                || rel_low.contains(".spec.")
                || rel_low.contains("_test."))
        {
            continue;
        }
        let Ok(src) = fs::read_to_string(p) else {
            continue;
        };
        let Some(tree) = parser.parse(&src, None) else {
            continue;
        };
        let src_lines: Vec<&str> = src.lines().collect();
        let before = units.len();
        extract_units_from_tree(tree.root_node(), &src, &rel, &src_lines, &mut units);
        if units.len() == before && !src_lines.is_empty() {
            units.push(RawUnit {
                file: rel.clone(),
                symbol: "<module>".to_string(),
                start_line: 1,
                end_line: src_lines.len().min(60),
                name_tokens: tokenize_code_and_query(&rel),
                body_tokens: tokenize_code_and_query(&src),
                lines: src_lines
                    .iter()
                    .take(60)
                    .map(|l| (*l).to_string())
                    .collect(),
            });
        }
    }

    let n_docs = units.len().max(1) as f64;
    let avg_dl =
        (units.iter().map(|u| u.body_tokens.len()).sum::<usize>() as f64 / n_docs).max(1.0);
    let mut df: HashMap<String, usize> = HashMap::new();
    for u in &units {
        let uniq: HashSet<&String> = u.name_tokens.iter().chain(&u.body_tokens).collect();
        for tok in uniq {
            if q_set.contains(tok) {
                *df.entry(tok.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut scored: Vec<(f64, Vec<String>, &RawUnit)> = Vec::new();
    for u in &units {
        let mut tf_name: HashMap<&str, usize> = HashMap::new();
        let mut tf_body: HashMap<&str, usize> = HashMap::new();
        for t in &u.name_tokens {
            *tf_name.entry(t.as_str()).or_insert(0) += 1;
        }
        for t in &u.body_tokens {
            *tf_body.entry(t.as_str()).or_insert(0) += 1;
        }
        let dl = u.body_tokens.len().max(1) as f64;
        let mut score = 0.0f64;
        let mut matched = Vec::new();
        for q in &q_set {
            let n_cnt = *tf_name.get(q.as_str()).unwrap_or(&0) as f64;
            let b_cnt = *tf_body.get(q.as_str()).unwrap_or(&0) as f64;
            let tf = n_cnt * 3.0 + b_cnt;
            if tf > 0.0 {
                let d_f = *df.get(q).unwrap_or(&1) as f64;
                let idf = ((n_docs - d_f + 0.5) / (d_f + 0.5) + 1.0).ln();
                let bm25 = idf * (tf * 2.5) / (tf + 1.5 * (0.25 + 0.75 * dl / avg_dl));
                score += bm25;
                matched.push(q.clone());
            }
        }
        if !u.symbol.starts_with('<') && q_low.contains(&u.symbol.to_ascii_lowercase()) {
            score += 12.0;
        }
        if score > 0.0 {
            matched.sort();
            scored.push((score, matched, u));
        }
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let hits = scored
        .into_iter()
        .take(top_k.clamp(1, 10))
        .map(|(sc, mt, u)| {
            let max_show = u.lines.len().min(32);
            let mut numbered = (0..max_show)
                .map(|i| format!("{:4} | {}", u.start_line + i, u.lines[i]))
                .collect::<Vec<_>>()
                .join("\n");
            if u.lines.len() > max_show {
                numbered.push_str(&format!(
                    "\n  ... ({} more lines)",
                    u.lines.len() - max_show
                ));
            }
            LocatedUnit {
                file: u.file.clone(),
                symbol: u.symbol.clone(),
                start_line: u.start_line,
                end_line: u.end_line,
                score: (sc * 100.0).round() / 100.0,
                matched_terms: mt,
                numbered_source: numbered,
            }
        })
        .collect();

    LocateReport {
        query: query.to_string(),
        elapsed_ms: (t0.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0,
        scanned_units: units.len(),
        hits,
    }
}

pub fn render_locate_md(rep: &LocateReport) -> String {
    let mut out = vec![format!(
        "# HIERARCHICAL FAULT LOCALIZER (`{}` | {} functions ranked in {}ms)",
        rep.query, rep.scanned_units, rep.elapsed_ms
    )];
    if rep.hits.is_empty() {
        out.push("*(no matching functions — try keywords or error symbol names)*".to_string());
    } else {
        for (idx, h) in rep.hits.iter().enumerate() {
            out.push(format!(
                "\n## #{}: `{}` :: `{}` (L{}-L{}, score={}, matched={:?})\n```text\n{}\n```",
                idx + 1,
                h.file,
                h.symbol,
                h.start_line,
                h.end_line,
                h.score,
                h.matched_terms,
                h.numbered_source
            ));
        }
    }
    format!("{}\n", out.join("\n"))
}
