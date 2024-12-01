//! Application logic and the four Zephyr threads.
//!
//! `health`, `comms`, `button` and `led` own the threads; `ipc` holds the
//! shared state and fixed-size queues; `fault` and `identity` are services.

pub mod button;
pub mod comms;
pub mod fault;
pub mod health;
pub mod identity;
pub mod ipc;
pub mod led;
