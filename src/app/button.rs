//! Button thread: B1 (PC13) debounce, short and long press.
//!
//! Author: Sandesh Ghimire <sandesh@soccentric.com>
//! SRS:    RZFN-002; BTN-001..BTN-004
//!
//! The thread only debounces B1 and sends one command per press.  The LED
//! thread owns the pattern state (as in the C demo), so a second press that
//! arrives before the first is handled never loses a step.  The GPIO interrupt
//! wakes the thread on an edge; the 10 ms timeout keeps debounce and check-ins
//! running while B1 is held.

use crate::app::ipc::{self, LedCommand};
use crate::platform::plat;

/// Time the level must be stable to count, in ms.
const DEBOUNCE_MS: u64 = 30;
/// Longest press that counts as short, in ms.
const SHORT_MAX_MS: u64 = 1000;
/// Hold time that triggers a long press, in ms (BTN-003).
const LONG_MIN_MS: u64 = 2000;
/// Wake-up timeout while waiting for an edge, in ms.
const POLL_MS: u32 = 10;
/// Check-in period towards Health, in ms.
const CHECKIN_PERIOD_MS: u64 = 500;

/// Button thread body; never returns.
///
/// Short press: `LedCommand::ButtonShort` on release.  Long press:
/// `LedCommand::ButtonLong` once B1 has been held for 2 s.
#[zephyr::thread(stack_size = crate::BUTTON_STACK)]
pub fn thread() {
    plat::thread_name(c"Button");

    // B1 is high when pressed and has an external pull-down on the board.
    let mut pressed = plat::button_pressed();
    let mut candidate = pressed;
    let mut candidate_since = plat::now_ms();
    let mut press_start = candidate_since;
    let mut long_sent = false;
    let mut last_checkin = candidate_since;

    let mut report = crate::log::TaskReport::new(
        2,
        "Button",
        "debounces B1, short press cycles the LED pattern, long press resets it",
    );

    loop {
        report.tick();
        plat::button_wait(POLL_MS);

        let now = plat::now_ms();
        let raw = plat::button_pressed();
        if raw != candidate {
            candidate = raw;
            candidate_since = now;
        } else if candidate != pressed && now - candidate_since >= DEBOUNCE_MS {
            pressed = candidate;
            if pressed {
                press_start = now;
                long_sent = false;
            } else if !long_sent && now - press_start < SHORT_MAX_MS {
                log_info!("button: short press, next LED pattern");
                let _ = ipc::LED_COMMANDS.try_send(LedCommand::ButtonShort);
            }
        }

        // BTN-003: act at 2 s, not on release, as the C demo does.
        if pressed && !long_sent && now - press_start >= LONG_MIN_MS {
            long_sent = true;
            log_info!("button: long press, default LED patterns");
            let _ = ipc::LED_COMMANDS.try_send(LedCommand::ButtonLong);
        }

        if now - last_checkin >= CHECKIN_PERIOD_MS {
            ipc::check_in(ipc::TASK_BUTTON);
            last_checkin = now;
        }
    }
}
