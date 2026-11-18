pub mod audit;
pub mod bench;
pub mod exec;
pub mod install;
pub mod mcp;
pub mod stats;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "ap",
    version,
    about = "Zero-config Windows & macOS execution engine for AI coding agents — heals shell/UTF-8 crashes, prevents hung servers & cuts token waste by 50-80%"
)]
struct TopCli {
    #[command(subcommand)]
    cmd: TopCmd,
}

#[derive(Subcommand, Debug)]
enum TopCmd {
    /// 1-command auto-setup for all detected coding agents (install & forget)
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
    /// View saved tokens, compressed logs & prevented Windows crashes
    #[command(visible_alias = "gain")]
    Stats {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        reset: bool,
        #[arg(long)]
        home: Option<String>,
    },
    /// Check Windows/macOS shell, UTF-8, headless browser CDP & agent wiring health
    Doctor,
    /// Run commands with UTF-8 lock, Bash-in-PowerShell auto-heal, port unlock & log compression
    Exec {
        #[arg(short = 'c', long = "cmd")]
        cmd: Option<String>,
    },
    /// Show current repository and global multi-agent installation status
    Status {
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        home: Option<String>,
    },
    /// Cleanly remove installed skills, hooks, MCP configs, and rule blocks
    Uninstall {
        #[arg(long)]
        here: bool,
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        home: Option<String>,
    },
    /// Start native Model Context Protocol (MCP) stdio JSON-RPC server
    Mcp,
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
        "exec" => Some(exec::run_cli(&sub_argv)),
        "stats" | "gain" => Some(stats::run_cli(&sub_argv)),
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

    // 1. Busybox `argv[0]` dispatch if symlinked/hardlinked
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

    // 2. Subcommand passthrough to `exec`, `stats`/`gain`, `audit`, and internal engine modules
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
                    "\nInternal Engine Modules (automatically invoked by agents via MCP & hooks):\n  atlas / splice / probe / sieve / guard / recall / baton / audit"
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
        TopCmd::Stats { .. } => stats::run_cli(&args[1..]),
        TopCmd::Doctor => install::cmd_doctor(),
        TopCmd::Exec { .. } => exec::run_cli(&args[1..]),
        TopCmd::Status { dir, home } => install::cmd_status(dir.as_deref(), home.as_deref()),
        TopCmd::Uninstall { here, dir, home } => {
            install::cmd_uninstall(here, dir.as_deref(), home.as_deref())
        }
        TopCmd::Mcp => mcp::run_mcp_stdio(),
        TopCmd::Bench { out } => bench::run_bench(std::path::Path::new(&out)),
    };
    std::process::exit(code);
}
