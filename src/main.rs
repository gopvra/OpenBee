// OpenBee - Rust reimplementation of Captain Claw (1997)
//
// A complete, modular game engine reimplementing the classic platformer
// with modern enhancements: Lua scripting, level editor, multiplayer,
// mod support, and more.

use anyhow::Result;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

use openbee_game::game_app::{BeeGameApp, GameConfig, Difficulty};

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("OpenBee v{} starting up...", env!("CARGO_PKG_VERSION"));
    info!("Rust reimplementation of Captain Claw (1997)");

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let mode = parse_launch_mode(&args);

    match mode {
        LaunchMode::Game(config) => run_game(config),
        LaunchMode::Editor => run_editor(),
        LaunchMode::Server(addr, port) => run_server(addr, port),
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
    Help,
}

fn parse_launch_mode(args: &[String]) -> LaunchMode {
    if args.len() < 2 {
        return LaunchMode::Game(GameConfig::default());
    }

    match args[1].as_str() {
        "--editor" | "-e" => LaunchMode::Editor,
        "--server" | "-s" => {
            let addr = args.get(2).cloned().unwrap_or_else(|| "0.0.0.0".to_string());
            let port = args.get(3)
                .and_then(|p| p.parse().ok())
                .unwrap_or(27015);
            LaunchMode::Server(addr, port)
        }
        "--help" | "-h" => LaunchMode::Help,
        "--fullscreen" | "-f" => {
            let config = GameConfig { fullscreen: true, ..Default::default() };
            LaunchMode::Game(config)
        }
        "--windowed" | "-w" => {
            let config = GameConfig { fullscreen: false, ..Default::default() };
            LaunchMode::Game(config)
        }
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
        _ => {
            let config = GameConfig::default();
            LaunchMode::Game(config)
        }
    }
}

fn run_game(config: GameConfig) -> Result<()> {
    info!("Starting game mode...");
    info!("Window: {}x{}, Fullscreen: {}, VSync: {}",
        config.window_width, config.window_height,
        config.fullscreen, config.vsync);

    let mut app = BeeGameApp::new(config)?;
    app.initialize()?;

    info!("Game initialized successfully. Entering main loop...");

    // Main game loop
    let mut last_time = std::time::Instant::now();
    let target_frame_time = std::time::Duration::from_secs_f64(1.0 / 60.0);

    while app.running {
        let now = std::time::Instant::now();
        let dt = now.duration_since(last_time).as_secs_f64();
        last_time = now;

        if let Err(e) = app.run_frame(dt) {
            error!("Frame error: {}", e);
        }

        // Frame rate limiting
        let frame_time = std::time::Instant::now().duration_since(now);
        if frame_time < target_frame_time {
            std::thread::sleep(target_frame_time - frame_time);
        }
    }

    app.shutdown();
    info!("Game shut down cleanly.");
    Ok(())
}

fn run_editor() -> Result<()> {
    info!("Starting level editor mode...");

    let editor = openbee_editor::EditorApp::new();
    info!("Level editor initialized.");

    // Editor would run its own event loop here
    info!("Editor app created. GUI event loop would start here.");
    drop(editor);

    Ok(())
}

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

        // Server tick loop
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

fn print_help() {
    println!("OpenBee - Rust reimplementation of Captain Claw (1997)");
    println!();
    println!("USAGE:");
    println!("    openbee [OPTIONS] [REZ_FILE]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help           Show this help message");
    println!("    -e, --editor         Launch the level editor");
    println!("    -s, --server [ADDR] [PORT]  Start a dedicated multiplayer server");
    println!("    -f, --fullscreen     Run in fullscreen mode");
    println!("    -w, --windowed       Run in windowed mode");
    println!("    -d, --difficulty [LEVEL]    Set difficulty (easy, normal, hard)");
    println!();
    println!("ARGUMENTS:");
    println!("    REZ_FILE             Path to CLAW.REZ game archive");
    println!();
    println!("EXAMPLES:");
    println!("    openbee                         Run with default settings");
    println!("    openbee CLAW.REZ                Run with specified REZ file");
    println!("    openbee --editor                Open the level editor");
    println!("    openbee --server 0.0.0.0 27015  Start a multiplayer server");
    println!("    openbee --fullscreen -d hard     Fullscreen, hard difficulty");
    println!();
    println!("KEYBINDINGS (default):");
    println!("    Arrow Keys / WASD    Move");
    println!("    Space / Z            Jump");
    println!("    Ctrl / X             Attack (sword)");
    println!("    1                    Pistol");
    println!("    2                    Dynamite");
    println!("    3                    Magic Claw");
    println!("    Escape               Pause / Menu");
    println!("    F1                   Toggle debug console");
    println!("    F11                  Toggle fullscreen");
    println!("    F12                  Screenshot");
}
