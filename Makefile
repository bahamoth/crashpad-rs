.PHONY: clean build rebuild dist

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
	cargo build --package crashpad-sys

build-release:
	cargo build --release --package crashpad-sys
	cargo build --release --package crashpad

clean:
	rm -rf third_party
	rm -rf $(DIST_DIR)
	cargo clean

rebuild: clean build

# Create distribution package with crashpad_handler
dist: build-release
	@echo "Creating distribution package for platform: $(PLATFORM)"
	@mkdir -p $(DIST_DIR)/lib
	@mkdir -p $(DIST_DIR)/include
	@mkdir -p $(DIST_DIR)/bin
	
	# Copy crashpad_handler
	@if [ -f "third_party/crashpad_checkout/crashpad/out/$(PLATFORM)/$(HANDLER_NAME)" ]; then \
		cp third_party/crashpad_checkout/crashpad/out/$(PLATFORM)/$(HANDLER_NAME) $(DIST_DIR)/bin/; \
		chmod +x $(DIST_DIR)/bin/$(HANDLER_NAME); \
		echo "✓ Copied crashpad_handler to dist/bin/"; \
	else \
		echo "ERROR: crashpad_handler not found at third_party/crashpad_checkout/crashpad/out/$(PLATFORM)/$(HANDLER_NAME)"; \
		echo "Make sure to build crashpad-sys first"; \
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
	@echo "  make rebuild       - Clean and rebuild"
	@echo "  make dist          - Create distribution package with crashpad_handler"
	@echo "  make test          - Run tests"
	@echo "  make help          - Show this help message"