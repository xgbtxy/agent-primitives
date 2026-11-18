use crate::stats::record_event;
use clap::Parser;
use flate2::read::{DeflateDecoder, ZlibDecoder};
use regex::Regex;
use serde_json::json;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use zip::ZipArchive;

#[derive(Parser, Debug)]
#[command(
    name = "read",
    about = "Read any file (.docx, .xlsx, .pptx, .pdf, .csv, .json, GBK/UTF-16 text) or URL without context overflow or mojibake"
)]
struct ReadCli {
    /// File path or http(s):// URL to read
    target: String,
    /// Optional line range (e.g. `1-120` or `200:350`)
    #[arg(long)]
    lines: Option<String>,
    /// Filter or search rows/paragraphs matching keyword or regex
    #[arg(short = 'q', long)]
    query: Option<String>,
    /// Optional worksheet name filter for .xlsx files
    #[arg(long)]
    sheet: Option<String>,
    /// Maximum characters before smart outline + pagination kicks in
    #[arg(long, default_value_t = 8000)]
    max_chars: usize,
    /// Emit structured JSON output
    #[arg(long)]
    json: bool,
}

struct XmlRegexes {
    xml_tag: Regex,
    docx_block: Regex,
    docx_style: Regex,
    docx_tr: Regex,
    docx_tc: Regex,
    docx_t: Regex,
    xlsx_si: Regex,
    xlsx_sheet: Regex,
    xlsx_row: Regex,
    xlsx_cell: Regex,
    xlsx_v: Regex,
    pptx_p: Regex,
    pptx_t: Regex,
    pdf_tj: Regex,
}

fn xre() -> &'static XmlRegexes {
    static R: OnceLock<XmlRegexes> = OnceLock::new();
    R.get_or_init(|| XmlRegexes {
        xml_tag: Regex::new(r"<[^>]+>").unwrap(),
        docx_block: Regex::new(r"(?s)<w:tbl\b.*?</w:tbl>|<w:p\b.*?</w:p>").unwrap(),
        docx_style: Regex::new(r#"<w:pStyle\s+w:val="([^"]+)""#).unwrap(),
        docx_tr: Regex::new(r"(?s)<w:tr\b.*?</w:tr>").unwrap(),
        docx_tc: Regex::new(r"(?s)<w:tc\b.*?</w:tc>").unwrap(),
        docx_t: Regex::new(r"(?s)<w:t(?:\s[^>]*)?>(.*?)</w:t>").unwrap(),
        xlsx_si: Regex::new(r"(?s)<si\b[^>]*>(.*?)</si>").unwrap(),
        xlsx_sheet: Regex::new(r#"<sheet\b[^>]*\bname="([^"]+)""#).unwrap(),
        xlsx_row: Regex::new(r"(?s)<row\b[^>]*>(.*?)</row>").unwrap(),
        xlsx_cell: Regex::new(r#"(?s)<c\b([^>]*)>(.*?)</c>"#).unwrap(),
        xlsx_v: Regex::new(r"(?s)<v>(.*?)</v>|<t(?:\s[^>]*)?>(.*?)</t>").unwrap(),
        pptx_p: Regex::new(r"(?s)<a:p\b[^>]*>(.*?)</a:p>").unwrap(),
        pptx_t: Regex::new(r"(?s)<a:t(?:\s[^>]*)?>(.*?)</a:t>").unwrap(),
        pdf_tj: Regex::new(r"\(([^()\\]*(?:\\.[^()\\]*)*)\)\s*Tj|\[(.*?)\]\s*TJ").unwrap(),
    })
}

fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#10;", "\n")
        .replace("&#13;", "")
        .replace("&amp;", "&")
}

pub fn decode_any_bytes(raw: &[u8]) -> (String, &'static str, u32) {
    if let Some(rest) = raw.strip_prefix(b"\xEF\xBB\xBF") {
        if let Ok(s) = std::str::from_utf8(rest) {
            return (s.to_string(), "utf-8-bom", 1);
        }
    }
    if let Some(rest) = raw.strip_prefix(b"\xFF\xFE") {
        let (cow, _, _) = encoding_rs::UTF_16LE.decode(rest);
        return (cow.into_owned(), "utf-16le", 1);
    }
    if let Some(rest) = raw.strip_prefix(b"\xFE\xFF") {
        let (cow, _, _) = encoding_rs::UTF_16BE.decode(rest);
        return (cow.into_owned(), "utf-16be", 1);
    }
    if let Ok(s) = std::str::from_utf8(raw) {
        return (s.to_string(), "utf-8", 0);
    }
    let (gbk_cow, gbk_err) = encoding_rs::GB18030.decode_without_bom_handling(raw);
    if !gbk_err {
        return (gbk_cow.into_owned(), "gb18030/gbk", 1);
    }
    let (win_cow, _) = encoding_rs::WINDOWS_1252.decode_without_bom_handling(raw);
    (win_cow.into_owned(), "windows-1252", 1)
}

fn extract_docx_md(bytes: &[u8]) -> Result<String, String> {
    let mut zip =
        ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("docx zip error: {e}"))?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .map_err(|_| "word/document.xml missing in .docx".to_string())?
        .read_to_string(&mut xml)
        .map_err(|e| e.to_string())?;

    let re = xre();
    let mut out = Vec::new();
    for m in re.docx_block.find_iter(&xml) {
        let blk = m.as_str();
        if blk.starts_with("<w:tbl") {
            let mut rows: Vec<Vec<String>> = Vec::new();
            for tr in re.docx_tr.find_iter(blk) {
                let mut cells = Vec::new();
                for tc in re.docx_tc.find_iter(tr.as_str()) {
                    let txt: String = re
                        .docx_t
                        .captures_iter(tc.as_str())
                        .map(|c| unescape_xml(&c[1]))
                        .collect::<Vec<_>>()
                        .join("");
                    cells.push(
                        txt.replace('|', "\\|")
                            .replace('\n', " ")
                            .trim()
                            .to_string(),
                    );
                }
                if !cells.is_empty() {
                    rows.push(cells);
                }
            }
            if !rows.is_empty() {
                let width = rows.iter().map(|r| r.len()).max().unwrap_or(1);
                for (ri, r) in rows.iter().enumerate() {
                    let mut padded = r.clone();
                    padded.resize(width, String::new());
                    out.push(format!("| {} |", padded.join(" | ")));
                    if ri == 0 {
                        out.push(format!("| {} |", vec!["---"; width].join(" | ")));
                    }
                }
                out.push(String::new());
            }
        } else {
            let txt: String = re
                .docx_t
                .captures_iter(blk)
                .map(|c| unescape_xml(&c[1]))
                .collect::<Vec<_>>()
                .join("");
            let trimmed = txt.trim();
            if trimmed.is_empty() {
                continue;
            }
            let prefix = match re
                .docx_style
                .captures(blk)
                .map(|c| c[1].to_ascii_lowercase())
                .as_deref()
            {
                Some(s) if s.contains("heading1") || s == "1" || s == "title" => "# ",
                Some(s) if s.contains("heading2") || s == "2" => "## ",
                Some(s) if s.contains("heading3") || s == "3" => "### ",
                _ => "",
            };
            out.push(format!("{prefix}{trimmed}"));
        }
    }
    Ok(out.join("\n"))
}

fn col_ref_to_index(attrs: &str) -> Option<usize> {
    let r_pos = attrs.find("r=\"")?;
    let after = &attrs[r_pos + 3..];
    let end_q = after.find('"')?;
    let letters: String = after[..end_q]
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect();
    if letters.is_empty() {
        return None;
    }
    let mut idx = 0usize;
    for ch in letters.bytes() {
        idx = idx * 26 + ((ch.to_ascii_uppercase() - b'A') as usize + 1);
    }
    Some(idx.saturating_sub(1))
}

fn extract_xlsx_md(
    bytes: &[u8],
    sheet_filter: Option<&str>,
    query: Option<&str>,
) -> Result<String, String> {
    let mut zip =
        ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("xlsx zip error: {e}"))?;
    let re = xre();

    let mut shared: Vec<String> = Vec::new();
    if let Ok(mut sf) = zip.by_name("xl/sharedStrings.xml") {
        let mut sxml = String::new();
        let _ = sf.read_to_string(&mut sxml);
        for cap in re.xlsx_si.captures_iter(&sxml) {
            let plain = unescape_xml(&re.xml_tag.replace_all(&cap[1], ""));
            shared.push(plain.replace('\n', " ").trim().to_string());
        }
    }

    let mut sheet_names: Vec<String> = Vec::new();
    if let Ok(mut wf) = zip.by_name("xl/workbook.xml") {
        let mut wxml = String::new();
        let _ = wf.read_to_string(&mut wxml);
        for cap in re.xlsx_sheet.captures_iter(&wxml) {
            sheet_names.push(unescape_xml(&cap[1]));
        }
    }

    let q_low = query.map(|q| q.to_ascii_lowercase());
    let sf_low = sheet_filter.map(|s| s.to_ascii_lowercase());
    let mut sections = Vec::new();
    let max_sheets = sheet_names.len().max(1);

    for idx in 0..max_sheets {
        let sname = sheet_names
            .get(idx)
            .cloned()
            .unwrap_or_else(|| format!("Sheet{}", idx + 1));
        if let Some(ref f) = sf_low {
            if !sname.to_ascii_lowercase().contains(f) {
                continue;
            }
        }
        let xml_path = format!("xl/worksheets/sheet{}.xml", idx + 1);
        let Ok(mut shf) = zip.by_name(&xml_path) else {
            continue;
        };
        let mut shxml = String::new();
        let _ = shf.read_to_string(&mut shxml);

        let mut rows: Vec<Vec<String>> = Vec::new();
        for rcap in re.xlsx_row.captures_iter(&shxml) {
            let mut row_cells: Vec<String> = Vec::new();
            let mut cur_col = 0usize;
            for ccap in re.xlsx_cell.captures_iter(&rcap[1]) {
                let attrs = &ccap[1];
                let inner = &ccap[2];
                let col_idx = col_ref_to_index(attrs).unwrap_or(cur_col);
                if row_cells.len() <= col_idx {
                    row_cells.resize(col_idx + 1, String::new());
                }
                let is_shared = attrs.contains("t=\"s\"");
                let raw_val = re
                    .xlsx_v
                    .captures(inner)
                    .and_then(|vc| vc.get(1).or_else(|| vc.get(2)))
                    .map(|m| unescape_xml(m.as_str()))
                    .unwrap_or_default();
                let resolved = if is_shared {
                    raw_val
                        .trim()
                        .parse::<usize>()
                        .ok()
                        .and_then(|si| shared.get(si).cloned())
                        .unwrap_or(raw_val)
                } else {
                    raw_val
                };
                row_cells[col_idx] = resolved
                    .replace('|', "\\|")
                    .replace('\n', " ")
                    .trim()
                    .to_string();
                cur_col = col_idx + 1;
            }
            if row_cells.iter().any(|c| !c.is_empty()) {
                rows.push(row_cells);
            }
        }

        let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut out_lines = vec![format!(
            "## Sheet: `{sname}` ({} non-empty rows x {width} cols)",
            rows.len()
        )];
        if rows.is_empty() || width == 0 {
            out_lines.push("*(empty sheet)*".to_string());
            sections.push(out_lines.join("\n"));
            continue;
        }

        let mut shown = 0usize;
        for (ri, r) in rows.iter().enumerate() {
            if ri > 0 {
                if let Some(ref ql) = q_low {
                    let hit = r.iter().any(|c| c.to_ascii_lowercase().contains(ql));
                    if !hit {
                        continue;
                    }
                }
            }
            let mut padded = r.clone();
            padded.resize(width, String::new());
            out_lines.push(format!("| {} |", padded.join(" | ")));
            if ri == 0 {
                out_lines.push(format!("| {} |", vec!["---"; width].join(" | ")));
            }
            shown += 1;
            if shown >= 40 {
                out_lines.push(format!(
                    "... ({} total rows in `{sname}` — pass `--query <term>` or `--sheet \"{sname}\"` to filter)",
                    rows.len()
                ));
                break;
            }
        }
        sections.push(out_lines.join("\n"));
    }
    Ok(sections.join("\n\n"))
}

fn extract_pptx_or_zip(bytes: &[u8], ext: &str) -> Result<String, String> {
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("zip error: {e}"))?;
    let re = xre();
    if ext == "pptx" {
        let mut slides = Vec::new();
        for i in 1..=200 {
            let name = format!("ppt/slides/slide{i}.xml");
            let Ok(mut sf) = zip.by_name(&name) else {
                if i == 1 {
                    break;
                }
                continue;
            };
            let mut xml = String::new();
            let _ = sf.read_to_string(&mut xml);
            let mut paras = Vec::new();
            for pm in re.pptx_p.captures_iter(&xml) {
                let txt: String = re
                    .pptx_t
                    .captures_iter(&pm[1])
                    .map(|c| unescape_xml(&c[1]))
                    .collect::<Vec<_>>()
                    .join("");
                if !txt.trim().is_empty() {
                    paras.push(format!("- {}", txt.trim()));
                }
            }
            slides.push(format!("## Slide {i}\n{}", paras.join("\n")));
        }
        return Ok(slides.join("\n\n"));
    }
    let mut listing = vec![format!("# ZIP Archive ({} entries)", zip.len())];
    for i in 0..zip.len().min(120) {
        if let Ok(f) = zip.by_index(i) {
            listing.push(format!("- `{}` ({} bytes)", f.name(), f.size()));
        }
    }
    Ok(listing.join("\n"))
}

fn extract_pdf_text(bytes: &[u8]) -> Result<String, String> {
    let re = xre();
    let mut chunks = Vec::new();
    let mut pos = 0usize;
    while let Some(rel_s) = bytes[pos..].windows(6).position(|w| w == b"stream") {
        let mut s_start = pos + rel_s + 6;
        if bytes.get(s_start) == Some(&b'\r') {
            s_start += 1;
        }
        if bytes.get(s_start) == Some(&b'\n') {
            s_start += 1;
        } else {
            pos = s_start;
            continue;
        }
        let Some(rel_e) = bytes[s_start..].windows(9).position(|w| w == b"endstream") else {
            break;
        };
        let stream_slice = &bytes[s_start..s_start + rel_e];
        pos = s_start + rel_e + 9;

        let mut decoded = Vec::new();
        if ZlibDecoder::new(stream_slice)
            .read_to_end(&mut decoded)
            .is_err()
        {
            decoded.clear();
            if DeflateDecoder::new(stream_slice)
                .read_to_end(&mut decoded)
                .is_err()
            {
                decoded.extend_from_slice(stream_slice);
            }
        }
        let text = String::from_utf8_lossy(&decoded);
        if text.contains("Tj") || text.contains("TJ") {
            let mut line_buf = Vec::new();
            for cap in re.pdf_tj.captures_iter(&text) {
                if let Some(m1) = cap.get(1) {
                    line_buf.push(m1.as_str().replace("\\(", "(").replace("\\)", ")"));
                } else if let Some(m2) = cap.get(2) {
                    let mut in_p = false;
                    let mut cur = String::new();
                    for ch in m2.as_str().chars() {
                        if ch == '(' && !in_p {
                            in_p = true;
                        } else if ch == ')' && in_p {
                            in_p = false;
                        } else if in_p {
                            cur.push(ch);
                        }
                    }
                    if !cur.is_empty() {
                        line_buf.push(cur);
                    }
                }
            }
            if !line_buf.is_empty() {
                chunks.push(line_buf.join(" "));
            }
        }
    }
    if chunks.is_empty() {
        Ok(
            "[ap read] Scanned PDF contains no embedded text streams (likely image-only scan)."
                .to_string(),
        )
    } else {
        Ok(chunks.join("\n"))
    }
}

fn parse_line_window(spec: &str, total: usize) -> Option<(usize, usize)> {
    let norm = spec.trim().replace(':', "-");
    let (a, b) = norm.split_once('-')?;
    let s = a.trim().parse::<usize>().ok()?.max(1);
    let e = b.trim().parse::<usize>().ok()?.max(s).min(total.max(1));
    Some((s, e))
}

fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, String> {
    let curl_bin = if cfg!(windows) { "curl.exe" } else { "curl" };
    let out = Command::new(curl_bin)
        .args([
            "-fsSL",
            "-L",
            "--max-time",
            "20",
            "-A",
            "Mozilla/5.0 ap-reader",
            url,
        ])
        .output()
        .map_err(|e| format!("failed to invoke {curl_bin}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{curl_bin} failed (exit {:?}): {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

pub fn run_cli(args: &[String]) -> i32 {
    let cli = match ReadCli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            return code;
        }
    };

    let target = cli.target.trim();
    let is_url = target.starts_with("http://") || target.starts_with("https://");

    let (raw_bytes, ext, label) = if is_url {
        match fetch_url_bytes(target) {
            Ok(b) => (b, "html".to_string(), target.to_string()),
            Err(e) => {
                eprintln!("[ap read] {e}");
                return 1;
            }
        }
    } else {
        let p = Path::new(target);
        if !p.exists() {
            eprintln!("[ap read] path not found: {target}");
            return 1;
        }
        if p.is_dir() {
            return ap_atlas::run_cli(&[
                "atlas".to_string(),
                "map".to_string(),
                target.to_string(),
            ]);
        }
        let Ok(b) = fs::read(p) else {
            eprintln!("[ap read] failed to read file: {target}");
            return 1;
        };
        let ext_s = p
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        (b, ext_s, p.to_string_lossy().replace('\\', "/"))
    };

    let raw_len = raw_bytes.len() as u64;
    let mut win_heals = 0u32;
    let mut format_tag = ext.as_str();

    let converted: String = match ext.as_str() {
        "docx" => match extract_docx_md(&raw_bytes) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[ap read] {e}");
                return 1;
            }
        },
        "xlsx" => match extract_xlsx_md(&raw_bytes, cli.sheet.as_deref(), cli.query.as_deref()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[ap read] {e}");
                return 1;
            }
        },
        "pptx" | "zip" => match extract_pptx_or_zip(&raw_bytes, &ext) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[ap read] {e}");
                return 1;
            }
        },
        "pdf" => match extract_pdf_text(&raw_bytes) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[ap read] {e}");
                return 1;
            }
        },
        _ => {
            let (decoded, enc_name, healed) = decode_any_bytes(&raw_bytes);
            win_heals += healed;
            format_tag = enc_name;
            if (is_url
                || matches!(
                    ext.as_str(),
                    "csv" | "tsv" | "json" | "jsonl" | "html" | "htm"
                ))
                && decoded.len() > 2400
                && cli.lines.is_none()
                && cli.query.is_none()
            {
                let sieve_store = std::env::var("SIEVE_HOME")
                    .ok()
                    .filter(|s| !s.is_empty())
                    .map(PathBuf::from)
                    .unwrap_or_else(|| std::env::temp_dir().join("ap_read_sieve"));
                let sr = ap_sieve::compress_stream_in(&sieve_store, &decoded, &label, 0, 1.0);
                sr.rendered
            } else {
                decoded
            }
        }
    };

    let all_lines: Vec<&str> = converted.lines().collect();
    let total_lines = all_lines.len();

    let rendered = if let Some(ref lspec) = cli.lines {
        if let Some((s, e)) = parse_line_window(lspec, total_lines) {
            let slice = (s..=e)
                .map(|ln| format!("{ln:5}: {}", all_lines[ln - 1]))
                .collect::<Vec<_>>()
                .join("\n");
            format!("# READ `{label}` [L{s}-L{e} of {total_lines} | {format_tag}]\n{slice}")
        } else {
            eprintln!("[ap read] invalid --lines `{lspec}` (expected START-END, e.g. 1-120)");
            return 2;
        }
    } else if let Some(ref q) = cli.query.clone().filter(|_| ext != "xlsx") {
        let q_low = q.to_ascii_lowercase();
        let mut hits = Vec::new();
        for (idx, ln) in all_lines.iter().enumerate() {
            if ln.to_ascii_lowercase().contains(&q_low) {
                let s = idx.saturating_sub(1);
                let e = (idx + 1).min(total_lines.saturating_sub(1));
                let ctx = (s..=e)
                    .map(|k| format!("  {}: {}", k + 1, all_lines[k]))
                    .collect::<Vec<_>>()
                    .join("\n");
                hits.push(ctx);
                if hits.len() >= 25 {
                    break;
                }
            }
        }
        format!(
            "# READ `{label}` [query='{q}' | {} match block(s) in {total_lines} lines]\n{}",
            hits.len(),
            if hits.is_empty() {
                "*(no matches)*".to_string()
            } else {
                hits.join("\n  ---\n")
            }
        )
    } else if converted.len() > cli.max_chars {
        let headings: Vec<String> = all_lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.starts_with('#') || l.starts_with("## Sheet:"))
            .take(25)
            .map(|(i, l)| format!("  L{}: {}", i + 1, l))
            .collect();
        let head_n = 45.min(total_lines);
        let head_s = (1..=head_n)
            .map(|ln| format!("{ln:5}: {}", all_lines[ln - 1]))
            .collect::<Vec<_>>()
            .join("\n");
        let outline = if headings.is_empty() {
            String::new()
        } else {
            format!("Outline:\n{}\n\n", headings.join("\n"))
        };
        format!(
            "# READ `{label}` [paged: {}B -> preview of L1-L{head_n} / {total_lines} total lines | {format_tag}]\n{outline}{head_s}\n\n[ap read pager: pass `--lines {}-{} ` or `--query <keyword>` to inspect specific sections]",
            raw_len,
            head_n + 1,
            (head_n + 120).min(total_lines.max(1))
        )
    } else {
        format!("# READ `{label}` [{total_lines} lines | {format_tag}]\n{converted}")
    };

    let out_len = rendered.len() as u64;
    record_event("read", raw_len, out_len, win_heals, 0, format_tag);

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "target": label,
                "format": format_tag,
                "raw_bytes": raw_len,
                "emitted_bytes": out_len,
                "total_lines": total_lines,
                "content": rendered,
            }))
            .unwrap_or_default()
        );
    } else {
        println!("{rendered}");
    }
    0
}
