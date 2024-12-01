//! Shared state and queues between the four Zephyr threads.
//!
//! Author: Sandesh Ghimire <sandesh@soccentric.com>
//! SRS:    RZRT-002..005
//!
//! Everything is static and fixed-size; there is no allocator.  Critical
//! sections come from the zephyr crate (`irq_lock()`) and are held only to
//! copy a few bytes.

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering};

use critical_section::Mutex;

/// Check-in index of the LED thread.
pub const TASK_LED: usize = 0;
/// Check-in index of the Button thread.
pub const TASK_BUTTON: usize = 1;
/// Check-in index of the Comms thread.
pub const TASK_COMMS: usize = 2;
/// Check-in index of the Health thread.
pub const TASK_HEALTH: usize = 3;
/// Number of supervised threads.
pub const TASK_COUNT: usize = 4;

/// Application state reported by `Status`.
#[derive(Clone, Copy)]
pub struct StatusSnapshot {
    /// LD1 `hwlink::pattern` code.
    pub led1_pattern: u8,
    /// LD1 period in ms.
    pub led1_period: u16,
    /// LD2 `hwlink::pattern` code.
    pub led2_pattern: u8,
    /// LD2 period in ms.
    pub led2_period: u16,
    /// LD3 `hwlink::pattern` code.
    pub led3_pattern: u8,
    /// LD3 period in ms.
    pub led3_period: u16,
    /// `board::HEALTH_*`.
    pub health_state: u8,
    /// LD3 latched for a fault or watchdog reset (LED-005).
    pub fault_active: bool,
    /// Health check failed.
    pub health_failed: bool,
    /// Total check-ins counted by Health.
    pub task_checks: u32,
}

impl StatusSnapshot {
    /// Power-on state: LD1 heartbeat, LD2 and LD3 off, 1000 ms periods.
    pub const fn initial() -> Self {
        Self {
            led1_pattern: crate::protocol::hwlink::pattern::HEARTBEAT,
            led1_period: 1000,
            led2_pattern: crate::protocol::hwlink::pattern::OFF,
            led2_period: 1000,
            led3_pattern: crate::protocol::hwlink::pattern::OFF,
            led3_period: 1000,
            health_state: crate::platform::board::HEALTH_NOT_REQUIRED,
            fault_active: false,
            health_failed: false,
            task_checks: 0,
        }
    }
}

/// The shared status, copied in and out under a critical section.
static STATUS: Mutex<Cell<StatusSnapshot>> = Mutex::new(Cell::new(StatusSnapshot::initial()));

/// Copy of the current status.
pub fn status() -> StatusSnapshot {
    critical_section::with(|cs| STATUS.borrow(cs).get())
}

/// Modify the status atomically.
///
/// `f` runs inside a critical section; keep it short and non-blocking.
pub fn update_status(f: impl FnOnce(&mut StatusSnapshot)) {
    critical_section::with(|cs| {
        let cell = STATUS.borrow(cs);
        let mut snapshot = cell.get();
        f(&mut snapshot);
        cell.set(snapshot);
    });
}

/// Fixed-capacity, non-blocking FIFO of `N` `Copy` messages.
pub struct Queue<T: Copy, const N: usize> {
    /// Ring buffer behind a critical-section mutex.
    inner: Mutex<RefCell<Ring<T, N>>>,
}

/// Ring buffer storage of a [`Queue`].
struct Ring<T: Copy, const N: usize> {
    /// Slots; `None` when free.
    items: [Option<T>; N],
    /// Index of the oldest item.
    head: usize,
    /// Number of queued items.
    len: usize,
}

impl<T: Copy, const N: usize> Queue<T, N> {
    /// Create an empty queue.
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(Ring {
                items: [None; N],
                head: 0,
                len: 0,
            })),
        }
    }

    /// Append `item`; returns it back in `Err` when the queue is full.
    pub fn try_send(&self, item: T) -> Result<(), T> {
        critical_section::with(|cs| {
            let mut ring = self.inner.borrow_ref_mut(cs);
            if ring.len == N {
                return Err(item);
            }
            let tail = (ring.head + ring.len) % N;
            ring.items[tail] = Some(item);
            ring.len += 1;
            Ok(())
        })
    }

    /// Remove the oldest item; `None` when empty.
    pub fn try_receive(&self) -> Option<T> {
        critical_section::with(|cs| {
            let mut ring = self.inner.borrow_ref_mut(cs);
            if ring.len == 0 {
                return None;
            }
            let head = ring.head;
            let item = ring.items[head].take();
            ring.head = (head + 1) % N;
            ring.len -= 1;
            item
        })
    }
}

/// Command for the LED thread.
#[derive(Clone, Copy)]
pub enum LedCommand {
    /// Set one LED (`SetLed`).
    Set {
        /// `hwlink::led` number, 1..3.
        led: u8,
        /// `hwlink::pattern` code.
        pattern: u8,
        /// Period in ms.
        period_ms: u16,
        /// Allows LD3; ignored for LD1/LD2.
        force: bool,
    },
    /// Set LD1 and LD2 (`SetLedAll`).
    SetAll {
        /// `hwlink::pattern` code.
        pattern: u8,
        /// Period in ms.
        period_ms: u16,
    },
    /// Short B1 press: next pattern (BTN-002).
    ButtonShort,
    /// Long B1 press: default patterns (BTN-003).
    ButtonLong,
    /// Release the LD3 fault latch.
    ClearFault,
    /// Health check failed: fast-blink LD3.
    HealthFailed,
}

/// Commands from Comms, Button and Health to the LED thread.
pub static LED_COMMANDS: Queue<LedCommand, 8> = Queue::new();

/// `ButtonEvent::press` for a short press.
pub const BUTTON_SHORT: u8 = 1;
/// `ButtonEvent::press` for a long press.
pub const BUTTON_LONG: u8 = 2;

/// Button press reported to the host, with the resulting LD1 pattern.
#[derive(Clone, Copy)]
pub struct ButtonEvent {
    /// [`BUTTON_SHORT`] or [`BUTTON_LONG`].
    pub press: u8,
    /// New `hwlink::pattern` code.
    pub pattern: u8,
    /// New period in ms.
    pub period_ms: u16,
}

/// Button events from the LED thread to Comms.
pub static BUTTON_EVENTS: Queue<ButtonEvent, 4> = Queue::new();

/// Check-ins per thread since Health last read them.
///
/// One counter per thread, so several check-ins between two Health polls all
/// count (the C demo's `k_event` would merge them).
static CHECKINS: [AtomicU32; TASK_COUNT] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

/// Smallest unused stack seen per thread, in bytes; `u32::MAX` before the
/// first sample (RZRT-005).
static STACK_UNUSED: [AtomicU32; TASK_COUNT] = [
    AtomicU32::new(u32::MAX),
    AtomicU32::new(u32::MAX),
    AtomicU32::new(u32::MAX),
    AtomicU32::new(u32::MAX),
];

/// Record a check-in for `task` (RZRT-004); call at least every 500 ms.
///
/// Also samples the caller's unused stack for `Status`.  The stack scan is
/// why fast loops check in on their own slower cadence.
///
/// # Panics
///
/// Panics if `task >= TASK_COUNT`.
pub fn check_in(task: usize) {
    CHECKINS[task].fetch_add(1, Ordering::Relaxed);
    STACK_UNUSED[task].fetch_min(crate::platform::plat::stack_unused(), Ordering::Relaxed);
}

/// Return and reset the check-ins of `task` since the last call.
///
/// # Panics
///
/// Panics if `task >= TASK_COUNT`.
pub fn take_check_in(task: usize) -> u32 {
    CHECKINS[task].swap(0, Ordering::Relaxed)
}

/// Smallest unused stack seen for `task` in bytes; 0 before the first sample.
///
/// # Panics
///
/// Panics if `task >= TASK_COUNT`.
pub fn stack_unused(task: usize) -> u32 {
    match STACK_UNUSED[task].load(Ordering::Relaxed) {
        u32::MAX => 0,
        unused => unused,
    }
}
