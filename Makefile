# ==============================================================================
# KatanA Dual-App Build & Packaging Makefile
# ==============================================================================
# One-click build and installation for both KatanA and KatanB macOS applications.
# Both apps run simultaneously with isolated configs, caches, and distinctive icons.
# ==============================================================================

.PHONY: all help build icons package install run run-a run-b stop clean test

# Default target: One-click build and packaging
all: build icons package

help:
	@echo "======================================================================="
	@echo " KatanA Dual-App One-Click Build System"
	@echo "======================================================================="
	@echo "  make (or make all)   Build binary, generate gold icon, and package both apps"
	@echo "  make build           Compile release binary (target/release/KatanA)"
	@echo "  make icons           Generate KatanB dark gold icon (target/icon_b.icns)"
	@echo "  make package         Package and install KatanA.app & KatanB.app"
	@echo "  make install         Alias for 'make all'"
	@echo "  make run             Launch both KatanA and KatanB simultaneously"
	@echo "  make run-a           Launch KatanA"
	@echo "  make run-b           Launch KatanB"
	@echo "  make stop            Close any running KatanA or KatanB instances"
	@echo "  make test            Run katana-ui test suite"
	@echo "  make clean           Clean build target and temporary icon files"
	@echo "======================================================================="

build:
	@echo "==> [1/3] Compiling release binary..."
	cargo build --release --package katana-ui --bin KatanA

icons:
	@echo "==> [2/3] Generating KatanB dark gold icon..."
	python3 scripts/build/generate_gold_icon.py

package:
	@echo "==> [3/3] Packaging and signing dual applications..."
	./scripts/build/package-dual-mac.sh

install: all

run:
	@echo "==> Launching KatanA and KatanB simultaneously..."
	@open -a /Applications/KatanA.app
	@open -a /Applications/KatanB.app

run-a:
	@echo "==> Launching KatanA..."
	@open -a /Applications/KatanA.app

run-b:
	@echo "==> Launching KatanB..."
	@open -a /Applications/KatanB.app

stop:
	@echo "==> Closing any running KatanA / KatanB instances..."
	@pkill -f "/Applications/Katan[A|B].app" 2>/dev/null || true

test:
	@echo "==> Running katana-ui tests..."
	cargo test -p katana-ui

clean:
	@echo "==> Cleaning artifacts..."
	cargo clean
	rm -rf target/icon_b.icns target/icon_b.icns.iconset /tmp/icon_*.icns /tmp/icon_*.iconset /tmp/icon_katana_*.png
