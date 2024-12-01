//! Board, bootloader-contract and reset-cause constants.
//!
//! Author: Sandesh Ghimire <sandesh@soccentric.com>
//! SRS:    RZBT-001..006, RZGEN-005, RZSAFE-001..004
//!
//! Same values as the Rust Embassy demo's `board.rs` and the C demos'
//! `boot_contract.h` (SRS-BOOT-F767 LAY-001..LAY-005, HND-003, Appendix B).

/// Application link address and vector table base.
pub const SLOT_BASE: u32 = 0x0800_C000;
/// First word of a valid CONFIRMED record, "CONF".
pub const CONFIRMED_MAGIC: u32 = 0x434F_4E46;

/// "STAY" in RTC_BKP1R: BOOT stays in the bootloader after reset.
pub const STAY_IN_BOOT_MAGIC: u32 = 0x5941_5453;

// Hand-over record in RTC_BKP2R: bits 0..23 = magic "HND", bits 24..25 =
// slot, bit 26 = pending, bits 27..28 = trial count.

/// Hand-over magic, bytes "HND", 0.
pub const HANDOVER_MAGIC: u32 = 0x0044_4E48;
/// Bits holding [`HANDOVER_MAGIC`].
pub const HANDOVER_MAGIC_MASK: u32 = 0x00FF_FFFF;
/// Shift of the slot field.
pub const HANDOVER_SLOT_SHIFT: u32 = 24;
/// Mask of the slot field after shifting.
pub const HANDOVER_SLOT_MASK: u32 = 0x3;
/// Bit set while the slot is on trial.
pub const HANDOVER_PENDING_BIT: u32 = 26;
/// Shift of the trial count field.
pub const HANDOVER_TRIAL_SHIFT: u32 = 27;
/// Mask of the trial count field after shifting.
pub const HANDOVER_TRIAL_MASK: u32 = 0x3;

/// Slot not known (no valid hand-over).
pub const SLOT_UNKNOWN: u8 = 0;
/// Slot A.
pub const SLOT_A: u8 = 1;
/// Slot B.
pub const SLOT_B: u8 = 2;

/// Trial state not known.
pub const TRIAL_UNKNOWN: u8 = 0;
/// Slot is confirmed.
pub const TRIAL_CONFIRMED: u8 = 1;
/// Slot is on trial.
pub const TRIAL_PENDING: u8 = 2;

/// No health check needed (slot not on trial).
pub const HEALTH_NOT_REQUIRED: u8 = 0;
/// Health check in progress.
pub const HEALTH_RUNNING: u8 = 1;
/// Health check passed.
pub const HEALTH_PASSED: u8 = 2;
/// Health check failed.
pub const HEALTH_FAILED: u8 = 3;

/// RCC clock control and status register.
pub const RCC_CSR: *mut u32 = 0x4002_3874 as *mut u32;
/// Write 1 to clear the reset flags.
pub const RCC_CSR_RMVF: u32 = 1 << 24;
/// Brown-out reset flag.
pub const RCC_CSR_BORRSTF: u32 = 1 << 25;
/// NRST pin reset flag.
pub const RCC_CSR_PINRSTF: u32 = 1 << 26;
/// Power-on reset flag.
pub const RCC_CSR_PORRSTF: u32 = 1 << 27;
/// Software reset flag.
pub const RCC_CSR_SFTRSTF: u32 = 1 << 28;
/// Independent watchdog reset flag.
pub const RCC_CSR_WDGRSTF: u32 = 1 << 29;
/// Window watchdog reset flag.
pub const RCC_CSR_WWDGRSTF: u32 = 1 << 30;
/// Low-power reset flag.
pub const RCC_CSR_LPWRRSTF: u32 = 1 << 31;
/// All reset flags.
pub const RCC_CSR_RESET_MASK: u32 = RCC_CSR_BORRSTF
    | RCC_CSR_PINRSTF
    | RCC_CSR_PORRSTF
    | RCC_CSR_SFTRSTF
    | RCC_CSR_WDGRSTF
    | RCC_CSR_WWDGRSTF
    | RCC_CSR_LPWRRSTF;

/// RCC APB1 peripheral clock enable register.
pub const RCC_APB1ENR: *mut u32 = 0x4002_3840 as *mut u32;
/// PWR clock enable.
pub const RCC_APB1ENR_PWREN: u32 = 1 << 28;
/// RTC APB clock enable.
pub const RCC_APB1ENR_RTCEN: u32 = 1 << 10;
/// PWR control register.
pub const PWR_CR: *mut u32 = 0x4000_7000 as *mut u32;
/// Backup domain write access enable.
pub const PWR_CR_DBP: u32 = 1 << 8;
/// Backup register 1: stay-in-BOOT request.
pub const RTC_BKP1R: *mut u32 = 0x4000_2854 as *mut u32;
/// Backup register 2: BOOT hand-over record.
pub const RTC_BKP2R: *mut u32 = 0x4000_2858 as *mut u32;
/// RTC write protection register.
pub const RTC_WPR: *mut u32 = 0x4000_2824 as *mut u32;
/// First RTC unlock key.
pub const RTC_WPR_UNLOCK1: u32 = 0x00CA;
/// Second RTC unlock key.
pub const RTC_WPR_UNLOCK2: u32 = 0x0053;
/// Any other value re-locks the RTC.
pub const RTC_WPR_LOCK: u32 = 0xFF;

/// DBGMCU device ID and revision register.
pub const DBGMCU_IDCODE: *const u32 = 0xE004_2000 as *const u32;
/// STM32F76x/F77x device ID, reported when IDCODE reads 0.
pub const DBGMCU_DEV_ID: u32 = 0x451;
/// 96-bit factory unique ID, three words.
pub const UID_BASE: *const u32 = 0x1FF0_F420 as *const u32;
