# Changelog

## [0.2.1](https://github.com/bahamoth/crashpad-rs/compare/v0.2.0...v0.2.1) (2025-08-13)


### Bug Fixes

* fix version bump ([3698268](https://github.com/bahamoth/crashpad-rs/commit/3698268c35a1238dee5e9095276876f196031ea5))

## [0.2.0](https://github.com/bahamoth/crashpad-rs/compare/v0.1.1...v0.2.0) (2025-08-13)


### ⚠ BREAKING CHANGES

* Submodules moved from third_party/ to crashpad-sys/third_party/
* Android builds now require NDK r23+ and symlink creation

### Features

* add complete CI/CD pipeline with GitHub Actions ([7acaf34](https://github.com/bahamoth/crashpad-rs/commit/7acaf34048075944c1a11f003dafcf16502cddab))
* add iOS simulator support with in-process handler ([d7aa5d0](https://github.com/bahamoth/crashpad-rs/commit/d7aa5d00e39e1dc47f71da9731095e7830ed24f6))
* **build:** add native dependencies version pinning ([743596b](https://github.com/bahamoth/crashpad-rs/commit/743596bce7a3180f336fd93b2e258c00ec36c736))
* **build:** complete macOS/iOS build fixes and add development guide ([e475ce7](https://github.com/bahamoth/crashpad-rs/commit/e475ce711f4651d995033f4bddf84503ee85ace6))
* **build:** refactor build system with modular phases and Android APK support ([9cdb1e8](https://github.com/bahamoth/crashpad-rs/commit/9cdb1e845855419754a647fb07cf31134a7069c6))
* **build:** replace depot_tools with Git submodules and native Rust build ([91bc9c0](https://github.com/bahamoth/crashpad-rs/commit/91bc9c03a86e982c89ffa49ea0a33ccbd4bacc50))
* **ci:** complete CI/CD pipeline with automated testing and publishing ([ed3349c](https://github.com/bahamoth/crashpad-rs/commit/ed3349ceb2fbb3c01fcb09fd538995a419c3754f))
* Implement working Crashpad Rust bindings ([cd94e9a](https://github.com/bahamoth/crashpad-rs/commit/cd94e9aa622bc6cd1ef4b36bdc3f746964820d9c))
* major refactoring and Android cross-compilation support ([6c6f6db](https://github.com/bahamoth/crashpad-rs/commit/6c6f6dbc7529119dff3eda09b361d8435831ca02))


### Bug Fixes

* bump version to 0.1.1 ([fe66c24](https://github.com/bahamoth/crashpad-rs/commit/fe66c24cde0f62fa93013857c5701ba388f2d71f))
* conditionally link mig_output only on macOS/iOS platforms ([9002eba](https://github.com/bahamoth/crashpad-rs/commit/9002ebaefbd585edad3c18774358ce3760600156))
* fix version bump ([56780fc](https://github.com/bahamoth/crashpad-rs/commit/56780fc3ca90357f3b9b5470473bd8e0246c94a1))
* resolve all clippy warnings and format code ([ce81deb](https://github.com/bahamoth/crashpad-rs/commit/ce81deb910c23e70c1d339090cb8fc96b531ff80))
* resolve Linux asynchronous_start test failures ([f1e3007](https://github.com/bahamoth/crashpad-rs/commit/f1e30074e6b7805228d9b33bc4cb3f0124883157))
* restructure integration tests to handle global state ([9f979c7](https://github.com/bahamoth/crashpad-rs/commit/9f979c784f73bff333da9f20e55e2abe7ce9670c))


### Code Refactoring

* reorganize project structure for crates.io publishing ([3fa4b8c](https://github.com/bahamoth/crashpad-rs/commit/3fa4b8c8a30301c1bd4825ae1de47d4ac70a5651))
