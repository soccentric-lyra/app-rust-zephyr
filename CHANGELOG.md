# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Host test crate for `src/protocol/hwlink/frame.rs`: COBS examples, block
  boundaries, CRC corruption, reserved flags and size limits
- `make test-host`, `make boot-contract` and `make image-check` quality gates
- `docs/architecture.md`: Rust/C layers, thread model, devicetree overlay,
  boot hand-over and the remaining C responsibilities
- `cargo audit` and `cargo deny` jobs plus a `deny.toml` policy
- GitHub Actions workflow with build, lint, host tests and image checks

### Changed

- The pinned Zephyr and SDK versions stay in `docker/Dockerfile.zephyr`,
  while the Rust module revision stays in the child `docker/Dockerfile`;
  the split is now documented so the pin is easy to find
- `make lint` is now documented as the single lint entry point;
  `scripts/lint.sh` is its implementation
- The `dt-rust.yaml` TODO is replaced with an explanation of the pinned
  module's limitation instead of a loose end

### Fixed

- The README described `app-rust-freertos` as "Rust on Embassy"; it is Rust
  on FreeRTOS

## [1.0.0] - 2026-11-19

### Added

- Demo application in Rust on Zephyr RTOS, using Zephyr's official Rust
  module: LED patterns, the user button, HWLINK over UART, watchdog supervision
  and self-confirmation
- Thin C layer covers the drivers the Rust bindings do not reach yet
- Completes the C/Rust by FreeRTOS/Zephyr comparison set
- Runs from either A/B slot under the bootloader

[Unreleased]: https://github.com/soccentric-stm32/app-rust-zephyr/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/soccentric-stm32/app-rust-zephyr/releases/tag/v1.0.0
