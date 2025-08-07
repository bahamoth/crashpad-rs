# Task: T-006 - Refactor crashpad-sys build system for platform consistency
Agent: architect-thor
Date: 2025-08-07

## Pre-Implementation
- [ ] Read ARCHITECTURE.md FFI sections
- [ ] Read PRD.md for requirements
- [ ] Review current build.rs implementation
- [ ] Analyze build_refactor_plan.md proposal
- [ ] Identify platform-specific build issues
- [ ] Review GN/Ninja build flow

## Design Phase
- [ ] Create unified PlatformBuildConfig structure
- [ ] Design phase-based build system
- [ ] Plan configuration validation strategy
- [ ] Define clear separation of concerns
- [ ] Document build phases and dependencies

## Implementation Plan
- TBD
## Key Improvements
- [ ] Unify GN args and wrapper compiler settings
- [ ] Fix compiler/linker mismatch issues
- [ ] Implement proper Android NDK handling
- [ ] Add iOS simulator configuration
- [ ] Ensure Linux PIC flags consistency
- [ ] Add Windows cross-compilation support

## Platform-Specific Testing
- [ ] Linux native build
- [ ] macOS native build
- [ ] Android cross-compilation (arm64-v8a)
- [ ] iOS simulator build
- [ ] Windows cross-compilation (MinGW)

## Build Validation
- [ ] Clean native artifacts: `make clean`
- [ ] Rebuild: `cargo build --package crashpad-sys`
- [ ] Test wrapper: `cargo build --package crashpad`
- [ ] Run example: `cargo run --example crashpad_test_cli`
- [ ] Verify phase caching works

## Documentation
- [ ] Update build system documentation
- [ ] Add platform configuration guide
- [ ] Document phase caching mechanism
- [ ] Update CONVENTIONS.md if needed

## Pre-Commit Validation
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Run `cargo test`
- [ ] Self-review changes

## Finalization
- [ ] Update TASKS.md status to REVIEW
- [ ] Create PR with detailed description
- [ ] Link PR to T-006 in TASKS.md
- [ ] Document any follow-up tasks needed