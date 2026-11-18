use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub method: String,
    pub path: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPageEntry {
    pub file: String,
    pub title: String,
    pub controls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCallEntry {
    pub method: String,
    pub path: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlopFileEntry {
    pub file: String,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMap {
    pub root: String,
    pub elapsed_ms: f64,
    pub pages: Vec<UiPageEntry>,
    pub backend_routes: Vec<RouteEntry>,
    pub client_calls: Vec<ClientCallEntry>,
    pub contract_mismatches: Vec<String>,
    pub slop_smells: Vec<SlopFileEntry>,
}

struct ProductRegexes {
    fw_route: Regex,
    node_exact: Regex,
    node_regex_var: Regex,
    node_if_match: Regex,
    html_title: Regex,
    html_pair_ctrl: Regex,
    html_input_ctrl: Regex,
    html_attr_id: Regex,
    html_attr_ph: Regex,
    tag_strip: Regex,
    fetch_call: Regex,
    method_opt: Regex,
    any_ts: Regex,
    ts_ignore: Regex,
    empty_catch: Regex,
    py_pass_except: Regex,
    ts_strip_trap: Regex,
    dom_get_id: Regex,
    js_dyn_id: Regex,
    param_re: Regex,
}

fn pre() -> &'static ProductRegexes {
    static R: OnceLock<ProductRegexes> = OnceLock::new();
    R.get_or_init(|| ProductRegexes {
        fw_route: Regex::new(r#"(?i)\b(?:app|router|server|api|bp)\s*\.\s*(get|post|put|patch|delete)\s*\(\s*['"`](/[^'"`\s)]*)['"`]"#).unwrap(),
        node_exact: Regex::new(r#"method\s*===\s*['"]([A-Z]+)['"]\s*&&\s*(?:pathname|url|path)\s*===\s*['"](/[^'"]+)['"]"#).unwrap(),
        node_regex_var: Regex::new(r#"const\s+([A-Za-z0-9_]+)\s*=\s*(?:pathname|url|path)\.match\(\s*/\^(\\\/[^$]+)\$/"#).unwrap(),
        node_if_match: Regex::new(r#"method\s*===\s*['"]([A-Z]+)['"]\s*&&\s*([A-Za-z0-9_]+)"#).unwrap(),
        html_title: Regex::new(r"(?is)<title[^>]*>(.*?)</title>").unwrap(),
        html_pair_ctrl: Regex::new(r"(?is)<(button|select|textarea|form)\b([^>]*)>(.*?)</(?:button|select|textarea|form)>").unwrap(),
        html_input_ctrl: Regex::new(r"(?is)<input\b([^>]*)/?>").unwrap(),
        html_attr_id: Regex::new(r#"(?i)\bid\s*=\s*["']([^"']+)["']"#).unwrap(),
        html_attr_ph: Regex::new(r#"(?i)\bplaceholder\s*=\s*["']([^"']+)["']"#).unwrap(),
        tag_strip: Regex::new(r"<[^>]+>").unwrap(),
        fetch_call: Regex::new(r#"fetch\s*\(\s*['"`](/api/[^'"`?$\s]*)(?:\$\{[^}]+\}(/[^'"`?\s]*))?"#).unwrap(),
        method_opt: Regex::new(r#"(?i)\bmethod\s*:\s*['"]([A-Z]+)['"]"#).unwrap(),
        any_ts: Regex::new(r"(:\s*any\b|\bas\s+any\b|<any>)").unwrap(),
        ts_ignore: Regex::new(r"(@ts-ignore|@ts-nocheck|#\s*type:\s*ignore)").unwrap(),
        empty_catch: Regex::new(r"catch\s*(?:\([^)]*\))?\s*\{\s*(?://[^\n]*\s*)?\}").unwrap(),
        py_pass_except: Regex::new(r"(?m)^\s*except\b[^:]*:\s*\n\s*pass\b").unwrap(),
        ts_strip_trap: Regex::new(r"(\benum\s+[A-Za-z_]\w*|\bconstructor\s*\([^)]*\b(?:private|public|protected|readonly)\s+[A-Za-z_])").unwrap(),
        dom_get_id: Regex::new(r#"(?:getElementById\s*\(\s*['"]([A-Za-z0-9_-]+)['"]\s*\)|querySelector\s*\(\s*['"]#([A-Za-z0-9_-]+)['"]\s*\))"#).unwrap(),
        js_dyn_id: Regex::new(r#"(?:\.id\s*=\s*['"`]([A-Za-z0-9_-]+)['"`]|\bid\s*=\s*["']([A-Za-z0-9_-]+)["'])"#).unwrap(),
        param_re: Regex::new(r"\([^)]+\)|\{[^}]+\}|\$\{[^}]+\}|:[A-Za-z0-9_]+").unwrap(),
    })
}

fn norm_route_pattern(raw: &str) -> String {
    let unescaped = raw.replace("\\/", "/");
    let mut s = pre().param_re.replace_all(&unescaped, ":param").to_string();
    if s.len() > 1 && s.ends_with('/') {
        s.pop();
    }
    s
}

fn routes_match(client_m: &str, client_p: &str, srv_m: &str, srv_p: &str) -> bool {
    if client_m != "ANY" && srv_m != "ANY" && client_m != srv_m {
        return false;
    }
    let c_norm = norm_route_pattern(client_p);
    let s_norm = norm_route_pattern(srv_p);
    let cp: Vec<&str> = c_norm.split('/').collect();
    let sp: Vec<&str> = s_norm.split('/').collect();
    if cp.len() != sp.len() {
        return false;
    }
    cp.iter()
        .zip(sp.iter())
        .all(|(a, b)| a == b || *a == ":param" || *b == ":param")
}

pub fn build_product_map(root: &Path, skip_dirs: &'static [&'static str]) -> ProductMap {
    let t0 = Instant::now();
    let r = pre();
    let mut pages = Vec::new();
    let mut backend_routes = Vec::new();
    let mut client_calls = Vec::new();
    let mut slop_smells = Vec::new();
    let mut declared_dom_ids: BTreeSet<String> = BTreeSet::new();
    let mut referenced_dom_ids: Vec<(String, usize, String)> = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .filter_entry(|e| {
            let nm = e.file_name().to_string_lossy();
            !skip_dirs.iter().any(|s| *s == nm)
        })
        .build();

    for entry in walker.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let Ok(meta) = p.metadata() else { continue };
        if meta.len() > 512_000 {
            continue;
        }
        let Ok(text) = fs::read_to_string(p) else {
            continue;
        };
        let rel = p
            .strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");

        if ext == "html" {
            for cap in r.html_attr_id.captures_iter(&text) {
                declared_dom_ids.insert(cap[1].to_string());
            }
            let title = r
                .html_title
                .captures(&text)
                .map(|c| c[1].trim().to_string())
                .unwrap_or_default();
            let mut controls = Vec::new();
            for cap in r.html_input_ctrl.captures_iter(&text) {
                let attrs = &cap[1];
                if let Some(id_cap) = r.html_attr_id.captures(attrs) {
                    let ph = r
                        .html_attr_ph
                        .captures(attrs)
                        .map(|c| c[1].to_string())
                        .unwrap_or_default();
                    let desc = if ph.is_empty() {
                        format!("#{} <input>", &id_cap[1])
                    } else {
                        format!(
                            "#{} <input> \"{}\"",
                            &id_cap[1],
                            ph.chars().take(28).collect::<String>()
                        )
                    };
                    if !controls.contains(&desc) {
                        controls.push(desc);
                    }
                }
            }
            for cap in r.html_pair_ctrl.captures_iter(&text) {
                let tag = &cap[1];
                let attrs = &cap[2];
                let inner = r
                    .tag_strip
                    .replace_all(&cap[3], " ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                if let Some(id_cap) = r.html_attr_id.captures(attrs) {
                    let desc = if inner.is_empty() {
                        format!("#{} <{tag}>", &id_cap[1])
                    } else {
                        format!(
                            "#{} <{tag}> \"{}\"",
                            &id_cap[1],
                            inner.chars().take(28).collect::<String>()
                        )
                    };
                    if !controls.contains(&desc) {
                        controls.push(desc);
                    }
                }
            }
            pages.push(UiPageEntry {
                file: rel.clone(),
                title,
                controls,
            });
            continue;
        }

        if !matches!(
            ext,
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "py" | "go" | "rs"
        ) {
            continue;
        }

        // Slop hygiene scan (skip test files)
        if !rel.starts_with("tests/") && !rel.contains(".test.") && !rel.contains("_test.") {
            let mut smells = Vec::new();
            let any_cnt = r.any_ts.find_iter(&text).count();
            if any_cnt > 0 && matches!(ext, "ts" | "tsx") {
                smells.push(format!("{any_cnt}x `any` type"));
            }
            let ign_cnt = r.ts_ignore.find_iter(&text).count();
            if ign_cnt > 0 {
                smells.push(format!("{ign_cnt}x `@ts-ignore`/`type:ignore`"));
            }
            let catch_cnt =
                r.empty_catch.find_iter(&text).count() + r.py_pass_except.find_iter(&text).count();
            if catch_cnt > 0 {
                smells.push(format!("{catch_cnt}x silent empty `catch`/swallowed error"));
            }
            if matches!(ext, "ts" | "tsx") && r.ts_strip_trap.is_match(&text) {
                smells.push("Node `--experimental-strip-types` hazard (`enum` or constructor parameter property)".to_string());
            }
            if !smells.is_empty() {
                slop_smells.push(SlopFileEntry {
                    file: rel.clone(),
                    issues: smells,
                });
            }
        }

        if matches!(ext, "js" | "jsx" | "mjs" | "ts" | "tsx") {
            for cap in r.js_dyn_id.captures_iter(&text) {
                if let Some(m) = cap.get(1).or_else(|| cap.get(2)) {
                    declared_dom_ids.insert(m.as_str().to_string());
                }
            }
            if text.contains("document.") && !rel.starts_with("tests/") {
                for cap in r.dom_get_id.captures_iter(&text) {
                    if let Some(m) = cap.get(1).or_else(|| cap.get(2)) {
                        let line = text[..m.start()].bytes().filter(|b| *b == b'\n').count() + 1;
                        referenced_dom_ids.push((rel.clone(), line, m.as_str().to_string()));
                    }
                }
            }
        }

        // Map regex vars in node:http routers
        let mut regex_vars = Vec::new();
        for cap in r.node_regex_var.captures_iter(&text) {
            regex_vars.push((cap[1].to_string(), norm_route_pattern(&cap[2])));
        }

        for (idx, line) in text.lines().enumerate() {
            let lno = idx + 1;
            for cap in r.fw_route.captures_iter(line) {
                backend_routes.push(RouteEntry {
                    method: cap[1].to_uppercase(),
                    path: norm_route_pattern(&cap[2]),
                    file: rel.clone(),
                    line: lno,
                });
            }
            for cap in r.node_exact.captures_iter(line) {
                backend_routes.push(RouteEntry {
                    method: cap[1].to_uppercase(),
                    path: norm_route_pattern(&cap[2]),
                    file: rel.clone(),
                    line: lno,
                });
            }
            for cap in r.node_if_match.captures_iter(line) {
                let var_nm = &cap[2];
                if let Some((_, pat)) = regex_vars.iter().find(|(v, _)| v == var_nm) {
                    backend_routes.push(RouteEntry {
                        method: cap[1].to_uppercase(),
                        path: pat.clone(),
                        file: rel.clone(),
                        line: lno,
                    });
                }
            }
        }

        // Scan client fetch('/api/...') calls
        for cap in r.fetch_call.captures_iter(&text) {
            let full_match = cap.get(0).unwrap();
            let line = text[..full_match.start()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            let tail_end = (full_match.end() + 220).min(text.len());
            let mut slice_end = tail_end;
            while !text.is_char_boundary(slice_end) {
                slice_end -= 1;
            }
            let window = &text[full_match.start()..slice_end];
            let call_stmt = window.split(");").next().unwrap_or(window);
            let method = r
                .method_opt
                .captures(call_stmt)
                .map(|c| c[1].to_uppercase())
                .unwrap_or_else(|| "GET".to_string());
            let mut pth = cap[1].to_string();
            if let Some(rest) = cap.get(2) {
                pth.push_str(":param");
                pth.push_str(rest.as_str());
            }
            if pth.ends_with('/') && pth.len() > 1 {
                pth.push_str(":param");
            }
            client_calls.push(ClientCallEntry {
                method,
                path: norm_route_pattern(&pth),
                file: rel.clone(),
                line,
            });
        }
    }

    let mut mismatches = BTreeSet::new();
    if !backend_routes.is_empty() {
        for c in &client_calls {
            let matched = backend_routes
                .iter()
                .any(|b| routes_match(&c.method, &c.path, &b.method, &b.path));
            if !matched {
                mismatches.insert(format!(
                    "{}:{} calls `{} {}` -> NO matching backend route registered!",
                    c.file, c.line, c.method, c.path
                ));
            }
        }
    }

    if !declared_dom_ids.is_empty() {
        for (file, line, id_nm) in &referenced_dom_ids {
            if !declared_dom_ids.contains(id_nm) {
                mismatches.insert(format!(
                    "{file}:{line} references DOM `#{id_nm}` -> NO matching `id=\"{id_nm}\"` in HTML!"
                ));
            }
        }
    }

    ProductMap {
        root: root.to_string_lossy().replace('\\', "/"),
        elapsed_ms: (t0.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0,
        pages,
        backend_routes,
        client_calls,
        contract_mismatches: mismatches.into_iter().collect(),
        slop_smells,
    }
}

pub fn render_product_map(pm: &ProductMap) -> String {
    let mut lines = vec![format!(
        "# Product Feature & Contract Map (`{}` | {:.1}ms)",
        pm.root, pm.elapsed_ms
    )];

    if !pm.pages.is_empty() {
        lines.push("\n## 1. Frontend Pages & Interactive Controls".to_string());
        for pg in &pm.pages {
            let t = if pg.title.is_empty() {
                pg.file.clone()
            } else {
                format!("{} (`{}`)", pg.title, pg.file)
            };
            lines.push(format!("- **{t}**"));
            if !pg.controls.is_empty() {
                lines.push(format!("  - Controls: {}", pg.controls.join(" · ")));
            }
        }
    }

    if !pm.backend_routes.is_empty() {
        lines.push("\n## 2. Registered Backend API Routes".to_string());
        for rt in &pm.backend_routes {
            lines.push(format!(
                "- `{} {}`  ({}:{})",
                rt.method, rt.path, rt.file, rt.line
            ));
        }
    }

    if !pm.client_calls.is_empty() {
        lines.push("\n## 3. Frontend -> Backend API Wiring".to_string());
        for c in &pm.client_calls {
            lines.push(format!(
                "- `{} {}`  (called at {}:{})",
                c.method, c.path, c.file, c.line
            ));
        }
    }

    if !pm.contract_mismatches.is_empty() {
        lines.push("\n## 4. [CONTRACT MISMATCHES] Broken Frontend <-> Backend Links".to_string());
        for m in &pm.contract_mismatches {
            lines.push(format!("- [BROKEN] {m}"));
        }
    }

    if !pm.slop_smells.is_empty() {
        lines.push(
            "\n## 5. [SLOP RADAR] Code Hygiene Warnings (Do NOT Copy These Patterns)".to_string(),
        );
        for s in &pm.slop_smells {
            lines.push(format!("- `{}` -> {}", s.file, s.issues.join(", ")));
        }
    }

    format!("{}\n", lines.join("\n"))
}
