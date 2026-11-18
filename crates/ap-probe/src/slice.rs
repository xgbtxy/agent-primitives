use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Instant;
use tree_sitter::{Language, Node, Parser};

const SKIP_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    "env",
    "dist",
    "build",
    "target",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".sieve",
    ".baton",
    ".recall",
    ".guard",
    ".probe",
    ".splice",
    ".agents",
    ".claude",
    ".grok",
    ".cursor",
    ".codex",
    ".openclaw",
    ".hermes",
    "vendor",
    "coverage",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachableModule {
    pub path: String,
    pub imports: Vec<String>,
    pub signatures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlicePayload {
    pub language: String,
    pub elapsed_ms: f64,
    pub total_code_files: usize,
    pub total_py_files: usize,
    pub reachable_count: usize,
    pub pruned_unreferenced_count: usize,
    pub pruned_files: Vec<String>,
    pub reachable_modules: Vec<ReachableModule>,
}

#[allow(clippy::type_complexity)]
fn slice_regexes() -> &'static (Regex, Regex, Regex, Regex, Regex, Regex, Regex) {
    static RE: OnceLock<(Regex, Regex, Regex, Regex, Regex, Regex, Regex)> = OnceLock::new();
    RE.get_or_init(|| {
        let ts_imp = Regex::new(
            r#"(?m)(?:import|export)\s+(?:type\s+)?(?:[^'"]*?\s+from\s+)?['"](\.[^'"]+)['"]|require\(\s*['"](\.[^'"]+)['"]\s*\)|import\(\s*['"](\.[^'"]+)['"]\s*\)"#,
        )
        .unwrap();
        let py_from = Regex::new(r"(?m)^\s*from\s+(\.*)\s*([A-Za-z0-9_.]*)\s+import\s+([^\r\n#]+)").unwrap();
        let py_imp = Regex::new(r"(?m)^\s*import\s+([A-Za-z0-9_.,\s]+)").unwrap();
        let go_mod = Regex::new(r"(?m)^\s*module\s+(\S+)").unwrap();
        let rs_mod = Regex::new(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;").unwrap();
        let rs_use = Regex::new(r"\b(?:use\s+crate::|crate::)([A-Za-z_][A-Za-z0-9_]*)").unwrap();
        let c_inc = Regex::new(r#"(?m)^\s*#\s*include\s*"([^"]+)""#).unwrap();
        (ts_imp, py_from, py_imp, go_mod, rs_mod, rs_use, c_inc)
    })
}

fn lang_for_ext(ext: &str) -> Option<Language> {
    match ext {
        "py" => Some(tree_sitter_python::LANGUAGE.into()),
        "ts" | "mts" | "cts" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "js" | "jsx" | "mjs" | "cjs" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "c" | "h" => Some(tree_sitter_c::LANGUAGE.into()),
        _ => None,
    }
}

fn end_ln(n: Node) -> usize {
    let ep = n.end_position();
    if ep.column == 0 && ep.row > n.start_position().row {
        ep.row
    } else {
        ep.row + 1
    }
}

fn first_line_header(n: Node, src: &[u8]) -> String {
    let txt = n.utf8_text(src).unwrap_or("");
    let hdr = txt
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .trim_end_matches('{')
        .trim();
    let capped: String = hdr.chars().take(92).collect();
    if hdr.chars().count() > 92 {
        format!("{capped}...")
    } else {
        capped
    }
}

pub fn extract_signatures(ext: &str, text: &str) -> Vec<String> {
    let Some(lang) = lang_for_ext(ext) else {
        return Vec::new();
    };
    let mut parser = Parser::new();
    let _ = parser.set_language(&lang);
    let Some(tree) = parser.parse(text, None) else {
        return Vec::new();
    };
    let src = text.as_bytes();
    let root = tree.root_node();
    let mut sigs = Vec::new();
    let mut cur = root.walk();

    for child in root.children(&mut cur) {
        if sigs.len() >= 20 {
            break;
        }
        let target = match child.kind() {
            "decorated_definition" => child.child_by_field_name("definition").unwrap_or(child),
            "export_statement" => child.child_by_field_name("declaration").unwrap_or(child),
            _ => child,
        };
        let s1 = child.start_position().row + 1;
        let e1 = end_ln(child);

        match target.kind() {
            "class_definition"
            | "class_declaration"
            | "abstract_class_declaration"
            | "impl_item" => {
                let hdr = first_line_header(target, src);
                let mut m_sigs = Vec::new();
                if let Some(body) = target.child_by_field_name("body") {
                    let mut bc = body.walk();
                    for sub in body.children(&mut bc) {
                        let m_in = if sub.kind() == "decorated_definition" {
                            sub.child_by_field_name("definition").unwrap_or(sub)
                        } else {
                            sub
                        };
                        if matches!(
                            m_in.kind(),
                            "function_definition" | "method_definition" | "function_item"
                        ) && m_sigs.len() < 12
                        {
                            m_sigs.push(format!(
                                "  {} [L{}-L{}]",
                                first_line_header(m_in, src),
                                sub.start_position().row + 1,
                                end_ln(sub)
                            ));
                        }
                    }
                }
                if m_sigs.is_empty() {
                    sigs.push(format!("{hdr} [L{s1}-{e1}]"));
                } else {
                    sigs.push(format!("{hdr} [L{s1}-{e1}]:\n{}", m_sigs.join("\n")));
                }
            }
            "function_definition"
            | "function_declaration"
            | "method_declaration"
            | "interface_declaration"
            | "type_alias_declaration"
            | "enum_declaration"
            | "type_declaration"
            | "function_item"
            | "struct_item"
            | "enum_item"
            | "trait_item"
            | "struct_specifier"
            | "type_definition" => {
                sigs.push(format!("{} [L{s1}-{e1}]", first_line_header(target, src)));
            }
            "lexical_declaration" | "variable_declaration" => {
                let mut dc = target.walk();
                for vd in target.children(&mut dc) {
                    if vd.kind() == "variable_declarator" {
                        if let Some(vn) = vd.child_by_field_name("value") {
                            if matches!(vn.kind(), "arrow_function" | "function_expression") {
                                sigs.push(format!(
                                    "{} [L{s1}-{e1}]",
                                    first_line_header(child, src)
                                ));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    sigs
}

fn format_compact_numbered(raw: &str) -> (String, usize) {
    let lines: Vec<&str> = raw.lines().collect();
    let mut out = Vec::new();
    let mut blank_run = 0usize;
    for (idx, &ln) in lines.iter().enumerate() {
        if ln.trim().is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
        } else {
            blank_run = 0;
        }
        out.push(format!("{:4}: {ln}", idx + 1));
    }
    (out.join("\n"), lines.len())
}

fn is_test_rel(rel_s: &str) -> bool {
    let low = rel_s.to_ascii_lowercase();
    low.starts_with("tests/")
        || low.starts_with("test/")
        || low.starts_with("__tests__/")
        || low.contains(".test.")
        || low.contains(".spec.")
        || low.ends_with("_test.go")
        || low.ends_with("_test.py")
        || low.rsplit('/').next().unwrap_or("").starts_with("test_")
}

fn render_bundle_section(
    root: &Path,
    reachable: &[PathBuf],
    max_bundle_kb: usize,
) -> (Vec<String>, HashSet<String>) {
    let budget = max_bundle_kb.max(1) * 1024;
    let mut used = 0usize;
    let mut sec =
        vec!["\n## Active Reachable Source Bundle (Skip Separate `read_file` Calls)".to_string()];
    let mut bundled: HashSet<String> = HashSet::new();
    let mut skipped: Vec<String> = Vec::new();

    for mf_name in [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "Makefile",
    ] {
        let mf = root.join(mf_name);
        if mf.is_file() {
            if let Ok(txt) = fs::read_to_string(&mf) {
                if txt.lines().count() <= 45 {
                    let (numbered, n_ln) = format_compact_numbered(&txt);
                    if used + numbered.len() <= budget {
                        used += numbered.len();
                        bundled.insert(mf_name.to_string());
                        sec.push(format!(
                            "\n### {mf_name} ({n_ln} lines)\n```\n{numbered}\n```"
                        ));
                    }
                }
            }
            break;
        }
    }

    let mut non_tests = Vec::new();
    let mut tests = Vec::new();
    for f in reachable {
        let rel = f
            .strip_prefix(root)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        if is_test_rel(&rel) {
            tests.push((f, rel));
        } else {
            non_tests.push((f, rel));
        }
    }
    non_tests.extend(tests);

    for (f, rel_s) in non_tests {
        if f.file_name().and_then(|s| s.to_str()) == Some("__init__.py")
            && f.metadata().map(|m| m.len() == 0).unwrap_or(false)
        {
            bundled.insert(rel_s);
            continue;
        }
        let Ok(bytes) = fs::read(f) else { continue };
        let raw = String::from_utf8_lossy(&bytes);
        let (numbered, n_ln) = format_compact_numbered(&raw);
        if used + numbered.len() > budget {
            skipped.push(rel_s);
            continue;
        }
        used += numbered.len();
        bundled.insert(rel_s.clone());
        sec.push(format!(
            "\n### {rel_s} ({n_ln} lines)\n```\n{numbered}\n```"
        ));
    }

    if !skipped.is_empty() {
        let preview = skipped
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        sec.push(format!(
            "\n[bundle budget {max_bundle_kb} KB reached — {} unbundled file(s): {preview}; use `ap splice view <file> --symbol <Sym>` on demand]",
            skipped.len()
        ));
    }
    (sec, bundled)
}

fn resolve_ts_import(from_file: &Path, spec: &str, all_set: &HashSet<PathBuf>) -> Option<PathBuf> {
    let base = from_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(spec);
    let mut cands = vec![base.clone()];
    if let Some(ext) = base.extension().and_then(|e| e.to_str()) {
        if matches!(ext, "js" | "mjs" | "cjs") {
            let stem = base.with_extension("");
            cands.push(stem.with_extension("ts"));
            cands.push(stem.with_extension("tsx"));
        }
    } else {
        for ext in ["ts", "tsx", "js", "jsx", "mjs"] {
            cands.push(PathBuf::from(format!("{}.{ext}", base.display())));
            cands.push(base.join(format!("index.{ext}")));
        }
    }
    for c in cands {
        if let Ok(rc) = fs::canonicalize(&c) {
            if all_set.contains(&rc) {
                return Some(rc);
            }
        }
    }
    None
}

pub fn compute_slice(root: &Path, entries: &[String]) -> SlicePayload {
    run_slice_inner(root, entries, false, false, 18, false).0
}

pub fn run_slice(
    root: &Path,
    entries: &[String],
    as_json: bool,
    bundle: bool,
    max_bundle_kb: usize,
) -> (SlicePayload, i32) {
    run_slice_inner(root, entries, as_json, bundle, max_bundle_kb, true)
}

fn run_slice_inner(
    root: &Path,
    entries: &[String],
    as_json: bool,
    bundle: bool,
    max_bundle_kb: usize,
    emit_stdout: bool,
) -> (SlicePayload, i32) {
    let t0 = Instant::now();
    let (ts_imp_re, py_from_re, py_imp_re, go_mod_re, rs_mod_re, rs_use_re, c_inc_re) =
        slice_regexes();
    let root_canon = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());

    let mut files: Vec<PathBuf> = Vec::new();
    let walker = WalkBuilder::new(&root_canon)
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let nm = e.file_name().to_string_lossy();
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                return !SKIP_DIRS.contains(&nm.as_ref()) && !nm.starts_with(".git");
            }
            true
        })
        .build();

    for entry in walker.flatten() {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let p = entry.path().to_path_buf();
        let fname = entry.file_name().to_string_lossy();
        if fname.ends_with(".d.ts") {
            continue;
        }
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if matches!(
            ext.as_str(),
            "py" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "go" | "rs" | "c" | "h"
        ) {
            files.push(p);
        }
    }
    files.sort();

    let all_set: HashSet<PathBuf> = files.iter().cloned().collect();
    let mut py_mod_to_file: HashMap<String, PathBuf> = HashMap::new();
    let mut file_to_py_mod: HashMap<PathBuf, String> = HashMap::new();
    let mut py_count = 0usize;

    for f in &files {
        if f.extension().and_then(|e| e.to_str()) == Some("py") {
            py_count += 1;
            if let Ok(rel) = f.strip_prefix(&root_canon) {
                let mut parts: Vec<String> = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect();
                if parts.first().map(String::as_str) == Some("src") && parts.len() > 1 {
                    parts.remove(0);
                }
                let mname = if parts.last().map(String::as_str) == Some("__init__.py") {
                    parts[..parts.len() - 1].join(".")
                } else if let Some(last) = parts.last_mut() {
                    *last = last.trim_end_matches(".py").to_string();
                    parts.join(".")
                } else {
                    String::new()
                };
                if !mname.is_empty() {
                    py_mod_to_file.insert(mname.clone(), f.clone());
                    file_to_py_mod.insert(f.clone(), mname);
                }
            }
        }
    }

    let go_mod_name = fs::read_to_string(root_canon.join("go.mod"))
        .ok()
        .and_then(|s| {
            go_mod_re
                .captures(&s)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
        })
        .unwrap_or_default();

    let mut imports_graph: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut signatures: HashMap<PathBuf, Vec<String>> = HashMap::new();

    for f in &files {
        let txt = String::from_utf8_lossy(&fs::read(f).unwrap_or_default()).to_string();
        let ext = f
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        signatures.insert(f.clone(), extract_signatures(&ext, &txt));
        let mut edges: Vec<PathBuf> = Vec::new();
        let mut push_edge = |tgt: PathBuf| {
            if tgt != *f && !edges.contains(&tgt) {
                edges.push(tgt);
            }
        };

        match ext.as_str() {
            "py" => {
                let cur_mod = file_to_py_mod.get(f).cloned().unwrap_or_default();
                let pkg_parts: Vec<&str> =
                    if f.file_name().and_then(|s| s.to_str()) == Some("__init__.py") {
                        cur_mod.split('.').filter(|s| !s.is_empty()).collect()
                    } else {
                        let v: Vec<&str> = cur_mod.split('.').filter(|s| !s.is_empty()).collect();
                        v[..v.len().saturating_sub(1)].to_vec()
                    };
                for cap in py_from_re.captures_iter(&txt) {
                    let dots = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let base = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                    let names_csv = cap.get(3).map(|m| m.as_str()).unwrap_or("");
                    let full_base = if !dots.is_empty() {
                        let up = dots.len().saturating_sub(1);
                        if up <= pkg_parts.len() {
                            let mut segs: Vec<&str> = pkg_parts[..pkg_parts.len() - up].to_vec();
                            if !base.is_empty() {
                                segs.push(base);
                            }
                            segs.join(".")
                        } else {
                            String::new()
                        }
                    } else {
                        base.to_string()
                    };
                    if !full_base.is_empty() {
                        if let Some(tf) = py_mod_to_file.get(&full_base) {
                            push_edge(tf.clone());
                        }
                        for raw_nm in names_csv.split(',') {
                            let nm = raw_nm.split_whitespace().next().unwrap_or("");
                            if !nm.is_empty() && nm != "*" {
                                let sub_m = format!("{full_base}.{nm}");
                                if let Some(tf) = py_mod_to_file.get(&sub_m) {
                                    push_edge(tf.clone());
                                }
                            }
                        }
                    }
                }
                for cap in py_imp_re.captures_iter(&txt) {
                    for raw_m in cap[1].split(',') {
                        let m = raw_m.split_whitespace().next().unwrap_or("");
                        if let Some(tf) = py_mod_to_file.get(m) {
                            push_edge(tf.clone());
                        }
                    }
                }
            }
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => {
                for cap in ts_imp_re.captures_iter(&txt) {
                    let spec = cap
                        .get(1)
                        .or_else(|| cap.get(2))
                        .or_else(|| cap.get(3))
                        .map(|m| m.as_str())
                        .unwrap_or("");
                    if !spec.is_empty() {
                        if let Some(tf) = resolve_ts_import(f, spec, &all_set) {
                            push_edge(tf);
                        }
                    }
                }
            }
            "go" => {
                if !go_mod_name.is_empty() && txt.contains(&go_mod_name) {
                    for cand in &files {
                        if cand.extension().and_then(|e| e.to_str()) == Some("go")
                            && !cand.to_string_lossy().ends_with("_test.go")
                        {
                            let rel_dir = cand
                                .parent()
                                .and_then(|p| p.strip_prefix(&root_canon).ok())
                                .map(|r| r.to_string_lossy().replace('\\', "/"))
                                .unwrap_or_default();
                            let pkg_spec = if rel_dir.is_empty() {
                                go_mod_name.clone()
                            } else {
                                format!("{go_mod_name}/{rel_dir}")
                            };
                            if txt.contains(&format!("\"{pkg_spec}\"")) {
                                push_edge(cand.clone());
                            }
                        }
                    }
                }
                for sib in &files {
                    if sib.parent() == f.parent()
                        && sib.extension().and_then(|e| e.to_str()) == Some("go")
                        && !sib.to_string_lossy().ends_with("_test.go")
                    {
                        push_edge(sib.clone());
                    }
                }
            }
            "rs" => {
                let mut mods: HashSet<String> = HashSet::new();
                for cap in rs_mod_re.captures_iter(&txt) {
                    mods.insert(cap[1].to_string());
                }
                for cap in rs_use_re.captures_iter(&txt) {
                    mods.insert(cap[1].to_string());
                }
                let parent = f.parent().unwrap_or(&root_canon);
                for mn in mods {
                    for cand in [
                        parent.join(format!("{mn}.rs")),
                        parent.join(&mn).join("mod.rs"),
                        root_canon.join("src").join(format!("{mn}.rs")),
                        root_canon.join("src").join(&mn).join("mod.rs"),
                    ] {
                        if let Ok(rc) = fs::canonicalize(&cand) {
                            if all_set.contains(&rc) {
                                push_edge(rc);
                            }
                        }
                    }
                }
            }
            "c" | "h" => {
                let parent = f.parent().unwrap_or(&root_canon);
                for cap in c_inc_re.captures_iter(&txt) {
                    let inc = &cap[1];
                    for base_dir in [
                        parent,
                        root_canon.as_path(),
                        root_canon.join("include").as_path(),
                    ] {
                        let hp = base_dir.join(inc);
                        if let Ok(rc_h) = fs::canonicalize(&hp) {
                            if all_set.contains(&rc_h) {
                                push_edge(rc_h.clone());
                                let impl_c = rc_h.with_extension("c");
                                if all_set.contains(&impl_c) {
                                    push_edge(impl_c);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        imports_graph.insert(f.clone(), edges);
    }

    let mut seeds: Vec<PathBuf> = Vec::new();
    let entries_clean: Vec<&str> = entries
        .iter()
        .map(String::as_str)
        .filter(|e| *e != "." && !e.trim().is_empty())
        .collect();
    for e in entries_clean {
        let cand = root_canon.join(e);
        if let Ok(rc) = fs::canonicalize(&cand) {
            if rc.is_file() && all_set.contains(&rc) {
                seeds.push(rc);
                continue;
            }
            if rc.is_dir() {
                let mut found_dir_ep = false;
                for ep in [
                    "__init__.py",
                    "main.py",
                    "index.ts",
                    "index.js",
                    "main.ts",
                    "lib.rs",
                    "main.rs",
                    "mod.rs",
                    "main.go",
                    "main.c",
                ] {
                    if let Ok(ec) = fs::canonicalize(rc.join(ep)) {
                        if all_set.contains(&ec) {
                            seeds.push(ec);
                            found_dir_ep = true;
                        }
                    }
                }
                if !found_dir_ep {
                    for f in &files {
                        if f.starts_with(&rc) {
                            seeds.push(f.clone());
                        }
                    }
                }
                continue;
            }
        }
        if let Some(py_f) = py_mod_to_file.get(e) {
            seeds.push(py_f.clone());
            continue;
        }
        for f in &files {
            let rel_s = f
                .strip_prefix(&root_canon)
                .unwrap_or(f)
                .to_string_lossy()
                .replace('\\', "/");
            if rel_s.contains(e) {
                seeds.push(f.clone());
            }
        }
    }

    if seeds.is_empty() {
        for f in &files {
            let rel_s = f
                .strip_prefix(&root_canon)
                .unwrap_or(f)
                .to_string_lossy()
                .replace('\\', "/");
            let fname = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if is_test_rel(&rel_s)
                || matches!(
                    fname,
                    "main.py"
                        | "app.py"
                        | "index.ts"
                        | "index.js"
                        | "main.ts"
                        | "server.ts"
                        | "main.go"
                        | "lib.rs"
                        | "main.rs"
                        | "main.c"
                )
            {
                seeds.push(f.clone());
            }
        }
        if seeds.is_empty() {
            if let Some(first) = files.first() {
                seeds.push(first.clone());
            }
        }
    }

    let mut reachable: Vec<PathBuf> = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut queue: VecDeque<PathBuf> = seeds.into_iter().collect();
    while let Some(cur) = queue.pop_front() {
        if !visited.insert(cur.clone()) {
            continue;
        }
        reachable.push(cur.clone());
        if let Some(nxts) = imports_graph.get(&cur) {
            for nxt in nxts {
                if !visited.contains(nxt) {
                    queue.push_back(nxt.clone());
                }
            }
        }
    }

    let pruned: Vec<PathBuf> = files
        .iter()
        .filter(|f| !visited.contains(*f))
        .cloned()
        .collect();
    let mut ext_counts: HashMap<String, usize> = HashMap::new();
    for f in &files {
        let e = f
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("code")
            .to_string();
        *ext_counts.entry(e).or_insert(0) += 1;
    }
    let dom_ext = ext_counts
        .into_iter()
        .max_by_key(|kv| kv.1)
        .map(|kv| kv.0)
        .unwrap_or_else(|| "polyglot".to_string());

    let reach_mods: Vec<ReachableModule> = reachable
        .iter()
        .map(|f| {
            let rel_s = f
                .strip_prefix(&root_canon)
                .unwrap_or(f)
                .to_string_lossy()
                .replace('\\', "/");
            let imps = imports_graph
                .get(f)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|t| {
                    t.strip_prefix(&root_canon)
                        .unwrap_or(&t)
                        .to_string_lossy()
                        .replace('\\', "/")
                })
                .collect();
            ReachableModule {
                path: rel_s,
                imports: imps,
                signatures: signatures.get(f).cloned().unwrap_or_default(),
            }
        })
        .collect();

    let pruned_rel: Vec<String> = pruned
        .iter()
        .map(|f| {
            f.strip_prefix(&root_canon)
                .unwrap_or(f)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;

    let payload = SlicePayload {
        language: dom_ext.clone(),
        elapsed_ms,
        total_code_files: files.len(),
        total_py_files: py_count,
        reachable_count: reachable.len(),
        pruned_unreferenced_count: pruned.len(),
        pruned_files: pruned_rel.clone(),
        reachable_modules: reach_mods.clone(),
    };

    if !emit_stdout {
        return (payload, 0);
    }
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
        return (payload, 0);
    }

    let mut out_lines = vec![format!(
        "[probe slice ({})] {}/{} reachable files ({} dead/unreferenced files pruned) in {elapsed_ms} ms",
        dom_ext.to_uppercase(),
        reachable.len(),
        files.len(),
        pruned.len()
    )];
    if !pruned_rel.is_empty() {
        let preview = pruned_rel
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let ell = if pruned_rel.len() > 12 { ", ..." } else { "" };
        out_lines.push(format!("Pruned (skip reading): {preview}{ell}"));
    }

    let (bundle_sec, bundled_set) = if bundle {
        render_bundle_section(&root_canon, &reachable, max_bundle_kb)
    } else {
        (Vec::new(), HashSet::new())
    };

    let unbundled_mods: Vec<&ReachableModule> = reach_mods
        .iter()
        .filter(|m| !bundled_set.contains(&m.path))
        .collect();
    if !unbundled_mods.is_empty() {
        out_lines.push("\n## Reachable Module Graph & Signatures".to_string());
        for m in unbundled_mods {
            let imp_s = if m.imports.is_empty() {
                String::new()
            } else {
                format!(" -> imports {}", m.imports.join(", "))
            };
            out_lines.push(format!("- `{}`{imp_s}", m.path));
            for sig in &m.signatures {
                for sub in sig.lines() {
                    out_lines.push(format!("    {sub}"));
                }
            }
        }
    }
    if bundle {
        out_lines.extend(bundle_sec);
    }
    println!("{}", out_lines.join("\n"));
    (payload, 0)
}
