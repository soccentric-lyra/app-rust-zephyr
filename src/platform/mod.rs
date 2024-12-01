//! Rust side of the C platform layer.
//!
//! `plat` is the FFI surface of `platform/platform.c`.  `board`, `boot` and
//! `build_info` hold the board constants, bootloader contract and build
//! metadata.

pub mod board;
pub mod boot;
pub mod build_info;
pub mod plat;
