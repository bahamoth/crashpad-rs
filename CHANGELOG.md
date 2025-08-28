# Changelog

## [0.2.7](https://github.com/bahamoth/crashpad-rs/compare/v0.2.6...v0.2.7) (2025-08-28)


### Features

* add crashpad-handler-bundler for automatic handler distribution ([a517250](https://github.com/bahamoth/crashpad-rs/commit/a51725052a8939b868c9d33d69c04b4e15d46bf8))

## [0.2.6](https://github.com/bahamoth/crashpad-rs/compare/v0.2.5...v0.2.6) (2025-08-27)


### Features

* add Windows build support for crashpad-rs ([e2c895c](https://github.com/bahamoth/crashpad-rs/commit/e2c895cd5d249920258daec0bea02c7d950f1368))
* add Windows CI workflow and badge ([3337197](https://github.com/bahamoth/crashpad-rs/commit/3337197325182082e1d474b49e5ad87666a0d7e6))
* implement prebuilt support for Windows ([91596fd](https://github.com/bahamoth/crashpad-rs/commit/91596fd467c63ae88c86659f512811dbdbd98a39))


### Bug Fixes

* **build:** fix Android prebuilt archives and cross-compilation ([ef12037](https://github.com/bahamoth/crashpad-rs/commit/ef1203788d2b51830bcfee46ca6b31bb9ba136a8))
* **ci:** fix GitHub token authentication for update-deps workflow ([18cfd3c](https://github.com/bahamoth/crashpad-rs/commit/18cfd3caa8f0443b352db716b740181517611e67))

## [0.2.5](https://github.com/bahamoth/crashpad-rs/compare/v0.2.4...v0.2.5) (2025-08-19)


### Bug Fixes

* **docs:** fix release-please pattern and restore version badge format ([244e91a](https://github.com/bahamoth/crashpad-rs/commit/244e91a153adabf2070b17790297723ae9180d5b))
* **docs:** use GitHub release badge for version display ([2642920](https://github.com/bahamoth/crashpad-rs/commit/2642920a1998538e7215862260416c6b56dc3b34))
* **docs:** wrap version badge with release-please markers ([e7b85c5](https://github.com/bahamoth/crashpad-rs/commit/e7b85c5b46beb4b3bdfe2c9d309aa28a6635449f))

## [0.2.4](https://github.com/bahamoth/crashpad-rs/compare/v0.2.3...v0.2.4) (2025-08-16)


### Bug Fixes

* **docs:** add missing color suffix to version badge URL ([d84ea10](https://github.com/bahamoth/crashpad-rs/commit/d84ea10abeddaec0306cbcdcf0a606367425ab75))
* **docs:** correct GitHub Actions badge URLs and use absolute paths for documentation links ([498553c](https://github.com/bahamoth/crashpad-rs/commit/498553c049088db6aba1b661dfbedf37d38bf02d))

## [0.2.3](https://github.com/bahamoth/crashpad-rs/compare/v0.2.2...v0.2.3) (2025-08-15)


### Features

* add handler arguments API for crashpad configuration ([32af425](https://github.com/bahamoth/crashpad-rs/commit/32af425f58f5faf3891630d1f2fe7562dd3df4ef))

## [0.2.2](https://github.com/bahamoth/crashpad-rs/compare/v0.2.1...v0.2.2) (2025-08-13)


### Bug Fixes

* **build:** fix docs.rs build failure ([d3e60d4](https://github.com/bahamoth/crashpad-rs/commit/d3e60d4a0e994dd32e6fc8176c917eadc1840622))

## [0.2.1](https://github.com/bahamoth/crashpad-rs/compare/v0.2.0...v0.2.1) (2025-08-13)


### Bug Fixes

* fix version bump ([abfed4a](https://github.com/bahamoth/crashpad-rs/commit/abfed4a87f4a6037f69f98d9c701f1523416e158))

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
