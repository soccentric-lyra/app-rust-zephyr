//! Safe wrappers over the C platform layer (`platform/platform.c`).
//!
//! Author: Sandesh Ghimire <sandesh@soccentric.com>
//! SRS:    RZCON-004
//!
//! Declarations must match `platform/platform.h`.  The C functions take fixed
//! arguments or a buffer with its length, check their own bounds and may be
//! called from any thread once the kernel runs.

use core::ffi::{c_char, CStr};

use zephyr::time::{Duration, Tick};

/// Same as `PLAT_FAULT_TASK_LEN`.
pub const FAULT_TASK_LEN: usize = 16;
/// Same as `PLAT_FAULT_FILE_LEN`.
pub const FAULT_FILE_LEN: usize = 32;

/// Copy of the no-init fault record.
///
/// Layout must match `struct plat_fault` in `platform.h`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Fault {
    /// Reason code, index into `app::fault::REASONS`.
    pub reason: u32,
    /// SCB->CFSR at the fault.
    pub cfsr: u32,
    /// SCB->HFSR at the fault.
    pub hfsr: u32,
    /// SCB->MMFAR at the fault.
    pub mmfar: u32,
    /// SCB->BFAR at the fault.
    pub bfar: u32,
    /// Stacked PC, 0 without an exception frame.
    pub pc: u32,
    /// Stacked LR, 0 without an exception frame.
    pub lr: u32,
    /// Stacked xPSR, 0 without an exception frame.
    pub psr: u32,
    /// Source line; always 0 on Zephyr.
    pub line: u32,
    /// NUL-terminated thread name.
    pub task: [u8; FAULT_TASK_LEN],
    /// NUL-terminated source name.
    pub file: [u8; FAULT_FILE_LEN],
}

impl Fault {
    /// All-zero record, the out-buffer for `plat_fault_get`.
    const fn empty() -> Self {
        Self {
            reason: 0,
            cfsr: 0,
            hfsr: 0,
            mmfar: 0,
            bfar: 0,
            pc: 0,
            lr: 0,
            psr: 0,
            line: 0,
            task: [0; FAULT_TASK_LEN],
            file: [0; FAULT_FILE_LEN],
        }
    }
}

/// Raw C declarations; see `platform.h` for each function.
mod ffi {
    use core::ffi::c_char;

    use super::Fault;

    extern "C" {
        pub fn plat_fault_boot_start();
        pub fn plat_fault_get(out: *mut Fault) -> bool;
        pub fn plat_fault_is_current_boot() -> bool;
        pub fn plat_fault_is_previous_boot() -> bool;
        pub fn plat_fault_clear();
        pub fn plat_fatal() -> !;

        pub fn plat_vtor() -> u32;
        pub fn plat_reboot() -> !;
        pub fn plat_log(level: u8, text: *const u8, len: u32);
        pub fn plat_dcache_invalidate(addr: usize, len: u32);
        pub fn plat_thread_name(name: *const c_char);
        pub fn plat_stack_unused() -> u32;

        pub fn plat_wdt_init() -> bool;
        pub fn plat_wdt_feed();

        pub fn plat_gpio_init() -> bool;
        pub fn plat_led_set(led: u32, on: bool);
        pub fn plat_button_pressed() -> bool;
        pub fn plat_button_wait(timeout_ms: u32);

        pub fn plat_uart_init() -> bool;
        pub fn plat_uart_set_baud(baud: u32) -> bool;
        pub fn plat_uart_read(dst: *mut u8, max_len: u32) -> u32;
        pub fn plat_uart_wait(timeout_ms: u32);
        pub fn plat_uart_rx_running() -> bool;
        pub fn plat_uart_send(data: *const u8, len: u32, timeout_ms: u32) -> bool;

        pub fn plat_confirmed_read(out32: *mut u8) -> bool;
        pub fn plat_confirmed_write(record32: *const u8) -> bool;
    }
}

/// Advance the no-init boot counter; call once, first, at start-up.
pub fn fault_boot_start() {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_fault_boot_start() }
}

/// Saved fault record, if valid.
pub fn fault_get() -> Option<Fault> {
    let mut fault = Fault::empty();
    // SAFETY: `fault` is a valid, writable `Fault` matching `struct plat_fault`.
    if unsafe { ffi::plat_fault_get(&mut fault) } {
        Some(fault)
    } else {
        None
    }
}

/// Whether the saved fault belongs to this boot.
pub fn fault_is_current_boot() -> bool {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_fault_is_current_boot() }
}

/// Whether the saved fault belongs to the previous boot.
pub fn fault_is_previous_boot() -> bool {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_fault_is_previous_boot() }
}

/// Invalidate the saved fault record.
pub fn fault_clear() {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_fault_clear() }
}

/// Raise `k_panic()`: save a fault record, flash LD3 and reset.
pub fn fatal() -> ! {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_fatal() }
}

/// Active vector table address (SCB->VTOR).
pub fn vtor() -> u32 {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_vtor() }
}

/// Flush the log and cold-reset.
pub fn reboot() -> ! {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_reboot() }
}

/// Queue one debug line for USART6 (RZDBG-001); text past 120 bytes is cut.
///
/// Use the `log_*!` macros in `src/log.rs` instead of calling this directly.
pub fn log(level: u8, text: &[u8]) {
    // SAFETY: pointer and length describe `text`; C copies at most `len` bytes.
    unsafe { ffi::plat_log(level, text.as_ptr(), text.len() as u32) }
}

/// Invalidate the D-cache for `len` bytes at `addr`.
pub fn dcache_invalidate(addr: u32, len: u32) {
    // SAFETY: callers pass read-only flash ranges, so no dirty line is lost.
    unsafe { ffi::plat_dcache_invalidate(addr as usize, len) }
}

/// Name the calling thread.
pub fn thread_name(name: &'static CStr) {
    // SAFETY: `name` is NUL-terminated and `'static`, as Zephyr keeps the pointer.
    unsafe { ffi::plat_thread_name(name.as_ptr() as *const c_char) }
}

/// Unused bytes on the calling thread's stack (RZRT-005).
pub fn stack_unused() -> u32 {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_stack_unused() }
}

/// Take over the IWDG with a 4 s timeout; `false` if the driver is not ready.
pub fn wdt_init() -> bool {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_wdt_init() }
}

/// Feed the IWDG through the driver.
pub fn wdt_feed() {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_wdt_feed() }
}

/// Configure LD1..LD3, B1 and the B1 edge interrupt.
pub fn gpio_init() -> bool {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_gpio_init() }
}

/// Drive one LED: `led` 0 = LD1, 1 = LD2, 2 = LD3; others are ignored.
pub fn led_set(led: u32, on: bool) {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_led_set(led, on) }
}

/// Raw B1 level; `true` while pressed.
pub fn button_pressed() -> bool {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_button_pressed() }
}

/// Wait for a B1 edge or the timeout, whichever comes first.
pub fn button_wait(timeout_ms: u32) {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_button_wait(timeout_ms) }
}

/// Open USART3 at 115200 8N1 and start reception.
pub fn uart_init() -> bool {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_uart_init() }
}

/// Switch USART3 to 115200 or 921600; drops buffered RX bytes.
pub fn uart_set_baud(baud: u32) -> bool {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_uart_set_baud(baud) }
}

/// Copy buffered RX bytes into `dst`; returns the count, 0 if none.
pub fn uart_read(dst: &mut [u8]) -> usize {
    // SAFETY: pointer and length describe `dst`.
    unsafe { ffi::plat_uart_read(dst.as_mut_ptr(), dst.len() as u32) as usize }
}

/// Wait for received bytes or the timeout, whichever comes first.
pub fn uart_wait(timeout_ms: u32) {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_uart_wait(timeout_ms) }
}

/// Whether USART3 reception is enabled.
pub fn uart_rx_running() -> bool {
    // SAFETY: no arguments or plain values; callable from any thread.
    unsafe { ffi::plat_uart_rx_running() }
}

/// Send one frame and block until its last stop bit has left USART3.
///
/// Returns `false` if `data` exceeds 2200 bytes or the send times out.
pub fn uart_send(data: &[u8], timeout_ms: u32) -> bool {
    // SAFETY: pointer and length describe `data`; C copies it before returning.
    unsafe { ffi::plat_uart_send(data.as_ptr(), data.len() as u32, timeout_ms) }
}

/// Read the 32-byte CONFIRMED record of the running slot.
pub fn confirmed_read() -> Option<[u8; 32]> {
    let mut record = [0u8; 32];
    // SAFETY: `record` is 32 writable bytes, as the C side requires.
    if unsafe { ffi::plat_confirmed_read(record.as_mut_ptr()) } {
        Some(record)
    } else {
        None
    }
}

/// Program the 32-byte CONFIRMED record of the running slot.
pub fn confirmed_write(record: &[u8; 32]) -> bool {
    // SAFETY: `record` is 32 readable bytes, as the C side requires.
    unsafe { ffi::plat_confirmed_write(record.as_ptr()) }
}

/// Milliseconds since boot.
pub fn now_ms() -> u64 {
    zephyr::sys::uptime_get() as u64
}

/// Sleep the calling thread for `ms` milliseconds.
pub fn sleep_ms(ms: u32) {
    let _ = zephyr::time::sleep(Duration::millis(ms as Tick));
}
