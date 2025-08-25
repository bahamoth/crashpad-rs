# justfile for crashpad-rs
# Cross-platform command runner (replacement for Makefile)

# Use PowerShell on Windows
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Default recipe - show available commands
default:
    @just --list

# Clean all build artifacts and caches
clean:
    cargo clean
    {{ if os() == "windows" { "if (Test-Path target) { Remove-Item -Recurse -Force target }; if (Test-Path $env:LOCALAPPDATA\\crashpad-rs) { Remove-Item -Recurse -Force $env:LOCALAPPDATA\\crashpad-rs }" } else { "rm -rf target ~/.cache/crashpad-rs" } }}

# Build the project (debug mode)
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run tests
test:
    cargo test --lib

# Run tests with nextest for better isolation
test-nextest:
    cargo nextest run

# Package the crates for distribution
dist:
    cargo xtask dist

# Install development tools (just, nextest, ndk)
install-tools:
    cargo xtask install-tools

# Update submodules to match Crashpad's DEPS
update-deps:
    cargo xtask update-deps

# Create symlinks for Crashpad dependencies
symlink:
    cargo xtask symlink

# Build prebuilt package for current platform
prebuilt:
    cargo xtask build-prebuilt

# Build prebuilt package for specific target
prebuilt-target target:
    cargo xtask build-prebuilt --target {{target}}

# Build and test with prebuilt feature
test-prebuilt:
    cargo build --package crashpad-rs-sys --features prebuilt --no-default-features
    cargo test --package crashpad-rs-sys --features prebuilt --no-default-features

# Format code
fmt:
    cargo fmt --all

# Run clippy lints
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Check everything (format, clippy, build, test)
check: fmt clippy build test

# Build for Android (requires cargo-ndk)
android-build target="arm64-v8a":
    cargo ndk -t {{target}} build --package crashpad-rs-sys

# Build for iOS
ios-build:
    cargo build --target aarch64-apple-ios

# Build for iOS simulator
ios-sim-build:
    cargo build --target aarch64-apple-ios-sim

# Run example
run-example:
    cargo run --example crashpad_test_cli

# Build documentation
doc:
    cargo doc --no-deps --open

# Show build output directory for debugging
show-out-dir:
    {{ if os() == "windows" { "Get-ChildItem target\\*\\build\\crashpad-rs-sys-*\\out -ErrorAction SilentlyContinue | Select-Object FullName" } else { "ls -la target/debug/build/crashpad-rs-sys-*/out 2>/dev/null || ls -la target/release/build/crashpad-rs-sys-*/out 2>/dev/null || echo 'No build output found'" } }}