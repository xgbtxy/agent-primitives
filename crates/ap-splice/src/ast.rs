use serde::{Deserialize, Serialize};
use tree_sitter::{Language, Node, Parser};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSpan {
    pub qual: String,
    pub short: String,
    pub kind: String,
    pub start_line: usize,
    pub end_line: usize,
    pub indent_col: usize,
}

pub fn language_for_ext(ext: &str) -> Option<Language> {
    match ext.to_ascii_lowercase().as_str() {
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

fn find_first_error(node: Node) -> Option<Node> {
    if node.is_error() || node.is_missing() {
        return Some(node);
    }
    if !node.has_error() {
        return None;
    }
    let mut cur = node.walk();
    for ch in node.children(&mut cur) {
        if let Some(err) = find_first_error(ch) {
            return Some(err);
        }
    }
    Some(node)
}

pub fn verify_syntax(source: &str, ext: &str) -> Result<(), String> {
    let ext_l = ext.trim_start_matches('.').to_ascii_lowercase();
    if ext_l == "json" {
        return serde_json::from_str::<serde_json::Value>(source)
            .map(|_| ())
            .map_err(|e| format!("Invalid JSON: {e}"));
    }
    let Some(lang) = language_for_ext(&ext_l) else {
        return Ok(());
    };
    let mut parser = Parser::new();
    let _ = parser.set_language(&lang);
    let Some(tree) = parser.parse(source, None) else {
        return Ok(());
    };
    let root = tree.root_node();
    if root.has_error() {
        if let Some(err_n) = find_first_error(root) {
            let pos = err_n.start_position();
            let snippet: String = err_n
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .chars()
                .take(60)
                .collect();
            return Err(format!(
                "SyntaxError at L{}:{} near `{}`",
                pos.row + 1,
                pos.column + 1,
                snippet
            ));
        }
        return Err("SyntaxError detected in AST".to_string());
    }
    Ok(())
}

fn node_text<'a>(n: Node<'a>, src: &'a [u8]) -> &'a str {
    n.utf8_text(src).unwrap_or("")
}

fn line_indent(lines: &[&str], zero_idx: usize) -> usize {
    lines
        .get(zero_idx)
        .map(|l| l.chars().take_while(|c| *c == ' ' || *c == '\t').count())
        .unwrap_or(0)
}

fn expand_leading_attrs(lines: &[&str], mut start_zero: usize) -> usize {
    while start_zero > 0 {
        let prev = lines[start_zero - 1].trim_start();
        if prev.starts_with('@') || prev.starts_with("#[") {
            start_zero -= 1;
        } else {
            break;
        }
    }
    start_zero
}

fn end_line_1idx(node: Node) -> usize {
    let ep = node.end_position();
    if ep.column == 0 && ep.row > node.start_position().row {
        ep.row
    } else {
        ep.row + 1
    }
}

fn find_c_ident<'a>(node: Node<'a>, src: &'a [u8]) -> Option<&'a str> {
    match node.kind() {
        "identifier" | "type_identifier" | "field_identifier" => Some(node_text(node, src)),
        _ => {
            if let Some(d) = node.child_by_field_name("declarator") {
                if let Some(id) = find_c_ident(d, src) {
                    return Some(id);
                }
            }
            let mut c = node.walk();
            for ch in node.children(&mut c) {
                if matches!(ch.kind(), "identifier" | "type_identifier") {
                    return Some(node_text(ch, src));
                }
            }
            None
        }
    }
}

pub fn scan_symbols(source: &str, ext: &str) -> Vec<SymbolSpan> {
    let ext_l = ext.trim_start_matches('.').to_ascii_lowercase();
    let Some(lang) = language_for_ext(&ext_l) else {
        return Vec::new();
    };
    let mut parser = Parser::new();
    let _ = parser.set_language(&lang);
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let src = source.as_bytes();
    let lines: Vec<&str> = source.lines().collect();
    let root = tree.root_node();
    let mut items = Vec::new();

    match ext_l.as_str() {
        "py" => {
            let mut cur = root.walk();
            for child in root.children(&mut cur) {
                let (outer, inner) = if child.kind() == "decorated_definition" {
                    (
                        child,
                        child.child_by_field_name("definition").unwrap_or(child),
                    )
                } else {
                    (child, child)
                };
                match inner.kind() {
                    "function_definition" => {
                        if let Some(nn) = inner.child_by_field_name("name") {
                            let nm = node_text(nn, src).to_string();
                            let s0 = outer.start_position().row;
                            items.push(SymbolSpan {
                                qual: nm.clone(),
                                short: nm,
                                kind: "def".to_string(),
                                start_line: s0 + 1,
                                end_line: end_line_1idx(outer),
                                indent_col: inner.start_position().column,
                            });
                        }
                    }
                    "class_definition" => {
                        let Some(nn) = inner.child_by_field_name("name") else {
                            continue;
                        };
                        let cname = node_text(nn, src).to_string();
                        let s0 = outer.start_position().row;
                        items.push(SymbolSpan {
                            qual: cname.clone(),
                            short: cname.clone(),
                            kind: "class".to_string(),
                            start_line: s0 + 1,
                            end_line: end_line_1idx(outer),
                            indent_col: inner.start_position().column,
                        });
                        if let Some(body) = inner.child_by_field_name("body") {
                            let mut bcur = body.walk();
                            for sub in body.children(&mut bcur) {
                                let (m_outer, m_in) = if sub.kind() == "decorated_definition" {
                                    (sub, sub.child_by_field_name("definition").unwrap_or(sub))
                                } else {
                                    (sub, sub)
                                };
                                if m_in.kind() == "function_definition" {
                                    if let Some(mn) = m_in.child_by_field_name("name") {
                                        let mname = node_text(mn, src).to_string();
                                        items.push(SymbolSpan {
                                            qual: format!("{cname}.{mname}"),
                                            short: mname,
                                            kind: "def".to_string(),
                                            start_line: m_outer.start_position().row + 1,
                                            end_line: end_line_1idx(m_outer),
                                            indent_col: m_in.start_position().column,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs" => {
            let mut cur = root.walk();
            for child in root.children(&mut cur) {
                let (outer, decl) = if child.kind() == "export_statement" {
                    (
                        child,
                        child.child_by_field_name("declaration").unwrap_or(child),
                    )
                } else {
                    (child, child)
                };
                let s0 = expand_leading_attrs(&lines, outer.start_position().row);
                let indent = line_indent(&lines, s0);
                match decl.kind() {
                    "class_declaration" | "abstract_class_declaration" => {
                        let Some(nn) = decl.child_by_field_name("name") else {
                            continue;
                        };
                        let cname = node_text(nn, src).to_string();
                        items.push(SymbolSpan {
                            qual: cname.clone(),
                            short: cname.clone(),
                            kind: "class".to_string(),
                            start_line: s0 + 1,
                            end_line: end_line_1idx(outer),
                            indent_col: indent,
                        });
                        if let Some(body) = decl.child_by_field_name("body") {
                            let mut bcur = body.walk();
                            for sub in body.children(&mut bcur) {
                                if sub.kind() == "method_definition" {
                                    if let Some(mn) = sub.child_by_field_name("name") {
                                        let mname = node_text(mn, src).to_string();
                                        let ms0 =
                                            expand_leading_attrs(&lines, sub.start_position().row);
                                        items.push(SymbolSpan {
                                            qual: format!("{cname}.{mname}"),
                                            short: mname,
                                            kind: "method".to_string(),
                                            start_line: ms0 + 1,
                                            end_line: end_line_1idx(sub),
                                            indent_col: line_indent(&lines, ms0),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    "function_declaration"
                    | "generator_function_declaration"
                    | "interface_declaration"
                    | "type_alias_declaration"
                    | "enum_declaration" => {
                        if let Some(nn) = decl.child_by_field_name("name") {
                            let nm = node_text(nn, src).to_string();
                            let kind = decl.kind().split('_').next().unwrap_or("fn").to_string();
                            items.push(SymbolSpan {
                                qual: nm.clone(),
                                short: nm,
                                kind,
                                start_line: s0 + 1,
                                end_line: end_line_1idx(outer),
                                indent_col: indent,
                            });
                        }
                    }
                    "lexical_declaration" | "variable_declaration" => {
                        let mut dcur = decl.walk();
                        for vd in decl.children(&mut dcur) {
                            if vd.kind() == "variable_declarator" {
                                if let (Some(nn), Some(vn)) = (
                                    vd.child_by_field_name("name"),
                                    vd.child_by_field_name("value"),
                                ) {
                                    if matches!(
                                        vn.kind(),
                                        "arrow_function" | "function_expression" | "function"
                                    ) {
                                        let nm = node_text(nn, src).to_string();
                                        items.push(SymbolSpan {
                                            qual: nm.clone(),
                                            short: nm,
                                            kind: "const_fn".to_string(),
                                            start_line: s0 + 1,
                                            end_line: end_line_1idx(outer),
                                            indent_col: indent,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "go" => {
            let mut cur = root.walk();
            for child in root.children(&mut cur) {
                let s0 = child.start_position().row;
                let indent = line_indent(&lines, s0);
                match child.kind() {
                    "function_declaration" => {
                        if let Some(nn) = child.child_by_field_name("name") {
                            let nm = node_text(nn, src).to_string();
                            items.push(SymbolSpan {
                                qual: nm.clone(),
                                short: nm,
                                kind: "func".to_string(),
                                start_line: s0 + 1,
                                end_line: end_line_1idx(child),
                                indent_col: indent,
                            });
                        }
                    }
                    "method_declaration" => {
                        let Some(nn) = child.child_by_field_name("name") else {
                            continue;
                        };
                        let mname = node_text(nn, src).to_string();
                        let recv = child
                            .child_by_field_name("receiver")
                            .map(|r| {
                                node_text(r, src)
                                    .trim_matches(|c| c == '(' || c == ')')
                                    .split_whitespace()
                                    .last()
                                    .unwrap_or("")
                                    .trim_start_matches('*')
                                    .to_string()
                            })
                            .unwrap_or_default();
                        let qual = if recv.is_empty() {
                            mname.clone()
                        } else {
                            format!("{recv}.{mname}")
                        };
                        items.push(SymbolSpan {
                            qual,
                            short: mname,
                            kind: "func".to_string(),
                            start_line: s0 + 1,
                            end_line: end_line_1idx(child),
                            indent_col: indent,
                        });
                    }
                    "type_declaration" => {
                        let mut tc = child.walk();
                        for sub in child.children(&mut tc) {
                            if sub.kind() == "type_spec" {
                                if let Some(nn) = sub.child_by_field_name("name") {
                                    let nm = node_text(nn, src).to_string();
                                    items.push(SymbolSpan {
                                        qual: nm.clone(),
                                        short: nm,
                                        kind: "type".to_string(),
                                        start_line: s0 + 1,
                                        end_line: end_line_1idx(child),
                                        indent_col: indent,
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "rs" => {
            let mut cur = root.walk();
            for child in root.children(&mut cur) {
                let s0 = expand_leading_attrs(&lines, child.start_position().row);
                let indent = line_indent(&lines, s0);
                match child.kind() {
                    "function_item" | "struct_item" | "enum_item" | "trait_item" | "type_item" => {
                        if let Some(nn) = child.child_by_field_name("name") {
                            let nm = node_text(nn, src).to_string();
                            let kind = child.kind().trim_end_matches("_item").to_string();
                            items.push(SymbolSpan {
                                qual: nm.clone(),
                                short: nm,
                                kind,
                                start_line: s0 + 1,
                                end_line: end_line_1idx(child),
                                indent_col: indent,
                            });
                        }
                    }
                    "impl_item" => {
                        let Some(tn) = child.child_by_field_name("type") else {
                            continue;
                        };
                        let raw_t = node_text(tn, src);
                        let owner = raw_t
                            .split('<')
                            .next()
                            .unwrap_or(raw_t)
                            .rsplit("::")
                            .next()
                            .unwrap_or(raw_t)
                            .trim()
                            .to_string();
                        if let Some(body) = child.child_by_field_name("body") {
                            let mut bc = body.walk();
                            for sub in body.children(&mut bc) {
                                if sub.kind() == "function_item" {
                                    if let Some(mn) = sub.child_by_field_name("name") {
                                        let mname = node_text(mn, src).to_string();
                                        let ms0 =
                                            expand_leading_attrs(&lines, sub.start_position().row);
                                        items.push(SymbolSpan {
                                            qual: format!("{owner}.{mname}"),
                                            short: mname,
                                            kind: "fn".to_string(),
                                            start_line: ms0 + 1,
                                            end_line: end_line_1idx(sub),
                                            indent_col: line_indent(&lines, ms0),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "c" | "h" => {
            let mut cur = root.walk();
            for child in root.children(&mut cur) {
                let s0 = child.start_position().row;
                let indent = line_indent(&lines, s0);
                match child.kind() {
                    "function_definition" => {
                        if let Some(decl) = child.child_by_field_name("declarator") {
                            if let Some(nm) = find_c_ident(decl, src) {
                                items.push(SymbolSpan {
                                    qual: nm.to_string(),
                                    short: nm.to_string(),
                                    kind: "fn".to_string(),
                                    start_line: s0 + 1,
                                    end_line: end_line_1idx(child),
                                    indent_col: indent,
                                });
                            }
                        }
                    }
                    "struct_specifier" | "enum_specifier" => {
                        if let Some(nn) = child.child_by_field_name("name") {
                            let nm = node_text(nn, src).to_string();
                            let kind = child.kind().trim_end_matches("_specifier").to_string();
                            items.push(SymbolSpan {
                                qual: nm.clone(),
                                short: nm,
                                kind,
                                start_line: s0 + 1,
                                end_line: end_line_1idx(child),
                                indent_col: indent,
                            });
                        }
                    }
                    "type_definition" => {
                        if let Some(decl) = child.child_by_field_name("declarator") {
                            if let Some(nm) = find_c_ident(decl, src) {
                                items.push(SymbolSpan {
                                    qual: nm.to_string(),
                                    short: nm.to_string(),
                                    kind: "typedef".to_string(),
                                    start_line: s0 + 1,
                                    end_line: end_line_1idx(child),
                                    indent_col: indent,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    items
}

pub fn find_symbol_span(source: &str, ext: &str, symbol: &str) -> Option<SymbolSpan> {
    let items = scan_symbols(source, ext);
    if let Some(exact) = items.iter().find(|it| it.qual == symbol) {
        return Some(exact.clone());
    }
    let short_hits: Vec<&SymbolSpan> = items.iter().filter(|it| it.short == symbol).collect();
    if short_hits.len() == 1 {
        return Some(short_hits[0].clone());
    }
    None
}
