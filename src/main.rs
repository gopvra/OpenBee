// OpenBee — AI Agent Platform
//
// A hardworking bee that executes tasks: crypto wallet operations,
// browser automation, AI reading, and extensible skill-based tasks.

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use openbee_agent::skills::{BrowserSkill, SystemSkill, WalletSkill};
use openbee_agent::BeeAgent;

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "--wallet" => run_wallet(&args[2..]),
        "--agent" | "--task" => run_agent_task(&args[2..]),
        "--interactive" | "-i" => run_interactive(),
        "--version" | "-v" => {
            println!("OpenBee v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            // Treat the entire args as a task instruction
            let instruction = args[1..].join(" ");
            run_agent_task_str(&instruction)
        }
    }
}

/// Run a single agent task from CLI args.
fn run_agent_task(args: &[String]) -> Result<()> {
    if args.is_empty() {
        println!("Usage: openbee --agent <instruction>");
        println!("Example: openbee --agent check system status");
        return Ok(());
    }
    let instruction = args.join(" ");
    run_agent_task_str(&instruction)
}

fn run_agent_task_str(instruction: &str) -> Result<()> {
    let mut agent = create_agent();
    info!("Task: '{}'", instruction);

    let id = agent.submit(instruction);
    let result = agent.get_result(id);

    match result {
        Some(r) => {
            if r.success {
                println!("{}", r.summary);
                if !r.data.is_null() {
                    println!(
                        "\n{}",
                        serde_json::to_string_pretty(&r.data).unwrap_or_default()
                    );
                }
                if !r.suggestions.is_empty() {
                    println!("\nSuggestions:");
                    for s in &r.suggestions {
                        println!("  → {}", s);
                    }
                }
            } else {
                eprintln!("Failed: {}", r.error.as_deref().unwrap_or("unknown"));
            }
        }
        None => eprintln!("No result for task #{}", id),
    }
    Ok(())
}

/// Interactive REPL mode.
fn run_interactive() -> Result<()> {
    use std::io::{self, BufRead, Write};

    let mut agent = create_agent();

    println!("🐝 OpenBee Agent v{}", env!("CARGO_PKG_VERSION"));
    println!("Type a task, or 'help' / 'quit'.\n");

    let stdin = io::stdin();
    loop {
        print!("🐝 > ");
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break; // EOF
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "quit" || line == "exit" || line == "q" {
            println!("Bye!");
            break;
        }
        if line == "help" || line == "帮助" {
            println!("{}", agent.help());
            continue;
        }
        if line == "stats" || line == "统计" {
            let s = agent.stats();
            println!(
                "Tasks: {} total ({} ok, {} failed) | Skills: {} | Avg: {}ms",
                s.total_tasks, s.succeeded, s.failed, s.skills_registered, s.avg_duration_ms
            );
            continue;
        }
        if line == "history" || line == "历史" {
            for task in agent.history().iter().rev().take(10) {
                let status = match &task.status {
                    openbee_agent::TaskStatus::Completed => "✓",
                    openbee_agent::TaskStatus::Failed { .. } => "✗",
                    _ => "?",
                };
                println!("  {} #{} {}", status, task.id, task.instruction);
            }
            continue;
        }

        let id = agent.submit(line);
        if let Some(r) = agent.get_result(id) {
            println!("{}", r.summary);
            if !r.suggestions.is_empty() {
                for s in &r.suggestions {
                    println!("  → {}", s);
                }
            }
        }
        println!();
    }
    Ok(())
}

/// Wallet subcommands.
fn run_wallet(args: &[String]) -> Result<()> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("help");
    let instruction = format!("wallet {}", args.join(" "));

    match sub {
        "help" | "--help" => {
            print_wallet_help();
            Ok(())
        }
        _ => run_agent_task_str(&instruction),
    }
}

/// Create an agent with all default skills.
fn create_agent() -> BeeAgent {
    let mut agent = BeeAgent::new();
    agent.register_skill(Box::new(WalletSkill::new()));
    agent.register_skill(Box::new(BrowserSkill::new()));
    agent.register_skill(Box::new(SystemSkill::new()));
    agent
}

fn print_help() {
    println!("🐝 OpenBee — AI Agent Platform");
    println!();
    println!("USAGE:");
    println!("    openbee [COMMAND | INSTRUCTION]");
    println!();
    println!("COMMANDS:");
    println!("    -i, --interactive        Interactive REPL mode");
    println!("    --agent <instruction>    Execute a single task");
    println!("    --wallet <subcommand>    Crypto wallet operations");
    println!("    -v, --version            Show version");
    println!("    -h, --help               Show this help");
    println!();
    println!("WALLET:");
    println!("    --wallet create           Create new wallet");
    println!("    --wallet recover          Recover from mnemonic");
    println!("    --wallet address [chain]  Show addresses");
    println!("    --wallet send CHAIN TO AMT  Sign transaction");
    println!("    --wallet status           Show wallet info");
    println!();
    println!("EXAMPLES:");
    println!("    openbee -i                        Start interactive mode");
    println!("    openbee check system status        Run a task directly");
    println!("    openbee --wallet create            Create crypto wallet");
    println!("    openbee help me with my balance    Natural language task");
    println!();
    println!("SKILLS:");
    println!("    wallet   — Crypto: balance, send, swap (8 chains, 4 DEX)");
    println!("    browser  — Web: read pages, extract AI responses");
    println!("    system   — Status, version, diagnostics");
}

fn print_wallet_help() {
    println!("🐝 OpenBee Wallet — Secure Multi-Chain Crypto");
    println!();
    println!("SECURITY: 6 layers (AES-256-GCM, Argon2id, machine binding,");
    println!("          auto-wipe after 5 wrong passwords, zeroize, default-off)");
    println!();
    println!("CHAINS: Ethereum, BSC, Polygon, Arbitrum, Optimism, Base, Avalanche, Solana");
    println!("DEX:    Uniswap V3, PancakeSwap V3, 1inch, Jupiter");
    println!();
    println!("COMMANDS:");
    println!("    openbee --wallet create           Generate new 12-word mnemonic");
    println!("    openbee --wallet recover           Restore from mnemonic");
    println!("    openbee --wallet address           Show all addresses");
    println!("    openbee --wallet address ethereum   Show ETH address");
    println!("    openbee --wallet send ethereum 0x... 0.1");
    println!("    openbee --wallet status            Show wallet info");
}
