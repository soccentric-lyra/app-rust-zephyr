//! HWLINK demo message set.
//!
//! Author: Sandesh Ghimire <sandesh@soccentric.com>
//! SRS:    RFN-003, RFN-005, RLNK-001..004; LINK A.3
//!
//! The schema is the shared `protocol/hwlink.fbs`.  See `fb.rs` for why the
//! FlatBuffers codec is local and fixed-buffer (OPEN-R01).  The constant
//! values below are the wire format; do not renumber.
#![allow(dead_code)]

pub mod fb;
pub mod frame;

use fb::{Builder, Table, Value};

/// Protocol minor version reported in HelloReply and Status.
pub const PROTOCOL_MINOR: u16 = 1;
/// Largest raw frame, reported in HelloReply.
pub const MAX_FRAME: usize = frame::MAX_DECODED;

/// `Message.body` union tags; the order is the wire format (LINK-013).
pub mod body {
    /// No body.
    pub const NONE: u8 = 0;
    /// Hello body.
    pub const HELLO: u8 = 1;
    /// HelloReply body.
    pub const HELLO_REPLY: u8 = 2;
    /// Ping body.
    pub const PING: u8 = 3;
    /// Pong body.
    pub const PONG: u8 = 4;
    /// SetBaud body.
    pub const SET_BAUD: u8 = 5;
    /// Ack body.
    pub const ACK: u8 = 6;
    /// Nak body.
    pub const NAK: u8 = 7;
    /// Log body.
    pub const LOG: u8 = 8;
    /// Progress body.
    pub const PROGRESS: u8 = 9;
    /// Reset body.
    pub const RESET: u8 = 10;
    /// GetBootInfo body.
    pub const GET_BOOT_INFO: u8 = 11;
    /// BootInfo body.
    pub const BOOT_INFO: u8 = 12;
    /// UpdateBegin body.
    pub const UPDATE_BEGIN: u8 = 13;
    /// UpdateData body.
    pub const UPDATE_DATA: u8 = 14;
    /// UpdateEnd body.
    pub const UPDATE_END: u8 = 15;
    /// UpdateResult body.
    pub const UPDATE_RESULT: u8 = 16;
    /// Activate body.
    pub const ACTIVATE: u8 = 17;
    /// UpdateAbort body.
    pub const UPDATE_ABORT: u8 = 18;
    /// SetLed body.
    pub const SET_LED: u8 = 19;
    /// SetLedAll body.
    pub const SET_LED_ALL: u8 = 20;
    /// ClearFault body.
    pub const CLEAR_FAULT: u8 = 21;
    /// EnterBootloader body.
    pub const ENTER_BOOTLOADER: u8 = 22;
    /// GetStatus body.
    pub const GET_STATUS: u8 = 23;
    /// Status body.
    pub const STATUS: u8 = 24;
    /// ButtonEvent body.
    pub const BUTTON_EVENT: u8 = 25;
}

/// HelloReply `role` values.
pub mod role {
    /// Hardware validation firmware.
    pub const HWVAL: u8 = 1;
    /// BOOT.
    pub const BOOTLOADER: u8 = 2;
    /// Demo application.
    pub const DEMO: u8 = 3;
}

/// HelloReply `flash_mode` values.
pub mod flash_mode {
    /// Not known.
    pub const UNKNOWN: u8 = 0;
    /// Single-bank flash.
    pub const SINGLE_BANK: u8 = 1;
    /// Dual-bank flash.
    pub const DUAL_BANK: u8 = 2;
}

/// Status `slot` values.
pub mod slot {
    /// Not known.
    pub const UNKNOWN: u8 = 0;
    /// Slot A.
    pub const A: u8 = 1;
    /// Slot B.
    pub const B: u8 = 2;
}

/// Status `trial` values.
pub mod trial {
    /// Not known.
    pub const UNKNOWN: u8 = 0;
    /// Confirmed.
    pub const CONFIRMED: u8 = 1;
    /// On trial.
    pub const PENDING: u8 = 2;
}

/// Status `health` values.
pub mod health {
    /// Not on trial.
    pub const NOT_REQUIRED: u8 = 0;
    /// In progress.
    pub const RUNNING: u8 = 1;
    /// Passed.
    pub const PASSED: u8 = 2;
    /// Failed.
    pub const FAILED: u8 = 3;
}

/// ButtonEvent `press` values.
pub mod button {
    /// Short press.
    pub const SHORT: u8 = 1;
    /// Long press.
    pub const LONG: u8 = 2;
}

/// LED pattern codes (Appendix A).
pub mod pattern {
    /// Off.
    pub const OFF: u8 = 0;
    /// On.
    pub const ON: u8 = 1;
    /// 50 % duty at the period.
    pub const BLINK: u8 = 2;
    /// Two 50 ms pulses per period (at least 250 ms).
    pub const HEARTBEAT: u8 = 3;
    /// Blink at a fifth of the period.
    pub const FAST: u8 = 4;
    /// LD1 and LD2 alternate.
    pub const CHASE: u8 = 5;
    /// SOS with a 100 ms unit.
    pub const MORSE_SOS: u8 = 6;

    /// Number of pattern codes.
    pub const COUNT: u8 = 7;

    /// Whether `v` is a known pattern code.
    pub fn valid(v: u8) -> bool {
        v < COUNT
    }
}

/// LED numbers on the wire (1-based).
pub mod led {
    /// LD1 (green).
    pub const LD1: u8 = 1;
    /// LD2 (blue).
    pub const LD2: u8 = 2;
    /// LD3 (red), reserved for faults.
    pub const LD3: u8 = 3;
}

/// Nak error codes.
pub mod error {
    /// No error.
    pub const NONE: u16 = 0;
    /// Malformed message.
    pub const MALFORMED: u16 = 1;
    /// Unsupported message.
    pub const UNSUPPORTED: u16 = 2;
    /// Argument out of range.
    pub const BAD_ARGUMENT: u16 = 3;
    /// Not allowed in this state.
    pub const BAD_STATE: u16 = 4;
    /// Resource busy.
    pub const BUSY: u16 = 5;
    /// Not found.
    pub const NOT_FOUND: u16 = 6;
}

/// Request payload rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Not a valid HWLINK FlatBuffer.
    Malformed,
    /// Valid but not handled; unused, [`parse_request`] returns `Request::Unknown`.
    Unsupported,
}

/// Decoded host request.
pub enum Request<'a> {
    /// Open a session.
    Hello {
        /// Host tool name.
        host_name: Option<&'a str>,
        /// Host tool version.
        host_version: Option<&'a str>,
        /// Host protocol minor version; 0 if absent.
        protocol_minor: u16,
    },
    /// Liveness check.
    Ping,
    /// Request a Status reply.
    GetStatus,
    /// Set one LED.
    SetLed {
        /// [`led`] number; defaults to LD1.
        led: u8,
        /// [`pattern`] code; defaults to OFF.
        pattern: u8,
        /// Period in ms; defaults to 0 (rejected).
        period_ms: u16,
        /// Required to drive LD3.
        force: bool,
    },
    /// Set LD1 and LD2.
    SetLedAll {
        /// [`pattern`] code; defaults to OFF.
        pattern: u8,
        /// Period in ms; defaults to 0 (rejected).
        period_ms: u16,
    },
    /// Clear the fault record and the LD3 latch.
    ClearFault,
    /// Reset into BOOT.
    EnterBootloader,
    /// Reset the MCU.
    Reset,
    /// Change the USART3 rate.
    SetBaud {
        /// Baud rate; 0 if absent.
        rate: u32,
    },
    /// Any other body type.
    Unknown,
}

/// Required `u8` field `id`; `Malformed` if absent.
fn required_u8(t: Table<'_>, id: u8) -> Result<u8, ParseError> {
    t.u8(id).ok_or(ParseError::Malformed)
}

/// Decode a request payload.
///
/// # Errors
///
/// [`ParseError::Malformed`] for a bad root, identifier, body type or missing
/// body table.
pub fn parse_request(payload: &[u8]) -> Result<Request<'_>, ParseError> {
    let msg = Table::root(payload, fb::FILE_ID).ok_or(ParseError::Malformed)?;
    let body_type = required_u8(msg, 0)?;
    let body = if body_type == body::NONE {
        None
    } else {
        Some(msg.table(1).ok_or(ParseError::Malformed)?)
    };

    Ok(match body_type {
        body::HELLO => {
            let t = body.ok_or(ParseError::Malformed)?;
            Request::Hello {
                host_name: t.str(0),
                host_version: t.str(1),
                protocol_minor: t.u16(2).unwrap_or(0),
            }
        }
        body::PING => Request::Ping,
        body::GET_STATUS => Request::GetStatus,
        body::SET_LED => {
            let t = body.ok_or(ParseError::Malformed)?;
            // FlatBuffers omits scalars equal to their default.  Return
            // LED1/OFF/0 like the generated C reader and let the argument
            // checks answer BAD_ARGUMENT.
            Request::SetLed {
                led: t.u8(0).unwrap_or(led::LD1),
                pattern: t.u8(1).unwrap_or(pattern::OFF),
                period_ms: t.u16(2).unwrap_or(0),
                force: t.u8(3).unwrap_or(0) != 0,
            }
        }
        body::SET_LED_ALL => {
            let t = body.ok_or(ParseError::Malformed)?;
            Request::SetLedAll {
                pattern: t.u8(0).unwrap_or(pattern::OFF),
                period_ms: t.u16(1).unwrap_or(0),
            }
        }
        body::CLEAR_FAULT => Request::ClearFault,
        body::ENTER_BOOTLOADER => Request::EnterBootloader,
        body::RESET => Request::Reset,
        body::SET_BAUD => Request::SetBaud {
            rate: body.ok_or(ParseError::Malformed)?.u32(0).unwrap_or(0),
        },
        _ => Request::Unknown,
    })
}

/// HelloReply contents.
pub struct HelloReply<'a> {
    /// Firmware name.
    pub firmware_name: &'a str,
    /// Version text.
    pub version: &'a str,
    /// Git commit.
    pub commit: &'a str,
    /// Build time, ISO 8601 UTC.
    pub build_time: &'a str,
    /// 64-bit device UID.
    pub uid: u64,
    /// DBGMCU device ID.
    pub dev_id: u32,
    /// DBGMCU revision ID.
    pub rev_id: u32,
    /// [`flash_mode`] value.
    pub flash_mode: u8,
    /// Identity detail string.
    pub detail: &'a str,
}

/// Encode a HelloReply message into `buf`.
///
/// Always returns `Some`.
///
/// # Panics
///
/// Panics if `buf` is too small.
pub fn hello_reply<'a>(buf: &'a mut [u8], info: &HelloReply<'_>) -> Option<&'a [u8]> {
    let mut b = Builder::new(buf);
    let name = b.string(info.firmware_name);
    let version = b.string(info.version);
    let commit = b.string(info.commit);
    let build_time = b.string(info.build_time);
    let detail = b.string(info.detail);
    let t = b.table(&[
        (0, Value::U8(role::DEMO)),
        (1, Value::Offset(name)),
        (2, Value::Offset(version)),
        (3, Value::Offset(commit)),
        (4, Value::Offset(build_time)),
        (5, Value::U64(info.uid)),
        (6, Value::U32(info.dev_id)),
        (7, Value::U32(info.rev_id)),
        (8, Value::U8(info.flash_mode)),
        (9, Value::U16(frame::PROTO_MAJOR as u16)),
        (10, Value::U16(PROTOCOL_MINOR)),
        (11, Value::U16(MAX_FRAME as u16)),
        (12, Value::Offset(detail)),
    ]);
    let m = b.message(body::HELLO_REPLY, t);
    Some(b.finish(m))
}

/// Encode a Pong message with the uptime in ms.
///
/// # Panics
///
/// Panics if `buf` is too small.
pub fn pong(buf: &mut [u8], uptime_ms: u64) -> Option<&[u8]> {
    let mut b = Builder::new(buf);
    let t = b.table(&[(0, Value::U64(uptime_ms))]);
    let m = b.message(body::PONG, t);
    Some(b.finish(m))
}

/// Status `fault` table contents.
pub struct FaultInfoData<'a> {
    /// A fault record exists.
    pub present: bool,
    /// Reason name.
    pub reason: &'a str,
    /// SCB->CFSR.
    pub cfsr: u32,
    /// SCB->HFSR.
    pub hfsr: u32,
    /// SCB->MMFAR.
    pub mmfar: u32,
    /// SCB->BFAR.
    pub bfar: u32,
    /// Stacked PC.
    pub pc: u32,
    /// Stacked LR.
    pub lr: u32,
    /// Thread name.
    pub task: &'a str,
    /// Source name.
    pub file: &'a str,
    /// Source line.
    pub line: u32,
}

/// Status entry for one LED.
pub struct LedStateData {
    /// [`pattern`] code.
    pub pattern: u8,
    /// Period in ms.
    pub period_ms: u16,
}

/// Status entry for one thread.
pub struct TaskStatsData<'a> {
    /// Thread name.
    pub name: &'a str,
    /// Zephyr priority.
    pub priority: u8,
    /// Thread state code.
    pub state: u8,
    /// Smallest unused stack seen, in bytes.
    pub stack_high_water: u16,
}

/// Status contents.
pub struct Status<'a> {
    /// Firmware name.
    pub firmware_name: &'a str,
    /// Version.
    pub version: &'a str,
    /// Git commit.
    pub commit: &'a str,
    /// Build time, ISO 8601 UTC.
    pub build_time: &'a str,
    /// 64-bit device UID.
    pub uid: u64,
    /// Uptime in ms.
    pub uptime_ms: u64,
    /// [`slot`] value.
    pub slot: u8,
    /// [`trial`] value.
    pub trial: u8,
    /// Trial boots counted by BOOT.
    pub trial_count: u8,
    /// [`health`] value.
    pub health: u8,
    /// RCC_CSR reset flags.
    pub reset_cause: u32,
    /// Reset cause name.
    pub reset_cause_text: &'a str,
    /// Saved fault, if any.
    pub fault: Option<FaultInfoData<'a>>,
    /// LD1..LD3.
    pub leds: [LedStateData; 3],
    /// Thread entries; only the first 8 are encoded.
    pub tasks: &'a [TaskStatsData<'a>],
    /// Free heap in bytes.
    pub heap_free: u32,
    /// Lowest free heap in bytes.
    pub heap_min_free: u32,
}

/// Encode a Status message into `buf`.
///
/// Always returns `Some`.
///
/// # Panics
///
/// Panics if `buf` is too small.
pub fn status<'a>(buf: &'a mut [u8], s: &Status<'_>) -> Option<&'a [u8]> {
    let mut b = Builder::new(buf);

    let firmware_name = b.string(s.firmware_name);
    let version = b.string(s.version);
    let commit = b.string(s.commit);
    let build_time = b.string(s.build_time);
    let reset_cause_text = b.string(s.reset_cause_text);

    let fault = s.fault.as_ref().map(|f| {
        let reason = b.string(f.reason);
        let task = b.string(f.task);
        let file = b.string(f.file);
        b.table(&[
            (0, Value::U8(f.present as u8)),
            (1, Value::Offset(reason)),
            (2, Value::U32(f.cfsr)),
            (3, Value::U32(f.hfsr)),
            (4, Value::U32(f.mmfar)),
            (5, Value::U32(f.bfar)),
            (6, Value::U32(f.pc)),
            (7, Value::U32(f.lr)),
            (8, Value::Offset(task)),
            (9, Value::Offset(file)),
            (10, Value::U32(f.line)),
        ])
    });

    let mut led_refs = [0usize; 3];
    for (i, led) in s.leds.iter().enumerate() {
        led_refs[i] = b.table(&[
            (0, Value::U8(led::LD1 + i as u8)),
            (1, Value::U8(led.pattern)),
            (2, Value::U16(led.period_ms)),
        ]);
    }
    let leds = b.offset_vector(&led_refs);

    let mut task_refs = [0usize; 8];
    for (i, task) in s.tasks.iter().enumerate().take(task_refs.len()) {
        let name = b.string(task.name);
        task_refs[i] = b.table(&[
            (0, Value::Offset(name)),
            (1, Value::U8(task.priority)),
            (2, Value::U8(task.state)),
            (3, Value::U16(task.stack_high_water)),
        ]);
    }
    let tasks = b.offset_vector(&task_refs[..s.tasks.len().min(task_refs.len())]);

    let mut fields = [(0u8, Value::U8(0)); 19];
    let mut n = 0usize;
    fields[n] = (0, Value::Offset(firmware_name));
    n += 1;
    fields[n] = (1, Value::Offset(version));
    n += 1;
    fields[n] = (2, Value::Offset(commit));
    n += 1;
    fields[n] = (3, Value::Offset(build_time));
    n += 1;
    fields[n] = (4, Value::U64(s.uid));
    n += 1;
    fields[n] = (5, Value::U64(s.uptime_ms));
    n += 1;
    fields[n] = (6, Value::U8(s.slot));
    n += 1;
    fields[n] = (7, Value::U8(s.trial));
    n += 1;
    fields[n] = (8, Value::U8(s.trial_count));
    n += 1;
    fields[n] = (9, Value::U8(s.health));
    n += 1;
    fields[n] = (10, Value::U32(s.reset_cause));
    n += 1;
    fields[n] = (11, Value::Offset(reset_cause_text));
    n += 1;
    if let Some(fault) = fault {
        fields[n] = (12, Value::Offset(fault));
        n += 1;
    }
    fields[n] = (13, Value::Offset(leds));
    n += 1;
    fields[n] = (14, Value::Offset(tasks));
    n += 1;
    fields[n] = (15, Value::U32(s.heap_free));
    n += 1;
    fields[n] = (16, Value::U32(s.heap_min_free));
    n += 1;
    fields[n] = (17, Value::U16(frame::PROTO_MAJOR as u16));
    n += 1;
    fields[n] = (18, Value::U16(PROTOCOL_MINOR));
    n += 1;

    let t = b.table(&fields[..n]);
    let m = b.message(body::STATUS, t);
    Some(b.finish(m))
}

/// Encode an Ack for request `seq`.
///
/// # Panics
///
/// Panics if `buf` is too small.
pub fn ack(buf: &mut [u8], seq: u16) -> Option<&[u8]> {
    let mut b = Builder::new(buf);
    let t = b.table(&[(0, Value::U16(seq)), (1, Value::U32(0))]);
    let m = b.message(body::ACK, t);
    Some(b.finish(m))
}

/// Encode a Nak for request `seq` with an [`error`] code and detail text.
///
/// # Panics
///
/// Panics if `buf` is too small.
pub fn nak<'a>(buf: &'a mut [u8], seq: u16, code: u16, detail: &str) -> Option<&'a [u8]> {
    let mut b = Builder::new(buf);
    let detail = b.string(detail);
    let t = b.table(&[
        (0, Value::U16(seq)),
        (1, Value::U16(code)),
        (2, Value::Offset(detail)),
        (3, Value::U32(0)),
    ]);
    let m = b.message(body::NAK, t);
    Some(b.finish(m))
}

/// Encode a ButtonEvent message.
///
/// # Panics
///
/// Panics if `buf` is too small.
pub fn button_event(buf: &mut [u8], press: u8, pattern: u8, period_ms: u16) -> Option<&[u8]> {
    let mut b = Builder::new(buf);
    let t = b.table(&[
        (0, Value::U8(press)),
        (1, Value::U8(pattern)),
        (2, Value::U16(period_ms)),
    ]);
    let m = b.message(body::BUTTON_EVENT, t);
    Some(b.finish(m))
}
