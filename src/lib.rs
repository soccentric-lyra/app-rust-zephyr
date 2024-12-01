//! Rust demo application on Zephyr: entry point.
//!
//! Author:  Sandesh Ghimire <sandesh@soccentric.com>
//! Date:    2026-11-04
//! Target:  NUCLEO-F767ZI (Zephyr board nucleo_f767zi), started by BOOT
//! SRS:     SRS-RZEPHYR-F767, SRS-DEMO-F767
//!
//! Zephyr's `main()` (in the Rust module) calls [`rust_main`], which runs the
//! start-up checks, starts the Health, Comms, Button and LED threads and
//! returns.  The only flash write is the CONFIRMED record in `boot.rs`.

#![no_std]
#![deny(warnings)]

#[macro_use]
mod log;

mod app;
mod platform;
mod protocol;

// RZRT-001: the C Zephyr demo's priorities and stack sizes, with more stack
// for Rust's formatting.

/// Health thread priority (highest, RZRT-001).
pub(crate) const PRIO_HEALTH: i32 = 2;
/// Comms thread priority.
pub(crate) const PRIO_COMMS: i32 = 3;
/// Button thread priority.
pub(crate) const PRIO_BUTTON: i32 = 4;
/// LED thread priority (lowest).
pub(crate) const PRIO_LED: i32 = 5;

/// Health thread stack in bytes.
pub(crate) const HEALTH_STACK: usize = 2048;
/// Comms thread stack in bytes.
pub(crate) const COMMS_STACK: usize = 4096;
/// Button thread stack in bytes.
pub(crate) const BUTTON_STACK: usize = 1536;
/// LED thread stack in bytes.
pub(crate) const LED_STACK: usize = 1536;

/// Run the start-up checks and start the four application threads.
///
/// Called once by Zephyr's `main()`.  A wrong vector table or failed GPIO or
/// UART set-up ends in [`app::fault::fatal`].
#[no_mangle]
extern "C" fn rust_main() {
    // The fault record is in no-init RAM; bump its boot id first.
    platform::plat::fault_boot_start();

    // RZBT-002: feed the watchdog BOOT started before any long set-up.
    platform::plat::wdt_feed();

    // RZGEN-005: keep the reset cause for Status, then clear it.
    let reset_cause = platform::boot::take_reset_cause();

    // RZBT-004: decode the hand-over record before Health starts.
    let handover = platform::boot::read_handover();

    // RZDBG-001: start-up line on the debug UART.
    log_info!(
        "start v{} {}, slot {} {}, reset {}",
        platform::build_info::VERSION,
        platform::build_info::GIT_COMMIT,
        match handover.slot {
            platform::board::SLOT_A => 'A',
            platform::board::SLOT_B => 'B',
            _ => '?',
        },
        match handover.trial_state {
            platform::board::TRIAL_PENDING => "on trial",
            platform::board::TRIAL_CONFIRMED => "confirmed",
            _ => "no hand-over",
        },
        platform::boot::reset_cause_text(reset_cause)
    );

    // RZBT-001/002: the vector table must be the image's own table.
    if platform::plat::vtor() != platform::board::SLOT_BASE {
        app::fault::fatal();
    }

    // The IWDG keeps running from BOOT even if the driver cannot take it
    // over; Health then keeps feeding it through the register path.
    let _ = platform::plat::wdt_init();

    // LED-005: latch LD3 for the last fault or watchdog reset.
    let initial_fault =
        app::fault::last().is_some() || platform::boot::reset_was_watchdog(reset_cause);
    if app::fault::last().is_some() {
        log_warn!("last fault recorded before this reset");
    }
    app::ipc::update_status(|s| s.fault_active = initial_fault);

    // COM-001/002: LEDs, B1 and USART3 must work before the health check.
    if !platform::plat::gpio_init() || !platform::plat::uart_init() {
        app::fault::fatal();
    }

    let info = app::comms::BootInfo {
        reset_cause,
        handover,
    };

    let thread = app::health::thread(handover);
    thread.set_priority(PRIO_HEALTH);
    thread.start();

    let thread = app::comms::thread(info);
    thread.set_priority(PRIO_COMMS);
    thread.start();

    let thread = app::button::thread();
    thread.set_priority(PRIO_BUTTON);
    thread.start();

    let thread = app::led::thread(initial_fault);
    thread.set_priority(PRIO_LED);
    thread.start();
}
