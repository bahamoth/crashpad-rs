# Development Conventions

Core development standards and practices for crashpad-rs.

## Quick Policy Matrix

| Category | Tool | Config | Enforcement | CI Check |
|----------|------|--------|-------------|----------|
| Code Format | rustfmt | Default | PR Required | ✅ |
| Linting | clippy | Default + `-D warnings` | PR Required | ✅ |
| Testing | cargo nextest | Cargo.toml | PR Required | ✅ |
| Commits | Conventional | release-please | Automated | ✅ |
| Dependencies | cargo/submodules | build.rs | Build step | ✅ |
| Platform Tests | GitHub Actions | Per-platform | PR Required | ✅ |

## Code Style

### General Principles
- **Rust Wrapper**: Prefer safe Rust abstractions over unsafe FFI
- **Clear Boundaries**: Separate sys crate (FFI) from safe wrapper
- **Cross-Platform**: Test on all supported platforms before merge
- **Build Reproducibility**: All dependencies managed through build.rs

### Rust Conventions

#### Formatting (rustfmt)
```bash
# Format all Rust code
cargo fmt --all

# Check formatting without changes
cargo fmt --all -- --check
```

#### Linting (clippy)
```bash
# Run clippy with all targets
cargo clippy --all-targets --all-features -- -D warnings

# Fix clippy warnings automatically
cargo clippy --fix
```

#### Code Organization
- `crashpad-sys/`: Raw FFI bindings (publishes as `crashpad-rs-sys`)
- `crashpad/`: Safe Rust API (publishes as `crashpad-rs`)
- `xtask/`: Build tooling and automation (not published)
- Use `thiserror` for error types (workspace dependency)

#### FFI Safety Rules
- All unsafe code must have safety comments
- Prefer `CString` over raw pointers for strings
- Check null pointers before dereferencing
- Use `std::mem::ManuallyDrop` for C++ managed resources

### C++ Conventions

#### Files
- `wrapper.h`: C API declarations
- `crashpad_wrapper.cc`: C++ to C bridge implementation

#### Naming
- C functions: `crashpad_` prefix (e.g., `crashpad_init`)
- Opaque types: Forward declarations only in header
- Resource management: Always provide cleanup functions

## Git Conventions

### Branch Naming
- Feature: `feature/description-kebab-case`
- Fix: `fix/issue-description`
- Refactor: `refactor/component-name`

### Commit Messages
Follow Conventional Commits format:

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature (e.g., platform support)
- `fix`: Bug fix
- `refactor`: Code restructuring
- `build`: Changes to build.rs or Makefile
- `test`: Adding or updating tests
- `docs`: Documentation changes
- `chore`: Maintenance tasks

Examples:
- `feat(android): add Android NDK cross-compilation support`
- `fix(ios): resolve in-process handler initialization`
- `build: replace depot_tools with Git submodules`
- `refactor: simplify Makefile platform detection`

### Merge Strategy
- **Prefer rebase**: Keep linear history
- **Fast-forward when possible**: No merge commits for simple changes
- **Squash small commits**: One logical change per commit
- **Delete branches after merge**: Keep repository clean


## Dependencies

- **Rust**: Workspace dependencies in root `Cargo.toml`
- **Native**: Git submodules in `third_party/`
- **Build tools**: Auto-downloaded by `build.rs`
- **Updates**: Security immediate, features quarterly

## Documentation

### Rust Doc Comments
```rust
/// Brief description
///
/// # Safety
/// Conditions for safe usage (unsafe functions only)
pub fn function_name() -> Result<()> {
    // Implementation
}
```

### Generate Docs
```bash
cargo doc --open
```

## Security & Compliance

### Security Practices
- No secrets in code or commits
- Validate all FFI inputs
- Handle panics at FFI boundary
- Clear sensitive data from memory

### Dependency Scanning
```bash
# Check for known vulnerabilities
cargo audit

# Update vulnerable dependencies
cargo update
cargo audit fix
```

## Pre-Release Checklist

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo nextest run`
- [ ] CI checks pass (automatic on PR)
- [ ] Version/CHANGELOG (automatic via release-please)


## Related Documents

- [DEVELOPING.md](./DEVELOPING.md) - Build, test, and development setup
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Technical design
- [README.md](./README.md) - Usage and integration