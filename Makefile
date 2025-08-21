.PHONY: clean clean-cache build rebuild dist test help

# Platform detection
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# OS detection
ifeq ($(UNAME_S),Linux)
    OS := linux
else ifeq ($(UNAME_S),Darwin)
    OS := macos
else
    OS := unknown
endif

# Architecture detection (using same naming as build.rs)
ifeq ($(UNAME_M),x86_64)
    ARCH := x64
else ifeq ($(UNAME_M),aarch64)
    ARCH := arm64
else ifeq ($(UNAME_M),arm64)
    ARCH := arm64
else
    ARCH := unknown
endif

PLATFORM := $(OS)-$(ARCH)
HANDLER_NAME := crashpad_handler
DIST_DIR := dist

build:
	cargo build --package crashpad-rs-sys

build-release:
	cargo build --release --package crashpad-rs-sys
	cargo build --release --package crashpad

clean:
	# Clean build artifacts
	rm -rf target/
	# Clean binary tool cache (optional, commented out by default)
	# rm -rf ~/.crashpad-cache
	rm -rf crashpad-sys/third_party/crashpad/third_party/
	cargo clean

rebuild: clean build

# Clean binary tool cache (GN and Ninja)
clean-cache:
	@echo "Cleaning binary tool cache..."
	@if [ "$$(uname -s)" = "Darwin" ]; then \
		rm -rf ~/Library/Caches/crashpad-cache; \
	elif [ "$$(uname -s)" = "Linux" ]; then \
		rm -rf ~/.cache/crashpad-cache; \
	else \
		echo "Windows: Please manually delete %LOCALAPPDATA%\\crashpad-cache"; \
	fi
	@echo "✓ Binary tool cache cleared"

# Create distribution package with crashpad_handler
dist: build-release
	@echo "Creating distribution package for platform: $(PLATFORM)"
	@mkdir -p $(DIST_DIR)/lib
	@mkdir -p $(DIST_DIR)/include
	@mkdir -p $(DIST_DIR)/bin
	
	# Copy crashpad_handler - check multiple possible locations
	@HANDLER_FOUND=0; \
	for HANDLER_PATH in \
		"target/release/$(HANDLER_NAME)" \
		"target/$(shell rustc -vV | sed -n 's/host: //p')/release/$(HANDLER_NAME)" \
		"target/*/release/crashpad_build/$(HANDLER_NAME)" \
		"third_party/crashpad_checkout/crashpad/out/$(PLATFORM)/$(HANDLER_NAME)"; do \
		if [ -f "$$HANDLER_PATH" ]; then \
			cp "$$HANDLER_PATH" $(DIST_DIR)/bin/; \
			chmod +x $(DIST_DIR)/bin/$(HANDLER_NAME); \
			echo "✓ Copied crashpad_handler from $$HANDLER_PATH to dist/bin/"; \
			HANDLER_FOUND=1; \
			break; \
		fi \
	done; \
	if [ $$HANDLER_FOUND -eq 0 ]; then \
		echo "ERROR: crashpad_handler not found in any expected location"; \
		echo "Make sure to build crashpad-rs-sys first"; \
		exit 1; \
	fi
	
	# Copy Rust libraries
	@for lib in target/release/libcrashpad*.rlib target/release/libcrashpad*.a; do \
		if [ -f "$$lib" ]; then \
			cp "$$lib" $(DIST_DIR)/lib/; \
			echo "✓ Copied library: $$(basename $$lib)"; \
		fi \
	done
	
	# Copy header files
	@if [ -f "crashpad-sys/wrapper.h" ]; then \
		cp crashpad-sys/wrapper.h $(DIST_DIR)/include/crashpad_wrapper.h; \
		echo "✓ Copied header: crashpad_wrapper.h"; \
	fi
	
	# Create README for distribution
	@echo "# Crashpad-rs Distribution Package" > $(DIST_DIR)/README.md
	@echo "" >> $(DIST_DIR)/README.md
	@echo "Platform: $(PLATFORM)" >> $(DIST_DIR)/README.md
	@echo "" >> $(DIST_DIR)/README.md
	@echo "## Directory Structure" >> $(DIST_DIR)/README.md
	@echo "" >> $(DIST_DIR)/README.md
	@echo '```' >> $(DIST_DIR)/README.md
	@echo "dist/" >> $(DIST_DIR)/README.md
	@echo "├── lib/          # Rust libraries (.rlib, .a)" >> $(DIST_DIR)/README.md
	@echo "├── include/      # C/C++ header files" >> $(DIST_DIR)/README.md
	@echo "├── bin/          # Executables (crashpad_handler)" >> $(DIST_DIR)/README.md
	@echo "└── README.md     # This file" >> $(DIST_DIR)/README.md
	@echo '```' >> $(DIST_DIR)/README.md
	@echo "" >> $(DIST_DIR)/README.md
	@echo "## Contents" >> $(DIST_DIR)/README.md
	@echo "" >> $(DIST_DIR)/README.md
	@echo "- bin/$(HANDLER_NAME) - The Crashpad handler executable" >> $(DIST_DIR)/README.md
	@echo "- lib/libcrashpad*.rlib - Rust library files" >> $(DIST_DIR)/README.md
	@echo "- include/crashpad_wrapper.h - C API header" >> $(DIST_DIR)/README.md
	@echo "" >> $(DIST_DIR)/README.md
	@echo "## Deployment" >> $(DIST_DIR)/README.md
	@echo "" >> $(DIST_DIR)/README.md
	@echo "When deploying your application:" >> $(DIST_DIR)/README.md
	@echo "1. Copy bin/$(HANDLER_NAME) to the same directory as your executable" >> $(DIST_DIR)/README.md
	@echo "2. Or install it system-wide in /usr/local/bin (Unix) or Program Files (Windows)" >> $(DIST_DIR)/README.md
	@echo "3. Or set CRASHPAD_HANDLER environment variable to its location" >> $(DIST_DIR)/README.md
	
	@echo ""
	@echo "✓ Distribution package created at: $(DIST_DIR)/"
	@echo "  Platform: $(PLATFORM)"
	@echo ""
	@echo "Directory structure:"
	@echo "  lib/      - Rust libraries"
	@echo "  include/  - Header files" 
	@echo "  bin/      - crashpad_handler executable"

test:
	cargo test

# Help target
help:
	@echo "Available targets:"
	@echo "  make build         - Build the project (debug mode)"
	@echo "  make build-release - Build the project (release mode)"
	@echo "  make clean         - Clean all build artifacts"
	@echo "  make clean-cache   - Clean binary tool cache (GN/Ninja)"
	@echo "  make rebuild       - Clean and rebuild"
	@echo "  make dist          - Create distribution package with crashpad_handler"
	@echo "  make test          - Run tests"
	@echo "  make help          - Show this help message"