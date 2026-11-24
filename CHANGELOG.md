# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
