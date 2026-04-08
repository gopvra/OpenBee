// OpenBee - Rust reimplementation of Captain Claw (1997)
//
// A complete, modular game engine reimplementing the classic platformer
// with modern enhancements: Lua scripting, level editor, multiplayer,
// mod support, crypto wallet, and more.

use anyhow::Result;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use openbee_game::game_app::{BeeGameApp, Difficulty, GameConfig};

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("OpenBee v{} starting up...", env!("CARGO_PKG_VERSION"));

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let mode = parse_launch_mode(&args);

    match mode {
        LaunchMode::Game(config) => run_game(config),
        LaunchMode::Editor => run_editor(),
        LaunchMode::Server(addr, port) => run_server(addr, port),
        LaunchMode::Wallet(cmd) => run_wallet(cmd),
        LaunchMode::Help => {
            print_help();
            Ok(())
        }
    }
}

#[derive(Debug)]
enum LaunchMode {
    Game(GameConfig),
    Editor,
    Server(String, u16),
    Wallet(WalletCommand),
    Help,
}

#[derive(Debug)]
enum WalletCommand {
    Create,
    Unlock,
    Lock,
    Address {
        chain: Option<String>,
    },
    Send {
        chain: String,
        to: String,
        amount: String,
    },
    Recover,
    Status,
    Help,
}

fn parse_launch_mode(args: &[String]) -> LaunchMode {
    if args.len() < 2 {
        return LaunchMode::Game(GameConfig::default());
    }

    match args[1].as_str() {
        "--editor" | "-e" => LaunchMode::Editor,
        "--server" | "-s" => {
            let addr = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| "0.0.0.0".to_string());
            let port = args.get(3).and_then(|p| p.parse().ok()).unwrap_or(27015);
            LaunchMode::Server(addr, port)
        }
        "--wallet" => {
            let sub = args.get(2).map(|s| s.as_str()).unwrap_or("help");
            let cmd = match sub {
                "create" => WalletCommand::Create,
                "unlock" => WalletCommand::Unlock,
                "lock" => WalletCommand::Lock,
                "address" => WalletCommand::Address {
                    chain: args.get(3).cloned(),
                },
                "send" => WalletCommand::Send {
                    chain: args.get(3).cloned().unwrap_or_default(),
                    to: args.get(4).cloned().unwrap_or_default(),
                    amount: args.get(5).cloned().unwrap_or_default(),
                },
                "recover" => WalletCommand::Recover,
                "status" => WalletCommand::Status,
                _ => WalletCommand::Help,
            };
            LaunchMode::Wallet(cmd)
        }
        "--help" | "-h" => LaunchMode::Help,
        "--fullscreen" | "-f" => LaunchMode::Game(GameConfig {
            fullscreen: true,
            ..Default::default()
        }),
        "--windowed" | "-w" => LaunchMode::Game(GameConfig {
            fullscreen: false,
            ..Default::default()
        }),
        "--difficulty" | "-d" => {
            let mut config = GameConfig::default();
            if let Some(diff) = args.get(2) {
                config.difficulty = match diff.to_lowercase().as_str() {
                    "easy" => Difficulty::Easy,
                    "hard" => Difficulty::Hard,
                    _ => Difficulty::Normal,
                };
            }
            LaunchMode::Game(config)
        }
        _ => LaunchMode::Game(GameConfig::default()),
    }
}

// ---- Game ----

fn run_game(config: GameConfig) -> Result<()> {
    info!("Starting game mode...");
    info!(
        "Window: {}x{}, Fullscreen: {}, VSync: {}",
        config.window_width, config.window_height, config.fullscreen, config.vsync
    );

    let mut app = BeeGameApp::new(config)?;
    app.initialize()?;

    info!("Game initialized successfully. Entering main loop...");

    let mut last_time = std::time::Instant::now();
    let target_frame_time = std::time::Duration::from_secs_f64(1.0 / 60.0);

    while app.running {
        let now = std::time::Instant::now();
        let dt = now.duration_since(last_time).as_secs_f64();
        last_time = now;

        if let Err(e) = app.run_frame(dt) {
            error!("Frame error: {}", e);
        }

        let frame_time = std::time::Instant::now().duration_since(now);
        if frame_time < target_frame_time {
            std::thread::sleep(target_frame_time - frame_time);
        }
    }

    app.shutdown();
    info!("Game shut down cleanly.");
    Ok(())
}

// ---- Editor ----

fn run_editor() -> Result<()> {
    info!("Starting level editor mode...");
    let editor = openbee_editor::EditorApp::new();
    info!("Editor app created. GUI event loop would start here.");
    drop(editor);
    Ok(())
}

// ---- Server ----

fn run_server(addr: String, port: u16) -> Result<()> {
    info!("Starting dedicated server on {}:{}", addr, port);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let config = openbee_net::server::ServerConfig {
            address: addr,
            port,
            max_players: 4,
            tick_rate: 60,
            password: None,
        };

        let mut server = openbee_net::GameServer::start(config).await?;
        info!("Server started. Waiting for connections...");

        loop {
            if let Err(e) = server.tick().await {
                error!("Server tick error: {}", e);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(16)).await;
        }

        server.shutdown().await?;
        Ok(())
    })
}

// ---- Wallet ----

fn run_wallet(cmd: WalletCommand) -> Result<()> {
    use openbee_wallet::safety::{WalletSafetyGate, SAFETY_WARNING};
    use std::io::{self, BufRead, Write};

    if let WalletCommand::Help = cmd {
        print_wallet_help();
        return Ok(());
    }

    // Step 1: Show safety warning and require confirmation
    let mut gate = WalletSafetyGate::new();

    println!("{}", SAFETY_WARNING);
    println!();
    print!("Type \"I understand the risks\" to continue, or anything else to cancel: ");
    io::stdout().flush()?;

    let mut confirmation = String::new();
    io::stdin().lock().read_line(&mut confirmation)?;
    let confirmation = confirmation.trim();

    if !gate.show_warning().is_empty() && !gate.accept(confirmation) {
        println!();
        println!("Wallet operation cancelled. You did not confirm the security warning.");
        return Ok(());
    }

    println!();
    println!("Security warning accepted. Proceeding...");
    println!();

    let wallet_dir = std::path::PathBuf::from("./wallet");

    match cmd {
        WalletCommand::Create => wallet_create(&wallet_dir),
        WalletCommand::Unlock => wallet_unlock(&wallet_dir),
        WalletCommand::Lock => wallet_lock(&wallet_dir),
        WalletCommand::Address { chain } => wallet_address(&wallet_dir, chain),
        WalletCommand::Send { chain, to, amount } => wallet_send(&wallet_dir, &chain, &to, &amount),
        WalletCommand::Recover => wallet_recover(&wallet_dir),
        WalletCommand::Status => wallet_status(&wallet_dir),
        WalletCommand::Help => Ok(()), // already handled above
    }
}

fn wallet_create(wallet_dir: &std::path::Path) -> Result<()> {
    use openbee_wallet::mnemonic::Mnemonic;
    use std::io::{self, BufRead, Write};

    if wallet_dir.join("wallet.json").exists() {
        println!("A wallet already exists at {:?}", wallet_dir);
        println!("Use --wallet recover to restore from mnemonic, or delete the directory to start fresh.");
        return Ok(());
    }

    println!("Creating a new multi-chain wallet...");
    println!();

    // Generate mnemonic
    let mnemonic = Mnemonic::generate_12()?;
    let phrase = mnemonic.phrase();

    println!("========================================");
    println!("   YOUR MNEMONIC PHRASE (12 words)");
    println!("========================================");
    println!();
    for (i, word) in phrase.split_whitespace().enumerate() {
        println!("  {:>2}. {}", i + 1, word);
    }
    println!();
    println!("========================================");
    println!();
    println!("WRITE THESE WORDS DOWN ON PAPER NOW.");
    println!("DO NOT take a screenshot or save to a file.");
    println!("DO NOT share with anyone.");
    println!("If you lose these words, your funds are GONE FOREVER.");
    println!();

    // Ask user to set a password
    print!("Set wallet password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().lock().read_line(&mut password)?;
    let password = password.trim();

    if password.len() < 8 {
        println!("Password must be at least 8 characters.");
        return Ok(());
    }

    print!("Confirm password: ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().lock().read_line(&mut confirm)?;

    if password != confirm.trim() {
        println!("Passwords do not match.");
        return Ok(());
    }

    // Create wallet
    std::fs::create_dir_all(wallet_dir)?;
    let wallet =
        openbee_wallet::wallet::Wallet::create("default", &mnemonic, password, wallet_dir)?;

    println!();
    println!("Wallet created successfully!");
    println!("Accounts:");
    for account in wallet.accounts() {
        println!("  [{}] {}", account.chain, account.address);
    }
    println!();
    println!("Wallet directory: {:?}", wallet_dir);
    println!();
    println!("REMEMBER: Back up your mnemonic phrase. It is your ONLY way to recover.");

    Ok(())
}

fn wallet_unlock(wallet_dir: &std::path::Path) -> Result<()> {
    use std::io::{self, BufRead, Write};

    if !wallet_dir.join("wallet.json").exists() {
        println!("No wallet found. Use --wallet create first.");
        return Ok(());
    }

    let mut wallet = openbee_wallet::wallet::Wallet::open(wallet_dir)?;

    print!("Enter wallet password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().lock().read_line(&mut password)?;

    match wallet.unlock(password.trim()) {
        Ok(()) => {
            println!("Wallet unlocked successfully.");
            println!("Accounts:");
            for account in wallet.accounts() {
                println!("  [{}] {}", account.chain, account.address);
            }
        }
        Err(e) => {
            println!("Failed to unlock wallet: {}", e);
            println!("WARNING: Repeated failures will trigger auto-wipe (5 attempts max).");
        }
    }

    Ok(())
}

fn wallet_lock(wallet_dir: &std::path::Path) -> Result<()> {
    if !wallet_dir.join("wallet.json").exists() {
        println!("No wallet found.");
        return Ok(());
    }

    let mut wallet = openbee_wallet::wallet::Wallet::open(wallet_dir)?;
    wallet.lock();
    println!("Wallet locked. All keys cleared from memory.");
    Ok(())
}

fn wallet_address(wallet_dir: &std::path::Path, chain: Option<String>) -> Result<()> {
    if !wallet_dir.join("wallet.json").exists() {
        println!("No wallet found. Use --wallet create first.");
        return Ok(());
    }

    let wallet = openbee_wallet::wallet::Wallet::open(wallet_dir)?;

    println!("Wallet addresses:");
    for account in wallet.accounts() {
        if let Some(ref filter) = chain {
            if &account.chain != filter {
                continue;
            }
        }
        println!(
            "  [{}] {} ({})",
            account.chain, account.address, account.derivation_path
        );
    }

    Ok(())
}

fn wallet_send(wallet_dir: &std::path::Path, chain: &str, to: &str, amount: &str) -> Result<()> {
    use std::io::{self, BufRead, Write};

    if chain.is_empty() || to.is_empty() || amount.is_empty() {
        println!("Usage: openbee --wallet send <chain> <to_address> <amount>");
        println!("Example: openbee --wallet send ethereum 0x1234...abcd 0.1");
        return Ok(());
    }

    if !wallet_dir.join("wallet.json").exists() {
        println!("No wallet found. Use --wallet create first.");
        return Ok(());
    }

    let mut wallet = openbee_wallet::wallet::Wallet::open(wallet_dir)?;

    // Unlock
    print!("Enter wallet password to sign transaction: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().lock().read_line(&mut password)?;
    wallet.unlock(password.trim())?;

    // Confirm
    println!();
    println!("  Chain:   {}", chain);
    println!("  To:      {}", to);
    println!("  Amount:  {} {}", amount, chain.to_uppercase());
    println!();
    println!("WARNING: This transaction is IRREVERSIBLE.");
    print!("Type \"confirm\" to send: ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().lock().read_line(&mut confirm)?;

    if confirm.trim() != "confirm" {
        println!("Transaction cancelled.");
        wallet.lock();
        return Ok(());
    }

    // Parse amount to smallest unit
    let amount_smallest: u128 = parse_amount(amount, chain);

    let from = wallet
        .accounts_for_chain(chain)
        .first()
        .map(|a| a.address.clone())
        .unwrap_or_default();

    let request = openbee_wallet::transaction::TransactionRequest {
        chain: chain.to_string(),
        from,
        to: to.to_string(),
        amount: amount_smallest,
        fee_limit: None,
        data: None,
        nonce: None,
        memo: None,
    };

    match wallet.sign_transaction(&request) {
        Ok(signed) => {
            println!();
            println!("Transaction signed successfully!");
            println!("  TX Hash: {}", signed.tx_hash);
            println!("  Raw TX:  {} bytes", signed.raw_tx.len());
            println!();
            println!("NOTE: Transaction broadcast requires RPC connection (not implemented in offline mode).");
            println!("Export the raw TX and broadcast via your preferred method.");
        }
        Err(e) => {
            println!("Transaction signing failed: {}", e);
        }
    }

    wallet.lock();
    Ok(())
}

fn wallet_recover(wallet_dir: &std::path::Path) -> Result<()> {
    use openbee_wallet::mnemonic::Mnemonic;
    use std::io::{self, BufRead, Write};

    if wallet_dir.join("wallet.json").exists() {
        println!(
            "A wallet already exists at {:?}. Delete it first to recover.",
            wallet_dir
        );
        return Ok(());
    }

    println!("Recover wallet from mnemonic phrase.");
    println!();
    print!("Enter your 12 or 24 word mnemonic: ");
    io::stdout().flush()?;
    let mut phrase = String::new();
    io::stdin().lock().read_line(&mut phrase)?;

    let mnemonic = match Mnemonic::from_phrase(phrase.trim()) {
        Ok(m) => m,
        Err(e) => {
            println!("Invalid mnemonic: {}", e);
            return Ok(());
        }
    };

    print!("Set new password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().lock().read_line(&mut password)?;
    let password = password.trim();

    if password.len() < 8 {
        println!("Password must be at least 8 characters.");
        return Ok(());
    }

    std::fs::create_dir_all(wallet_dir)?;
    let wallet =
        openbee_wallet::wallet::Wallet::create("recovered", &mnemonic, password, wallet_dir)?;

    println!();
    println!("Wallet recovered successfully!");
    println!("Accounts:");
    for account in wallet.accounts() {
        println!("  [{}] {}", account.chain, account.address);
    }

    Ok(())
}

fn wallet_status(wallet_dir: &std::path::Path) -> Result<()> {
    if !wallet_dir.join("wallet.json").exists() {
        println!("No wallet found. Use --wallet create first.");
        return Ok(());
    }

    let wallet = openbee_wallet::wallet::Wallet::open(wallet_dir)?;
    println!("Wallet: {:?}", wallet_dir);
    println!("Status: Locked");
    println!("Accounts: {}", wallet.accounts().len());
    for account in wallet.accounts() {
        println!(
            "  [{}] {} ({})",
            account.chain, account.address, account.derivation_path
        );
    }
    Ok(())
}

fn parse_amount(amount_str: &str, chain: &str) -> u128 {
    let decimals: u32 = match chain {
        "ethereum" => 18,
        "solana" => 9,
        _ => 18,
    };
    // Parse "0.1" -> 100000000000000000 (for 18 decimals)
    if let Some((whole, frac)) = amount_str.split_once('.') {
        let whole_val: u128 = whole.parse().unwrap_or(0);
        let frac_str = format!("{:0<width$}", frac, width = decimals as usize);
        let frac_val: u128 = frac_str[..decimals as usize].parse().unwrap_or(0);
        whole_val * 10u128.pow(decimals) + frac_val
    } else {
        let whole_val: u128 = amount_str.parse().unwrap_or(0);
        whole_val * 10u128.pow(decimals)
    }
}

// ---- Help ----

fn print_help() {
    println!("OpenBee - A hardworking bee game engine");
    println!();
    println!("USAGE:");
    println!("    openbee [OPTIONS] [REZ_FILE]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help                       Show this help");
    println!("    -e, --editor                     Launch level editor");
    println!("    -s, --server [ADDR] [PORT]       Start multiplayer server");
    println!("    -f, --fullscreen                 Fullscreen mode");
    println!("    -w, --windowed                   Windowed mode");
    println!("    -d, --difficulty [LEVEL]          Set difficulty (easy/normal/hard)");
    println!("    --wallet [COMMAND]                Crypto wallet (see --wallet help)");
    println!();
    println!("WALLET COMMANDS:");
    println!("    --wallet create                   Create a new wallet");
    println!("    --wallet recover                  Recover wallet from mnemonic");
    println!("    --wallet unlock                   Unlock wallet");
    println!("    --wallet lock                     Lock wallet");
    println!("    --wallet address [CHAIN]          Show wallet addresses");
    println!("    --wallet send CHAIN TO AMOUNT     Sign a transaction");
    println!("    --wallet status                   Show wallet status");
    println!("    --wallet help                     Wallet help");
    println!();
    println!("EXAMPLES:");
    println!("    openbee                            Run with default settings");
    println!("    openbee --editor                   Open level editor");
    println!("    openbee --wallet create            Create crypto wallet");
    println!("    openbee --wallet address ethereum  Show ETH address");
    println!("    openbee --wallet send ethereum 0xABC... 0.1");
}

fn print_wallet_help() {
    println!("OpenBee Wallet - Secure Multi-Chain Crypto Wallet");
    println!();
    println!("SECURITY:");
    println!("  - Private keys are stored ENCRYPTED (AES-256-GCM) on your device");
    println!("  - Keys are BOUND to this machine (cannot be used if file is stolen)");
    println!("  - 5 wrong password attempts = keystore AUTO-WIPED");
    println!("  - Keys are NEVER sent over the network");
    println!("  - All key material is zeroed from memory after use");
    println!();
    println!("SUPPORTED CHAINS:");
    println!("  - ethereum    Ethereum (ETH) - secp256k1");
    println!("  - solana      Solana (SOL) - Ed25519");
    println!();
    println!("COMMANDS:");
    println!();
    println!("  openbee --wallet create");
    println!("    Generate a new 12-word mnemonic and create encrypted wallet.");
    println!("    WRITE DOWN the mnemonic phrase - it's your ONLY backup!");
    println!();
    println!("  openbee --wallet recover");
    println!("    Restore a wallet from an existing mnemonic phrase.");
    println!();
    println!("  openbee --wallet unlock");
    println!("    Decrypt wallet keys into memory for signing.");
    println!();
    println!("  openbee --wallet lock");
    println!("    Clear all decrypted keys from memory.");
    println!();
    println!("  openbee --wallet address [chain]");
    println!("    Display wallet addresses. Optionally filter by chain.");
    println!();
    println!("  openbee --wallet send <chain> <to_address> <amount>");
    println!("    Build and sign a transaction (requires password).");
    println!("    Example: openbee --wallet send ethereum 0x1234...abcd 0.1");
    println!();
    println!("  openbee --wallet status");
    println!("    Show wallet info and account summary.");
    println!();
    println!("IMPORTANT:");
    println!("  - DO NOT store large amounts in this wallet");
    println!("  - Use a hardware wallet (Ledger/Trezor) for large holdings");
    println!("  - BACK UP your mnemonic phrase on paper");
    println!("  - Ensure your computer is free of malware before using");
}
