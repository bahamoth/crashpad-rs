# CLAUDE.md

Claude Code guidance for crashpad-rs - Rust bindings for Google Crashpad.

## 🎯 Task Execution Workflow

### 1. Starting a Task
- **CHECK**: Read [TASKS.md](./TASKS.md) for available work
- **ACTION**: Claim task with your agent identifier (e.g., dev-vision)
- **CREATE**: Personal checklist file:
  ```bash
  mkdir -p .agent-checklists
  touch .agent-checklists/{your-role}-{your-name}-{task-id}.md
  ```
- **USE**: TodoWrite for session-based tracking

### 2. Before Implementation
- **CHECK**: Read [ARCHITECTURE.md](./ARCHITECTURE.md) for FFI design patterns
- **CHECK**: Read [PRD.md](./PRD.md) for requirements and roadmap
- **CHECK**: Read [CONVENTIONS.md](./CONVENTIONS.md) for coding standards
- **REVIEW**: Existing implementations in:
  - `crashpad-sys/` for FFI bindings
  - `crashpad/` for safe Rust wrapper
- **ACTION**: Plan your approach based on existing patterns

### 3. During Implementation

#### FFI Work (crashpad-sys)
- **FOLLOW**: bindgen patterns in `build.rs`
- **UPDATE**: `wrapper.h` for new C API declarations
- **IMPLEMENT**: C++ bridge in `crashpad_wrapper.cc`
- **PREFIX**: All C functions with `crashpad_`
- **DOCUMENT**: Safety requirements in comments

#### Safe Wrapper (crashpad)
- **WRAP**: Unsafe FFI calls with safe Rust API
- **USE**: `thiserror` for error types (workspace dependency)
- **CHECK**: Null pointers before dereferencing
- **FOLLOW**: Existing patterns in `src/client.rs` and `src/config.rs`

### 4. Before Commit ⚠️
- **CHECK**: Read [CONVENTIONS.md#code-style](./CONVENTIONS.md#code-style)
- **EXECUTE**: All required checks:
  ```bash
  cargo fmt --all
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test
  ```
- **ONLY THEN**: Create your commit following [Conventional Commits](./CONVENTIONS.md#commit-messages)

### 5. Cross-Platform Testing
- **Linux/macOS**: Standard cargo build
- **Android**: Use cargo-ndk (see [CONVENTIONS.md#cross-compilation](./CONVENTIONS.md#cross-compilation))
- **iOS**: Test with simulator example
- **Windows**: Cross-compile with MinGW

### 6. Before Creating PR
- **UPDATE**: [TASKS.md](./TASKS.md) with completion status
- **ENSURE**: Handler binary is built and tested
- **VERIFY**: Documentation is updated

## 📋 Standard Task Checklist Template

Create this in `.agent-checklists/{your-role}-{your-name}-{task-id}.md`:

```markdown
# Task: {task-id} - {task-description}
Agent: {your-role}-{your-name}
Date: {YYYY-MM-DD}

## Pre-Implementation
- [ ] Read ARCHITECTURE.md FFI sections
- [ ] Read PRD.md for requirements
- [ ] Review similar implementations
- [ ] Check platform-specific needs
- [ ] Plan FFI/Safe wrapper approach

## Implementation
- [ ] Write FFI bindings (if needed)
- [ ] Implement safe Rust wrapper
- [ ] Add safety documentation
- [ ] Write unit tests
- [ ] Test cross-compilation

## Build Validation
- [ ] Clean native artifacts: `make clean`
- [ ] Rebuild: `cargo build --package crashpad-sys`
- [ ] Build wrapper: `cargo build --package crashpad`
- [ ] Test handler: `cargo run --example crashpad_test_cli`

## Pre-Commit Validation
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Run `cargo test`
- [ ] Self-review changes

## Cross-Platform Check
- [ ] Linux build passes
- [ ] macOS build passes (if available)
- [ ] Android cross-compile works (if applicable)
- [ ] iOS simulator test passes (if applicable)

## Finalization
- [ ] Update TASKS.md status
- [ ] Create PR with proper description
- [ ] Link PR to task in TASKS.md
```

## 🏗️ Build System Notes

### Automatic Dependency Management
- depot_tools is auto-downloaded to `third_party/depot_tools`
- Crashpad source fetched via gclient to `third_party/crashpad_checkout`
- DO NOT commit `third_party/` contents (gitignored)

### Build Process
1. `build.rs` manages all native dependencies
2. Uses gclient for Chromium-style dependency management
3. Generates build files with `gn`
4. Compiles with `ninja`
5. bindgen creates FFI bindings from `wrapper.h`

### Common Build Issues
- **Link errors**: Run `make clean` then rebuild
- **Android "ar not found"**: Create NDK symlinks (see README)
- **depot_tools issues**: Delete `third_party/` and rebuild

## 🚨 Critical Rules

1. **FFI Safety**
   - ALWAYS add safety comments for unsafe code
   - Check all pointers from C++ before use
   - Use `CString` for string passing
   - Handle panics at FFI boundary

2. **Platform Testing**
   - Test on target platform before marking complete
   - iOS uses in-process handler (different from desktop)
   - Android requires NDK setup

3. **Documentation**
   - Update wrapper.h comments for bindgen
   - Document platform-specific behavior
   - Keep examples working

## 📚 Documentation Quick Reference

| Document | When to Read | Purpose |
|----------|--------------|---------|
| [OVERVIEW.md](./OVERVIEW.md) | First time on project | Project structure & quick start |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Before implementing | FFI design & system architecture |
| [CONVENTIONS.md](./CONVENTIONS.md) | Before commit/push | Coding standards & build rules |
| [PRD.md](./PRD.md) | Feature planning | Requirements & roadmap |
| [TASKS.md](./TASKS.md) | Start/end of work | Task management |
| [README.md](./README.md) | Library usage | User-facing documentation |

## 🤝 Multi-Agent Coordination

### Agent Identification
- Choose a unique name when starting
- Use format: `{role}-{name}` where:
  - `role`: Your function (dev, reviewer, tester, architect)
  - `name`: Your chosen unique identifier
  - Examples: `dev-vision`, `reviewer-jarvis`, `tester-friday`, `architect-ultron`
- Consistently use this identifier in all interactions

### Task Management
- **Persistent**: `.agent-checklists/` directory (commit with your code)
- **Session-only**: TodoWrite tool (not shared between agents)
- **Shared state**: TASKS.md (authoritative task status)

### Avoiding Conflicts
1. Always check TASKS.md before claiming work
2. Update your task status immediately when starting
3. Commit your checklist file to show progress
4. Communicate through PR comments

## 🔧 Development Commands

### Essential Commands
```bash
# Standard build
cargo build --package crashpad-sys
cargo build --package crashpad

# Clean and rebuild (for link errors)
make clean
cargo build

# Run tests
cargo test

# Format and lint
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings

# Create distribution
cargo xtask dist

# Run example
cargo run --example crashpad_test_cli
```

### Cross-Compilation
```bash
# Android (with cargo-ndk)
cargo ndk -t arm64-v8a build --package crashpad-sys

# iOS
cargo build --target aarch64-apple-ios

# iOS Simulator
cargo build --target aarch64-apple-ios-sim --example ios_simulator_test
```

## ⚠️ Platform-Specific Notes

### Android
- MUST create NDK symlinks first (see README top)
- Use cargo-ndk for simplified builds
- Limited to library builds (no standalone handler)

### iOS
- In-process handler (no separate executable)
- Different initialization pattern
- Test with simulator example

### Windows
- Cross-compile with MinGW from Linux
- Native builds require Visual Studio
- Handler must be distributed with .exe

## 📝 Commit Message Format

Follow [Conventional Commits](./CONVENTIONS.md#commit-messages):
```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

Examples:
- `feat(android): add Android NDK cross-compilation support`
- `fix(ios): resolve in-process handler initialization`
- `build: add depot_tools auto-download in build.rs`
- `refactor(sys): simplify platform detection logic`