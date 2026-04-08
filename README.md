# OpenBee 🐝

**A hardworking AI Agent platform built in Rust** — secure crypto wallet, browser automation, extensible task system.

> 73 source files | 12,607 lines of Rust | 7 crates | 147 tests | 0 unsafe

## Quick Start

```bash
# Interactive mode — talk to the agent
openbee -i

# Run a single task
openbee check system status
openbee help me with my wallet

# Crypto wallet
openbee --wallet create
openbee --wallet address ethereum
openbee --wallet send ethereum 0xABC... 0.1
```

## Architecture

```
crates/
├── openbee_agent/     Task system: natural language → skill routing → execution
├── openbee_wallet/    Crypto wallet: 8 chains, 4 DEX, 6-layer security
├── openbee_browser/   Browser automation: Chrome CDP, AI page reading
├── openbee_core/      Shared: events, security sandbox, i18n, tween, profiler
├── openbee_scripting/ Lua 5.4 scripting for custom skills
├── openbee_net/       TCP networking for agent communication
└── openbee_plugin/    Plugin system for third-party skills
```

## Features

### 🤖 AI Agent (`openbee_agent`)

- **Natural language task routing** — say what you want, the agent finds the right skill
- **Skill-based architecture** — pluggable skills (wallet, browser, system, custom)
- **Auto or manual execution** — immediate or queue-based
- **Priority scheduling** — Low / Normal / High / Urgent
- **Task history** — track results, duration, success rate
- **Interactive REPL** — `openbee -i` for conversational mode
- **Bilingual** — understands English and Chinese instructions

### 💰 Crypto Wallet (`openbee_wallet`)

**8 Chains:**

| Chain | ID | Token | DEX |
|-------|----|-------|-----|
| Ethereum | 1 | ETH | Uniswap V3 |
| BSC | 56 | BNB | PancakeSwap V3 |
| Polygon | 137 | MATIC | 1inch |
| Arbitrum | 42161 | ETH | 1inch |
| Optimism | 10 | ETH | 1inch |
| Base | 8453 | ETH | 1inch |
| Avalanche | 43114 | AVAX | 1inch |
| Solana | — | SOL | Jupiter |

**6-Layer Security:**
1. **Default OFF** — must accept security warning + type confirmation phrase
2. **AES-256-GCM** — encrypted at rest, authenticated (tamper-proof)
3. **Argon2id KDF** — password-derived keys, anti GPU/ASIC brute-force
4. **Machine binding** — encryption key includes hardware fingerprint
5. **Auto-wipe** — 5 wrong passwords → keystore destroyed
6. **Zeroize** — all keys scrubbed from memory on drop

**ERC-20 Approval Security:**
- Never unlimited — exact amount only
- Zero-then-set pattern (anti race condition)
- Auto-revoke after swap

**Token Presets:** ETH, BNB, SOL, USDC, USDT, WETH, WBNB, WSOL, BUSD, CAKE, MATIC, AVAX, ARB

### 🌐 Browser Automation (`openbee_browser`)

- **Chrome DevTools Protocol** — connect to user's running browser
- **AI page reader** — extract AI responses from any web AI service
- **HTML parser** — tag/class extraction, code block detection
- **Session manager** — auto-reconnection

```bash
# User starts Chrome with debug port
chrome --remote-debugging-port=9222

# Agent reads AI responses from the browser
```

### 🔒 Security (`openbee_core::security`)

- **Filesystem sandbox** — ALL file I/O restricted to approved directories
- **Path traversal protection** — canonicalization, `..` blocking, null byte rejection
- **Sensitive file blocking** — .env, .ssh, .git, credentials, private_key
- **Permission levels** — ReadOnly vs ReadWrite per directory

### 🌍 i18n — 5 Languages

English, 中文, 日本語, 한국어, Español — 30+ strings per language

### 🔌 Extensibility

- **Lua scripting** (`openbee_scripting`) — write custom skills in Lua 5.4
- **Plugin system** (`openbee_plugin`) — load third-party skill packages
- **Network layer** (`openbee_net`) — TCP client/server for agent collaboration
- **Tween engine** — 31 easing functions for UI animations

## Building

```bash
cargo build              # Debug
cargo build --release    # Optimized
cargo test               # Run 147 tests
make help                # See all targets
```

## Platforms

| Platform | Architecture |
|----------|-------------|
| Linux | x86_64, ARM64 |
| Windows | x86_64, ARM64 |
| macOS | Intel, Apple Silicon |
| Web | WASM |

## License

Apache License 2.0
