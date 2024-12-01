//! Debug lines on USART6 (RZDBG-001).
//!
//! Author: Sandesh Ghimire <sandesh@soccentric.com>
//! SRS:    RZDBG-001..RZDBG-003
//!
//! Text goes through Zephyr's deferred logging (`platform/debug_log.c`) in the
//! line format BOOT and the other firmwares use:
//!
//! ```text
//! [   5012] R-ZEP I health: passed, CONFIRMED written
//! ```
//!
//! The macros format into a fixed stack buffer and never block.  Text past
//! [`TEXT_MAX`] bytes is cut off.

use core::fmt::{self, Write};

use crate::platform::plat;

/// Same limit as PLAT_LOG_TEXT_MAX in platform/platform.h.
const TEXT_MAX: usize = 120;

/// Fixed-size formatting buffer that silently truncates.
struct Line {
    /// Formatted bytes.
    buf: [u8; TEXT_MAX],
    /// Bytes used in `buf`.
    len: usize,
}

impl Write for Line {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let take = s.len().min(TEXT_MAX - self.len);
        self.buf[self.len..self.len + take].copy_from_slice(&s.as_bytes()[..take]);
        self.len += take;
        Ok(())
    }
}

/// Format `args` and queue it as one debug line.
///
/// `level` is `b'E'`, `b'W'`, `b'I'` or `b'D'`.
pub fn write(level: u8, args: fmt::Arguments) {
    let mut line = Line {
        buf: [0; TEXT_MAX],
        len: 0,
    };
    let _ = line.write_fmt(args);
    plat::log(level, &line.buf[..line.len]);
}

/// Log an error line (`E`), `format!` syntax.
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::log::write(b'E', format_args!($($arg)*)) };
}

/// Log a warning line (`W`), `format!` syntax.
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::log::write(b'W', format_args!($($arg)*)) };
}

/// Log an info line (`I`), `format!` syntax.
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::log::write(b'I', format_args!($($arg)*)) };
}

/// Rate-limited "task <n> <name>: <purpose>" line for one thread.
///
/// Each thread owns one and calls [`TaskReport::tick`] every loop iteration;
/// it prints at most once a second.
pub struct TaskReport {
    /// Task number shown in the line.
    number: u8,
    /// Thread name.
    name: &'static str,
    /// One-line description of the thread.
    purpose: &'static str,
    /// Uptime of the last print in ms; `None` before the first.
    last: Option<u64>,
}

impl TaskReport {
    /// Create a report that prints on its first `tick`.
    pub const fn new(number: u8, name: &'static str, purpose: &'static str) -> Self {
        Self {
            number,
            name,
            purpose,
            last: None,
        }
    }

    /// Print the line if at least 1000 ms have passed since the last one.
    pub fn tick(&mut self) {
        let now = plat::now_ms();
        let due = match self.last {
            Some(last) => now.wrapping_sub(last) >= 1000,
            None => true,
        };
        if due {
            self.last = Some(now);
            write(
                b'I',
                format_args!("task {} {}: {}", self.number, self.name, self.purpose),
            );
        }
    }
}
