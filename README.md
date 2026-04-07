# OpenClaw-RS

A complete Rust reimplementation of [OpenClaw](https://github.com/pjasicek/OpenClaw) — the open-source engine for **Captain Claw** (1997), the classic 2D platformer by Monolith Productions.

This project is a "mega" engine that 1:1 replicates every feature of the original OpenClaw C++ engine, plus adds modern enhancements including a level editor, Lua scripting, multiplayer support, mod system, and more.

## Features

### Core Engine (1:1 OpenClaw Parity)
- Custom Entity-Component-System (ECS) architecture
- Rapier2D physics engine integration (replaces Box2D)
- SDL2-based rendering with sprite/animation support
- Full audio system with MIDI music and spatial sound
- Scene graph with camera, tile planes, and actor nodes
- CLAW.REZ archive parser and all asset format loaders (WWD, PID, PCX, ANI, PAL)
- 40+ game components matching the original engine
- 16 gameplay systems (physics, combat, AI, pickups, hazards, etc.)
- Enemy AI with finite state machines
- 4 boss fights (Aquatis, Gabriel, Marrow, Red Tail)
- 14 levels with checkpoints, secrets, and boss battles
- Save/load game system
- HUD, menus, debug console

### Enhanced Features (Beyond OpenClaw)
- **Level Editor** — Visual tile/actor editor with egui, undo/redo, live preview
- **Lua Scripting** — Custom game logic, moddable AI and events via Lua 5.4
- **Multiplayer** — Client-server networking with state sync, lobby, chat
- **Replay System** — Record and playback gameplay sessions
- **Mod Support** — Load custom assets, levels, and scripts from mod packages
- **Particle System** — Configurable visual effects (explosions, sparkles, etc.)
- **Gamepad Support** — Full controller mapping via gilrs
- **Remappable Controls** — User-configurable keybindings
- **Achievement System** — Track player progress and milestones
- **Localization** — Multi-language support (i18n)
- **Accessibility** — Difficulty settings, colorblind modes
- **Spatial Audio** — Positional 3D audio for immersive sound

## Architecture

```
crates/
├── openclaw_core/      # Core engine: ECS, physics, rendering, audio, input, events, scene
├── openclaw_rez/       # CLAW.REZ archive & asset format parsers
├── openclaw_game/      # Game logic: components, systems, AI, levels, UI
├── openclaw_scripting/ # Lua 5.4 scripting engine
├── openclaw_editor/    # Visual level editor (egui)
├── openclaw_net/       # Multiplayer networking & replay
└── openclaw_mod/       # Mod loading & asset override system
```

### Technology Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust 2021 Edition |
| Physics | rapier2d |
| Rendering | SDL2 (rust-sdl2) |
| Audio | rodio + midly |
| Input | SDL2 + gilrs (gamepad) |
| Scripting | mlua (Lua 5.4) |
| GUI/Editor | egui |
| Networking | tokio + bincode |
| Serialization | serde + quick-xml + serde_json |

## Building

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Run the game
cargo run -- CLAW.REZ

# Run the level editor
cargo run -- --editor

# Run a dedicated multiplayer server
cargo run -- --server 0.0.0.0 27015

# Run tests
cargo test
```

## Usage

```bash
openclaw [OPTIONS] [REZ_FILE]

OPTIONS:
    -h, --help                       Show help
    -e, --editor                     Launch level editor
    -s, --server [ADDR] [PORT]       Start multiplayer server
    -f, --fullscreen                 Fullscreen mode
    -w, --windowed                   Windowed mode
    -d, --difficulty [easy|normal|hard]  Set difficulty
```

## Requirements

- Rust 1.75+ (2021 edition)
- Original CLAW.REZ game archive (from Captain Claw, 1997)
- SDL2 development libraries

## License

Apache License 2.0
