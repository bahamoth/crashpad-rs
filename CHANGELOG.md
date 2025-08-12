# Changelog

## [0.2.0](https://github.com/bahamoth/crashpad-rs/compare/v0.1.0...v0.2.0) (2025-08-12)


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
* **ci:** add Android emulator and iOS simulator tests ([90c9f24](https://github.com/bahamoth/crashpad-rs/commit/90c9f24845644bf9414c2daaf0860e7678362276))
* **ci:** add individual PR comments for each platform test ([34f4ce8](https://github.com/bahamoth/crashpad-rs/commit/34f4ce8b4672fe2eb2978ee02119c60a3d46e0c8))
* **ci:** add test result visualization ([d7fbe16](https://github.com/bahamoth/crashpad-rs/commit/d7fbe16a60f7dfa2e7841bd7755ebdac5c0df73b))
* **ci:** integrate test reporting across all platforms ([127f903](https://github.com/bahamoth/crashpad-rs/commit/127f9034353637b3938cc1bb65fc32392e5bdb4a))
* **ci:** unify test reporting across all platforms ([e82f4f0](https://github.com/bahamoth/crashpad-rs/commit/e82f4f0cebd433a2aaa6ea10fb0fc6b4b9dc902a))
* Implement working Crashpad Rust bindings ([cd94e9a](https://github.com/bahamoth/crashpad-rs/commit/cd94e9aa622bc6cd1ef4b36bdc3f746964820d9c))
* major refactoring and Android cross-compilation support ([6c6f6db](https://github.com/bahamoth/crashpad-rs/commit/6c6f6dbc7529119dff3eda09b361d8435831ca02))


### Bug Fixes

* **android:** enable crashpad_handler build for Android ([643e062](https://github.com/bahamoth/crashpad-rs/commit/643e06231755ce7fae013f01c4dfa10b277b74e6))
* **android:** improve libc++_shared.so deployment for emulator tests ([20aeccc](https://github.com/bahamoth/crashpad-rs/commit/20aecccb2c790b2db87ab01e854fb950fc2038fe))
* **ci:** add libc++_shared.so deployment for Android tests ([8fea9b9](https://github.com/bahamoth/crashpad-rs/commit/8fea9b9f4bcc42266fcba906157744d937a2c58e))
* **ci:** correctly calculate failures in JUnit XML generation ([c910f20](https://github.com/bahamoth/crashpad-rs/commit/c910f20f54b8f2f7234ad563adea2808321c64f2))
* **ci:** enable PR comments and fix iOS test result reporting ([46d17f1](https://github.com/bahamoth/crashpad-rs/commit/46d17f11c2604d6ac655234feaa1825de1649cc7))
* **ci:** prevent Android emulator hanging on termination ([4c72ff8](https://github.com/bahamoth/crashpad-rs/commit/4c72ff88bbdf64ca7a0fa3c7574adc3ef72a94a3))
* **ci:** remove incorrect fallback for Android handler ([5edaee2](https://github.com/bahamoth/crashpad-rs/commit/5edaee22ab0be7e3852b7060fc8a19312ce22f41))
* **ci:** resolve Android emulator test script execution issue ([f53df09](https://github.com/bahamoth/crashpad-rs/commit/f53df09e71b69780dcf332d63d40da10e31cb34e))
* **ci:** resolve shell script syntax errors in Android workflow ([8250db2](https://github.com/bahamoth/crashpad-rs/commit/8250db24c89a4b27df78f3856484f0b171fcbfa9))
* **ci:** stabilize Android emulator setup ([6202504](https://github.com/bahamoth/crashpad-rs/commit/6202504294a9fb9218e05b56697536300edda263))
* **ci:** update NDK version and improve iOS simulator setup ([98a2fe6](https://github.com/bahamoth/crashpad-rs/commit/98a2fe629b081347c1687f059d2a14a939ae2ea9))
* **ci:** use ARM64 for Android emulator tests ([ec53c10](https://github.com/bahamoth/crashpad-rs/commit/ec53c105dfa66eb5f22b1ca36139f6c21cc921ee))
* **ci:** use dorny/test-reporter for macOS/iOS compatibility ([817aa1f](https://github.com/bahamoth/crashpad-rs/commit/817aa1feebbce117d72a9cc9cd54be295e04c9d6))
* **ci:** use Intel-based macOS runner for Android emulator ([5544181](https://github.com/bahamoth/crashpad-rs/commit/554418130229ea01006ebf5866faad121151a89e))
* **ci:** use mikepenz/action-junit-report for better compatibility ([c3b143d](https://github.com/bahamoth/crashpad-rs/commit/c3b143de526ba8a34530ad4a1747595482d758b0))
* conditionally link mig_output only on macOS/iOS platforms ([9002eba](https://github.com/bahamoth/crashpad-rs/commit/9002ebaefbd585edad3c18774358ce3760600156))
* configure release-please to handle workspace version inheritance ([8f2c703](https://github.com/bahamoth/crashpad-rs/commit/8f2c70353316020869860c7af2185e848920fb1c))
* correct remaining crashpad_sys references ([5c76cc3](https://github.com/bahamoth/crashpad-rs/commit/5c76cc32138efb142cc40d99e6cafb9f7aa7a49b))
* **ios:** add TAP output mode to ios_simulator_test ([0f17056](https://github.com/bahamoth/crashpad-rs/commit/0f17056ddcb0a2a17ed5752f15671737d37dabaa))
* resolve all clippy warnings and format code ([ce81deb](https://github.com/bahamoth/crashpad-rs/commit/ce81deb910c23e70c1d339090cb8fc96b531ff80))
* resolve iOS build errors with conditional import ([43a3b8c](https://github.com/bahamoth/crashpad-rs/commit/43a3b8c5b7b5def1dd4ec16f8381eab65a23f527))
* resolve Linux asynchronous_start test failures ([f1e3007](https://github.com/bahamoth/crashpad-rs/commit/f1e30074e6b7805228d9b33bc4cb3f0124883157))
* restructure integration tests to handle global state ([9f979c7](https://github.com/bahamoth/crashpad-rs/commit/9f979c784f73bff333da9f20e55e2abe7ce9670c))
* support docs.rs build and fix package metadata ([d5a5290](https://github.com/bahamoth/crashpad-rs/commit/d5a52902c6d60a7dd2f15802b8bf7af9214e2a4f))
* update all references from crashpad-sys to crashpad-rs-sys ([115a1c0](https://github.com/bahamoth/crashpad-rs/commit/115a1c02a246a7ad31499d414a0b992d0dee627a))
* update library filename in xtask dist command ([7c66c3e](https://github.com/bahamoth/crashpad-rs/commit/7c66c3e40a196026a38284bf7c72b7b42da65868))
* use xtask symlink to avoid cargo package errors ([0a76e6f](https://github.com/bahamoth/crashpad-rs/commit/0a76e6f09bdea49c96a9ad7b4aa3057b1b95d3fd))


### Code Refactoring

* reorganize project structure for crates.io publishing ([3fa4b8c](https://github.com/bahamoth/crashpad-rs/commit/3fa4b8c8a30301c1bd4825ae1de47d4ac70a5651))

## Changelog

All notable changes to this project will be documented in this file.
