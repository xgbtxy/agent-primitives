pub mod audit;
pub mod bench;
pub mod install;
pub mod mcp;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "ap",
    version,
    about = "Single-binary Rust OS/AST/Heap/SQLite micro-infrastructure primitives for AI agents"
)]
struct TopCli {
    #[command(subcommand)]
    cmd: TopCmd,
}

#[derive(Subcommand, Debug)]
enum TopCmd {
    /// Auto-detect and onboard AI harnesses (Codex, Pi, Antigravity, Claude Code, Grok, OpenClaw, Hermes)
    Init {
        #[arg(short = 'a', long, default_value = "auto")]
        agent: String,
        #[arg(long, default_value = "sieve,baton,recall,guard,atlas,probe,splice")]
        only: String,
        #[arg(long)]
        here: bool,
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        home: Option<String>,
        #[arg(long)]
        with_hooks: bool,
        #[arg(long)]
        with_mcp: bool,
        #[arg(long)]
        no_rules: bool,
    },
    /// Start zero-dependency Model Context Protocol (MCP) stdio JSON-RPC server
    Mcp,
    /// Cleanly remove installed skills, binary shims, hooks, and fenced rule blocks
    Uninstall {
        #[arg(long)]
        here: bool,
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        home: Option<String>,
    },
    /// Verify bundled SQLite FTS5/BM25, 6 Tree-sitter AST grammars & all 7 embedded primitives
    Doctor,
    /// Show current repository and global multi-harness installation status
    Status {
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        home: Option<String>,
    },
    /// Run deterministic micro-benchmark suite (with honest negative controls)
    Bench {
        #[arg(long, default_value = "bench/results.json")]
        out: String,
    },
}

fn dispatch_primitive(name: &str, tail_args: &[String]) -> Option<i32> {
    let mut sub_argv = Vec::with_capacity(tail_args.len() + 1);
    sub_argv.push(name.to_string());
    sub_argv.extend_from_slice(tail_args);
    match name {
        "sieve" => Some(ap_sieve::run_cli(&sub_argv)),
        "baton" => Some(ap_baton::run_cli(&sub_argv)),
        "recall" => Some(ap_recall::run_cli(&sub_argv)),
        "guard" => Some(ap_guard::run_cli(&sub_argv)),
        "atlas" => Some(ap_atlas::run_cli(&sub_argv)),
        "probe" => Some(ap_probe::run_cli(&sub_argv)),
        "splice" => Some(ap_splice::run_cli(&sub_argv)),
        "audit" => Some(audit::run_cli(&sub_argv)),
        _ => None,
    }
}

fn main() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info.to_string();
        if msg.contains("failed printing to stdout")
            || msg.contains("Broken pipe")
            || msg.contains("os error 232")
        {
            std::process::exit(0);
        }
        default_hook(info);
    }));

    let args: Vec<String> = std::env::args().collect();

    // 1. Busybox `argv[0]` dispatch if symlinked/hardlinked as `sieve`, `probe`, `splice`, etc.
    if let Some(arg0) = args.first() {
        let stem = std::path::Path::new(arg0)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if stem != "ap" && stem != "agent-primitives" {
            if let Some(code) = dispatch_primitive(&stem, &args[1..]) {
                std::process::exit(code);
            }
        }
    }

    // 2. Subcommand passthrough to the 7 standalone primitives + `audit`
    if let Some(sub) = args.get(1).map(|s| s.as_str()) {
        if let Some(code) = dispatch_primitive(sub, &args[2..]) {
            std::process::exit(code);
        }
    }

    let cli = match TopCli::try_parse_from(&args) {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            if !e.use_stderr() {
                println!(
                    "\nPrimitives (pass `<primitive> --help` for details):\n  probe   Tree-sitter reachability slicer (`slice --bundle`), runtime & diff microscope\n  splice  Tree-sitter AST symbol view, zero-escaping atomic patcher (`apply --stdin --test`) & check\n  sieve   Lossless log/JSON/JSONL/HTML/CSV compressor + SQLite FTS5 `search` & `peek`\n  atlas   Sub-50ms 6-language Tree-sitter codebase & Markdown vault structural map\n  recall  Sub-ms bundled SQLite FTS5/BM25 + CJK triggered micro-memory vault\n  guard   Sub-2ms exit-2 blast-radius shield, loop breaker & dual-protocol hook gate\n  baton   Compaction-proof <500-token state & DO-NOT-RETRY handoff capsule\n  audit   Local zero-cloud session telemetry auditor (`ap audit --compare`)"
                );
            }
            std::process::exit(code);
        }
    };

    let code = match cli.cmd {
        TopCmd::Init {
            agent,
            only,
            here,
            dir,
            home,
            with_hooks,
            with_mcp,
            no_rules,
        } => install::cmd_init(
            &agent,
            &only,
            here,
            dir.as_deref(),
            home.as_deref(),
            with_hooks,
            with_mcp,
            no_rules,
        ),
        TopCmd::Mcp => mcp::run_mcp_stdio(),
        TopCmd::Uninstall { here, dir, home } => {
            install::cmd_uninstall(here, dir.as_deref(), home.as_deref())
        }
        TopCmd::Doctor => install::cmd_doctor(),
        TopCmd::Status { dir, home } => install::cmd_status(dir.as_deref(), home.as_deref()),
        TopCmd::Bench { out } => bench::run_bench(std::path::Path::new(&out)),
    };
    std::process::exit(code);
}
