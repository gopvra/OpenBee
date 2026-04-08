# OpenBee 🐝

**A hardworking bee engine** — a complete Rust reimplementation of [OpenClaw](https://github.com/pjasicek/OpenClaw), the open-source engine for **Captain Claw** (1997) by Monolith Productions.

OpenBee goes far beyond the original: it's a mega engine combining the best ideas from [Bevy](https://bevyengine.org/), [Godot](https://godotengine.org/), [OpenSurge](https://opensurge2d.org/), [NXEngine-evo](https://github.com/EXL/NXEngine), [Rigel Engine](https://github.com/lethal-guitar/RigelEngine), [Corgi Engine](https://corgi-engine.moremountains.com/), and Celeste/Hollow Knight game-feel techniques — all built from scratch in Rust.

> **214 source files | 25,000+ lines of Rust | 7 crates | 63 tests | 0 unsafe blocks**

---

## Architecture

```
crates/
├── openbee_core/       71 files — ECS, physics, rendering, audio, input, events, scene, security
├── openbee_rez/         7 files — CLAW.REZ archive & asset format parsers
├── openbee_game/      106 files — Components, systems, AI, levels, UI, game logic
├── openbee_scripting/  10 files — Lua 5.4 scripting engine
├── openbee_editor/      7 files — Visual level editor (egui)
├── openbee_net/         7 files — Multiplayer networking & replay
└── openbee_mod/         5 files — Mod loading & asset override
```

## Technology Stack

| Layer | Crate | Notes |
|-------|-------|-------|
| Physics | `rapier2d` | Replaces Box2D, 75 px/meter conversion |
| Rendering | SDL2 / WebGL | Multi-backend (SDL2, Web/WASM, Null) |
| Audio | `rodio` + `midly` | Sound effects + MIDI music + spatial 3D audio |
| Input | SDL2 + `gilrs` | Keyboard, mouse, gamepad, touch |
| Scripting | `mlua` (Lua 5.4) | Full game API bindings |
| Editor GUI | `egui` | Immediate-mode level editor |
| Networking | `tokio` + `bincode` | Async TCP client-server |
| Serialization | `serde` + `quick-xml` | Save files, configs, templates |
| Math | `glam` | Fast SIMD vector math |

---

## Complete Feature List

### 🎮 Core Engine (`openbee_core`)

**Custom ECS (Entity-Component-System)**
- Generational entity IDs with safe recycling
- Type-erased component storage (TypeId-based HashMap)
- System trait with ordered scheduler
- World: entity create/destroy, component add/remove/query

**Physics (Rapier2D)**
- Dynamic, static, kinematic rigid bodies
- Circle, box, capsule, polygon collision shapes
- Contact listener with begin/end collision events
- Force, impulse, velocity control
- Raycast queries
- Debug wireframe drawing
- 75 pixels-per-meter coordinate conversion

**Rendering**
- `Renderer` trait with multiple backends (SDL2, Web/WASM, Null)
- Sprite sheets with frame-based animation
- `AnimationPlayer` with looping, speed control, events
- Particle system (emitters, configurable lifetime/spread/color/gravity)
- Shader effects (Bloom, Grayscale, Blur, Invert, Custom)
- Screenshot capture (PNG/BMP/TGA)
- **Game Juice System** *(from Celeste/Hollow Knight)*:
  - Hit freeze / hitlag (frame-perfect impact feedback)
  - Squash & stretch deformation (jump, land, hit, bounce)
  - Ghost trail / afterimage effects
  - Hit flash (white/color flash on damage)
  - Slow motion / bullet time with ease in/out
  - Screen flash effects

**Audio**
- `AudioEngine` trait with Null fallback
- Sound effects (SoundId/SoundHandle)
- Music + MIDI playback (MusicId/MidiData)
- **Spatial audio** — positional 3D sound with distance attenuation and stereo panning

**Input**
- Keyboard (comprehensive KeyCode enum)
- Mouse (position, 5 buttons)
- Gamepad/controller via gilrs (buttons, axes, rumble)
- Touch input (multi-touch, gestures: pan, pinch, swipe)
- **Remappable keybindings** — action-based mapping, save/load configs

**Events**
- Type-erased publish/subscribe EventBus
- 19 game events (actor lifecycle, combat, items, levels, saves)
- 10 input events (keyboard, mouse, gamepad, touch)
- Immediate and queued dispatch modes

**Scene Graph**
- Hierarchical scene nodes (Actor, Tile, HUD, Particle)
- Camera with smooth follow, dead zone, look-ahead
- **Camera effects** *(from Corgi Engine)*:
  - Screen shake with decay
  - Zoom transitions
  - Letterbox / cinematic bars
  - Room-based camera constraints (Super Metroid style)
- Tile plane rendering with parallax scrolling

**Resource Management**
- `ResourceManager` with loader registry
- LRU cache with configurable memory budget
- Format loaders: ANI, PCX, PID, PNG, WAV, MIDI, PAL, WWD, XML
- **Hot reload** *(from Bevy)* — filesystem watcher for live asset/script reloading

**Tween Engine** *(from DOTween/Bevy Tweening)*
- **30 easing functions**: Linear, Sine, Quad, Cubic, Quart, Quint, Expo, Circ, Back, Elastic, Bounce (In/Out/InOut)
- Tween sequences and timelines
- Loop modes: None, Loop, PingPong
- Delay support
- TweenManager for managing multiple concurrent tweens

**Process System**
- Background process manager with chaining
- PowerupProcess for timed effects
- Process states: Running, Paused, Finished

**Security** 🔒
- **Filesystem sandbox** — ALL file I/O validated against user-approved directories
- Path traversal protection (canonicalization, null bytes, depth limits)
- Blocked sensitive patterns (.env, .ssh, .git, credentials, private keys)
- Permission levels: ReadOnly (assets) vs ReadWrite (saves)
- Symlink escape detection
- Global sandbox with `init_sandbox()` convenience

**Utilities**
- **Localization (i18n)** — multi-language support with English + Chinese (100+ strings)
- Template string substitution (`{name}` → value)
- Math helpers (lerp, clamp, angle conversion, rect collision)
- Performance profiler with scoped timers
- String utilities (path split, snake_case, FNV hash)
- Pixel/meter converters, hex color parsing

---

### 📦 Asset Parsers (`openbee_rez`)

| Format | Parser | Description |
|--------|--------|-------------|
| `.REZ` | `RezArchive` | Captain Claw resource archive (directory tree, file extraction) |
| `.WWD` | `WwdParser` | Level data (tile planes, object placement, properties) |
| `.PID` | `PidParser` | Proprietary indexed-color images (raw + RLE compressed) |
| `.PCX` | `PcxParser` | ZSoft PCX images with VGA palette |
| `.ANI` | `AniParser` | Animation frame sequences with timing/offsets |
| `.PAL` | `PalParser` | 256-color RGB palettes (index 0 = transparent) |

All parsers: binary format with `byteorder`, `thiserror` errors, 17 unit tests.

---

### 🎯 Game Logic (`openbee_game`)

**50+ Components** organized by category:

| Category | Components |
|----------|-----------|
| **Core** | Transform, Render, Animation, Physics, Kinematic, Collision, Controllable |
| **Combat** | Health, Destroyable, Explodeable, AmmoComponent |
| **Items** | Pickup (8 treasure types), Loot, Score, Life, Powerup (6 types) |
| **Inventory** | InventoryComponent (slots, equipment, gold, auto-pickup, rarity system) |
| **Movement** | PathElevator, ConveyorBelt, Rope, PredefinedMove (linear/sine/circular), Followable |
| **Advanced Movement** | WallJump, DoubleJump, Dash, WallSlide, Crouch/Crawl, Glide, LadderClimb, Swimming |
| **Platforms** | OneWayPlatform (directional, drop-through), SpringBoard, SteppingGround |
| **Hazards** | AreaDamage, FloorSpike, SawBlade, WaterZone, GravityZone (5 types), DestructibleTerrain |
| **Audio** | Sound, LocalAmbientSound, GlobalAmbientSound |
| **Triggers** | Trigger (8 types), BossStager, SoundTrigger |
| **Spawners** | ActorSpawner, ProjectileSpawner (8 projectile types) |
| **Effects** | Aura, Glitter, Checkpoint |

**22 ECS Systems:**

| System | Function |
|--------|----------|
| PhysicsSystem | Rapier2D simulation step + position sync |
| RenderSystem | Sprite/animation rendering |
| AnimationSystem | Frame advancement and looping |
| InputSystem | Player input → controllable components |
| **AdvancedMovementSystem** | Wall jumps, dashing, swimming, ladders, gliding |
| MovementSystem | Elevators, conveyors, ropes, predefined paths |
| AiSystem | Enemy AI state machine updates |
| CombatSystem | Hit detection, damage dealing, death |
| HazardSystem | Spikes, saws, area damage cycling |
| **WaterSystem** | Buoyancy, drag, breath, splash effects |
| **WeatherSystem** | Rain, snow, fog, thunder, wind physics |
| PickupSystem | Item collection and effect application |
| SpawnerSystem | Actor/projectile spawning with cooldowns |
| PowerupSystem | Timed powerup effects and expiry |
| CheckpointSystem | Activation and respawn tracking |
| TriggerSystem | Zone-based event triggering |
| **DestructibleSystem** | Terrain destruction, debris, respawn |
| **CutsceneSystem** | Scripted sequences (camera, actors, dialogue, effects) |
| **AchievementSystem** | 30+ achievements with progress tracking |
| **AccessibilitySystem** | Colorblind modes, difficulty assists, UI scaling |
| ScoreSystem | Point calculation and display |
| ParticleSystem | Particle effect updates |

**AI Systems:**

| AI Type | Description |
|---------|-------------|
| EnemyAI + FSM | 16 enemy types, 8 AI states (Idle → Patrol → Chase → Attack → Retreat) |
| **Behavior Tree** *(from Godot/Unreal)* | Sequence, Selector, Parallel, Random, Inverter, Repeat, Guard, Cooldown, Timeout nodes |
| **Boss Pattern DSL** *(from OpenSurge)* | Declarative boss attack language: projectile spreads, circles, aimed shots, movement, spawning, phases, conditions |
| PunkRat AI | Specialized charge-attack enemy |
| Toggle/Crumble Peg | Platform behavior AI |
| Projectile AI | Linear, arcing, homing, spiraling trajectories |

**4 Boss Fights with multi-phase AI:**
- **Aquatis** — Water attacks, whirlpools, minion spawning (4 phases)
- **Gabriel** — Teleport, magic, shield mechanics (3 phases)
- **Marrow** — Sword combat, parry, dark magic (3 phases)
- **Red Tail** — Acrobatic attacks, smoke bombs (3 phases)

**14 Levels:**
- TileMap with 6 collision types (None, Full, OneWay, Platform, Slope, Ladder)
- Level loading from WWD format
- Level transitions with state preservation
- Actor template registry for spawning

**UI:**

| Module | Features |
|--------|----------|
| GameHud | Health bar, score, lives, ammo display, minimap |
| MainMenu / PauseMenu / OptionsMenu | Keyboard navigation, settings |
| DebugConsole | 10+ commands (god, noclip, teleport, spawn, level, fps, quit) |
| ScoreScreen | Animated end-of-level tally |
| **DialogueSystem** *(from Corgi Engine)* | Dialogue trees, branching choices, typewriter text, portraits, flags |
| **SpeedrunTimer** | Splits, personal bests, delta display (HH:MM:SS.mmm) |
| **DebugOverlay** *(from Bevy Inspector)* | FPS graph, entity inspector, physics wireframe, AI paths, subsystem timing |

**Game State:**
- GameLogic with state machine (Menu, Playing, Paused, GameOver, Victory)
- Save/load (JSON serialization)
- Difficulty levels (Easy, Normal, Hard)
- Checkpoint system

---

### 📜 Lua Scripting (`openbee_scripting`)

- **LuaEngine** — Lua 5.4 VM via mlua
- **Script component** — attach scripts to entities
- **5 API modules:**
  - `Actor` — spawn, destroy, get/set position, health, components
  - `Physics` — apply_force, raycast, set_velocity
  - `Audio` — play_sound, play_music, volume control
  - `Input` — key/mouse/gamepad queries
  - `UI` — show_text, create_button, progress_bar
- **Rust↔Lua bindings:** Vec2, Entity, Color, Rect with metamethods

---

### 🎨 Level Editor (`openbee_editor`)

- **egui-based** visual editor with menus and toolbar
- **7 tools:** Paint, Erase, FloodFill, PlaceActor, Select, Move, Delete
- **Tile editor** — brush painting, erasing, flood fill, tile palette
- **Actor editor** — placement, selection, dragging, template palette
- **Property panel** — inspect/edit entity components
- **Undo/redo** — full action history stack
- **Live preview** — play-test levels in editor
- **Grid overlay** — configurable snap grid

---

### 🌐 Multiplayer (`openbee_net`)

- **Client-server** architecture over TCP
- **Network protocol** — Connect, Disconnect, Input, Snapshot, Chat, RPC messages
- **Client prediction** with server reconciliation (Quake-style)
- **Entity interpolation** — smooth remote entity movement
- **Lobby system** — player management, ready states, host migration, chat
- **Replay system** — record/playback with frame-by-frame input capture, seek, speed control

---

### 🔧 Mod System (`openbee_mod`)

- **Mod loader** — filesystem discovery, dependency resolution (topological sort)
- **Mod manifest** — JSON metadata (id, version, author, dependencies, conflicts)
- **Asset override** — Replace, Merge, or Skip modes for asset replacement
- **Mod registry** — enable/disable, load order management

---

## Building & Running

```bash
cargo build                          # Debug build
cargo build --release                # Release build
cargo run -- CLAW.REZ                # Play the game
cargo run -- --editor                # Level editor
cargo run -- --server 0.0.0.0 27015  # Multiplayer server
cargo run -- --fullscreen -d hard    # Fullscreen, hard mode
cargo test                           # Run all 63 tests
```

## Supported Platforms

| Platform | Architecture | Binary |
|----------|-------------|--------|
| Linux | x86_64 | `openbee-linux-x86_64.tar.gz` |
| Linux | ARM64 (Raspberry Pi) | `openbee-linux-aarch64.tar.gz` |
| Windows | x86_64 | `openbee-windows-x86_64.zip` |
| Windows | ARM64 | `openbee-windows-aarch64.zip` |
| macOS | Intel | `openbee-macos-x86_64.tar.gz` |
| macOS | Apple Silicon (M1/M2/M3) | `openbee-macos-aarch64.tar.gz` |
| Web | WebAssembly | `openbee-wasm.tar.gz` |

## CI/CD

Automated via **GitHub Actions**:

- **CI Pipeline** (`ci.yml`) — Runs on every push/PR:
  - `cargo check` — compilation verification
  - `cargo test` — all 63 tests
  - `cargo clippy` — lint analysis
  - `cargo fmt --check` — format verification

- **Release Pipeline** (`release.yml`) — Triggers on tag push (`v*.*.*`):
  - Builds all 7 platform binaries in parallel
  - Packages with README, LICENSE, config
  - Creates GitHub Release with changelog
  - Uploads all artifacts

### Creating a Release

```bash
git tag v0.1.0
git push origin v0.1.0
# → GitHub Actions builds 7 binaries and creates a Release automatically
```

### Cross-platform Build (local)

```bash
make build-linux        # Linux x86_64
make build-linux-arm    # Linux ARM64
make build-windows      # Windows x86_64
make build-macos-intel  # macOS Intel
make build-macos-arm    # macOS Apple Silicon
make build-wasm         # WebAssembly
```

## Requirements

- **Rust** 1.75+ (2021 edition)
- **CLAW.REZ** game archive (from original Captain Claw, 1997)
- **SDL2** development libraries (`apt install libsdl2-dev` on Linux)
- **ALSA** dev libraries (`apt install libasound2-dev` on Linux)

## Inspirations & Credits

This project draws ideas from many open-source projects:

| Project | What We Borrowed |
|---------|-----------------|
| [OpenClaw](https://github.com/pjasicek/OpenClaw) | game mechanics, component architecture, all 14 levels |
| [Bevy](https://bevyengine.org/) | ECS design, hot reload, debug inspector, tween engine |
| [Corgi Engine](https://corgi-engine.moremountains.com/) | Advanced movement, inventory, dialogue, camera rooms |
| [OpenSurge](https://opensurge2d.org/) | Scripting engine, boss pattern DSL |
| [Rigel Engine](https://github.com/lethal-guitar/RigelEngine) | Smooth 60fps scrolling, modern renderer |
| [NXEngine-evo](https://github.com/EXL/NXEngine) | Widescreen support, modding architecture |
| [Celeste](https://www.celestegame.com/) | Game juice (hit freeze, squash-stretch, trails) |
| [Hollow Knight](https://www.hollowknight.com/) | Screen shake, hit feedback, afterimage effects |
| [Godot](https://godotengine.org/) | Behavior trees, tween system, signal architecture |

## License

Apache License 2.0
