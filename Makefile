# OpenBee Build Targets
# Usage: make [target]

.PHONY: all build release test check clippy fmt clean run editor server

# ─── Development ───

all: check test

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

editor:
	cargo run -- --editor

server:
	cargo run -- --server 0.0.0.0 27015

# ─── Quality ───

check:
	cargo check --workspace

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace -- -W clippy::all

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

# ─── Cross-platform builds ───

build-linux:
	cargo build --release --target x86_64-unknown-linux-gnu

build-linux-arm:
	cross build --release --target aarch64-unknown-linux-gnu

build-windows:
	cargo build --release --target x86_64-pc-windows-msvc

build-macos-intel:
	cargo build --release --target x86_64-apple-darwin

build-macos-arm:
	cargo build --release --target aarch64-apple-darwin

build-wasm:
	cargo build --release --target wasm32-unknown-unknown --lib -p openbee_core

build-all: build-linux build-windows build-macos-intel build-macos-arm build-wasm

# ─── Packaging ───

package-linux: build-linux
	mkdir -p dist/openbee-linux-x86_64
	cp target/x86_64-unknown-linux-gnu/release/openbee dist/openbee-linux-x86_64/
	cp README.md LICENSE assets/config.xml dist/openbee-linux-x86_64/
	cd dist && tar czf openbee-linux-x86_64.tar.gz openbee-linux-x86_64

package-windows: build-windows
	mkdir -p dist/openbee-windows-x86_64
	cp target/x86_64-pc-windows-msvc/release/openbee.exe dist/openbee-windows-x86_64/
	cp README.md LICENSE assets/config.xml dist/openbee-windows-x86_64/
	cd dist && zip -r openbee-windows-x86_64.zip openbee-windows-x86_64

# ─── Clean ───

clean:
	cargo clean
	rm -rf dist/

# ─── Info ───

stats:
	@echo "=== OpenBee Project Stats ==="
	@find crates -name "*.rs" | wc -l | xargs -I{} echo "Rust files: {}"
	@find crates -name "*.rs" -exec cat {} + | wc -l | xargs -I{} echo "Lines of code: {}"
	@cargo test --workspace 2>&1 | awk '/^test result/ {sum += $$4} END {print "Tests passing:", sum}'

loc:
	@echo "Lines of code per crate:"
	@for crate in openbee_core openbee_rez openbee_game openbee_scripting openbee_editor openbee_net openbee_mod; do \
		lines=$$(find crates/$$crate/src -name "*.rs" -exec cat {} + | wc -l); \
		printf "  %-20s %s lines\n" "$$crate" "$$lines"; \
	done

help:
	@echo "OpenBee Build System"
	@echo ""
	@echo "Development:"
	@echo "  make build      - Debug build"
	@echo "  make release    - Release build"
	@echo "  make run        - Run the game"
	@echo "  make editor     - Launch level editor"
	@echo "  make server     - Start multiplayer server"
	@echo ""
	@echo "Quality:"
	@echo "  make test       - Run all tests"
	@echo "  make clippy     - Run linter"
	@echo "  make fmt        - Format code"
	@echo ""
	@echo "Cross-platform:"
	@echo "  make build-linux       - Linux x86_64"
	@echo "  make build-linux-arm   - Linux ARM64"
	@echo "  make build-windows     - Windows x86_64"
	@echo "  make build-macos-intel - macOS Intel"
	@echo "  make build-macos-arm   - macOS Apple Silicon"
	@echo "  make build-wasm        - WebAssembly"
	@echo ""
	@echo "Info:"
	@echo "  make stats      - Project statistics"
	@echo "  make loc        - Lines of code per crate"
