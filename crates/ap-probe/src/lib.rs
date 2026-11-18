pub mod runtime;
pub mod slice;

use base64::Engine;
use clap::{Parser, Subcommand};
use runtime::{detect_lang, run_diff_dispatch, run_trace_dispatch};
use slice::run_slice;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "probe", about = "Tree-sitter reachability slicer, runtime microscope & differential oracle")]
struct Cli {
    #[command(subcommand)]
    cmd: SubCmd,
}

#[derive(Subcommand, Debug)]
enum SubCmd {
    /// Static Tree-sitter/import reachability slice (Python, TS/JS, Go, Rust, C) & active bundle
    Slice {
        #[arg(default_value = ".")]
        root: PathBuf,
        #[arg(long = "entry")]
        entries: Vec<String>,
        #[arg(long)]
        bundle: bool,
        #[arg(long, default_value_t = 18)]
        max_bundle_kb: usize,
        #[arg(long)]
        json: bool,
    },
    /// Trace execution & auto-detect heap aliasing, in-place mutation, and state no-ops
    Run {
        #[arg(long, default_value = ".")]
        cwd: PathBuf,
        #[arg(long, default_value = "auto")]
        lang: String,
        #[arg(long, default_value = "")]
        focus: String,
        #[arg(long)]
        code: Option<String>,
        #[arg(long)]
        b64: Option<String>,
        #[arg(long)]
        stdin: bool,
        #[arg(long)]
        script: Option<String>,
        #[arg(long)]
        unittest: Option<String>,
        #[arg(long, default_value_t = 30)]
        max_calls: usize,
        #[arg(long)]
        include_tests: bool,
        #[arg(long)]
        json: bool,
    },
    /// Differential & boundary-matrix oracle across Python, TS/JS, or Rust
    Diff {
        #[arg(long, default_value = ".")]
        cwd: PathBuf,
        #[arg(long, default_value = "auto")]
        lang: String,
        #[arg(long, default_value = "")]
        setup: String,
        #[arg(long)]
        fn_a: String,
        #[arg(long)]
        fn_b: String,
        #[arg(long, default_value = "")]
        vars: String,
        #[arg(long, default_value = "")]
        inputs: String,
        #[arg(long)]
        json: bool,
    },
}

pub fn run_cli(args: &[String]) -> i32 {
    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            return code;
        }
    };
    match cli.cmd {
        SubCmd::Slice {
            root,
            entries,
            bundle,
            max_bundle_kb,
            json,
        } => run_slice(&root, &entries, json, bundle, max_bundle_kb).1,
        SubCmd::Run {
            cwd,
            lang,
            focus,
            code,
            b64,
            stdin,
            script,
            unittest,
            max_calls,
            include_tests,
            json,
        } => {
            let snippet = if stdin {
                let mut s = String::new();
                let _ = std::io::stdin().read_to_string(&mut s);
                Some(s)
            } else if let Some(b) = b64 {
                match base64::engine::general_purpose::STANDARD.decode(b.trim()) {
                    Ok(bytes) => Some(String::from_utf8_lossy(&bytes).to_string()),
                    Err(e) => {
                        eprintln!("[probe run] invalid --b64 payload: {e}");
                        return 2;
                    }
                }
            } else {
                code
            };
            if snippet.is_none() && script.is_none() && unittest.is_none() {
                eprintln!("[probe run] pass one of --code, --stdin, --b64, --script, or --unittest");
                return 2;
            }
            let plang = detect_lang(&cwd, &lang, &focus, snippet.as_deref(), script.as_deref());
            run_trace_dispatch(
                &cwd,
                plang,
                &focus,
                snippet.as_deref(),
                script.as_deref(),
                unittest.as_deref(),
                max_calls,
                include_tests,
                json,
            )
        }
        SubCmd::Diff {
            cwd,
            lang,
            setup,
            fn_a,
            fn_b,
            vars,
            inputs,
            json,
        } => {
            let combined = format!("{setup}\n{fn_a}");
            let plang = detect_lang(&cwd, &lang, "", Some(&combined), None);
            run_diff_dispatch(&cwd, plang, &setup, &fn_a, &fn_b, &vars, &inputs, json)
        }
    }
}
