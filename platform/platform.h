/**
 * @file    platform.h
 * @brief   C platform layer called from the Rust application.
 * @details Wraps the Zephyr UART, watchdog, devicetree partition and fatal-error
 *          paths that the zephyr crate does not bind yet.  Every function takes
 *          fixed arguments; no application logic lives here.
 *          src/platform/plat.rs declares the same functions for Rust; keep the
 *          two files in step.
 * @author  Sandesh Ghimire <sandesh@soccentric.com>
 * @date    2026-11-04
 * @target  NUCLEO-F767ZI (Zephyr board nucleo_f767zi)
 * @srs     RZCON-004, RZBT-002, RZBT-006, RZFN-003, RZRT-004, RZGEN-004
 */

#ifndef RZDEMO_PLATFORM_H
#define RZDEMO_PLATFORM_H

#include <stdbool.h>
#include <stdint.h>

/**
 * @defgroup platform Rust platform layer
 * @brief Driver glue exported to the Rust crate.
 * @{
 */

#define PLAT_FAULT_TASK_LEN 16U /**< Bytes in plat_fault::task, NUL included. */
#define PLAT_FAULT_FILE_LEN 32U /**< Bytes in plat_fault::file, NUL included. */

/**
 * @brief Copy of the no-init fault record.
 * @note Same layout as the C demo's record and as plat::Fault in Rust
 *       (repr(C)); do not reorder.
 */
struct plat_fault {
	uint32_t reason;                /**< FAULT_* reason code, 0..8. */
	uint32_t cfsr;                  /**< SCB->CFSR at the fault. */
	uint32_t hfsr;                  /**< SCB->HFSR at the fault. */
	uint32_t mmfar;                 /**< SCB->MMFAR at the fault. */
	uint32_t bfar;                  /**< SCB->BFAR at the fault. */
	uint32_t pc;                    /**< Stacked PC, 0 without an ESF. */
	uint32_t lr;                    /**< Stacked LR, 0 without an ESF. */
	uint32_t psr;                   /**< Stacked xPSR, 0 without an ESF. */
	uint32_t line;                  /**< Source line; always 0 on Zephyr. */
	char task[PLAT_FAULT_TASK_LEN]; /**< NUL-terminated thread name. */
	char file[PLAT_FAULT_FILE_LEN]; /**< NUL-terminated source; "zephyr". */
};

/** Longest debug text accepted by plat_log(), in bytes. */
#define PLAT_LOG_TEXT_MAX 120U

/**
 * @brief Queue one debug line for USART6 (RZDBG-001).
 * @details Copies the text, so @p text may be reused on return.  Never blocks.
 *
 * @param[in] level 'E', 'W', 'I' or 'D'; anything else logs as 'I'.
 * @param[in] text  Text bytes, not NUL-terminated.
 * @param[in] len   Length of @p text; cut to PLAT_LOG_TEXT_MAX.
 */
void plat_log(uint8_t level, const uint8_t *text, uint32_t len);

/**
 * @brief Advance the no-init boot counter (RZGEN-004).
 * @note Call once, first thing in rust_main().
 */
void plat_fault_boot_start(void);

/**
 * @brief Copy the saved fault record.
 *
 * @param[out] out Destination; written only on success.
 *
 * @return true if a valid record exists.
 */
bool plat_fault_get(struct plat_fault *out);

/**
 * @brief Check whether the saved fault belongs to this boot.
 *
 * @return true if a valid record carries the current boot id.
 */
bool plat_fault_is_current_boot(void);

/**
 * @brief Check whether the saved fault belongs to the previous boot.
 *
 * @return true if a valid record carries the previous boot id.
 */
bool plat_fault_is_previous_boot(void);

/** @brief Invalidate the saved fault record. */
void plat_fault_clear(void);

/**
 * @brief Raise k_panic(); the fatal handler saves a record and resets.
 * @note Does not return.
 */
void plat_fatal(void);

/**
 * @brief Read SCB->VTOR.
 *
 * @return Active vector table address.
 */
uint32_t plat_vtor(void);

/**
 * @brief Flush the log and cold-reset the MCU.
 * @note Does not return.
 */
void plat_reboot(void);

/**
 * @brief Invalidate a D-cache range.
 *
 * @param[in] addr Start address.
 * @param[in] len  Length in bytes.
 */
void plat_dcache_invalidate(uintptr_t addr, uint32_t len);

/**
 * @brief Name the calling thread.
 *
 * @param[in] name NUL-terminated name; must outlive the thread.
 */
void plat_thread_name(const char *name);

/**
 * @brief Measure the calling thread's unused stack.
 *
 * @return Unused bytes, or 0 if the query fails.
 */
uint32_t plat_stack_unused(void);

/**
 * @brief Take over the IWDG with a 4 s timeout (RZRT-004).
 *
 * @return true if the driver is installed and fed.
 */
bool plat_wdt_init(void);

/** @brief Feed the IWDG through the driver, if it is ready. */
void plat_wdt_feed(void);

/**
 * @brief Configure LD1..LD3 and B1 from the devicetree aliases.
 *
 * @return true if every pin and the B1 edge interrupt are set up.
 */
bool plat_gpio_init(void);

/**
 * @brief Drive one user LED.
 *
 * @param[in] led 0 = LD1, 1 = LD2, 2 = LD3; others are ignored.
 * @param[in] on  true to light the LED.
 */
void plat_led_set(uint32_t led, bool on);

/**
 * @brief Read B1.
 *
 * @return true while B1 is pressed (raw level, no debounce).
 */
bool plat_button_pressed(void);

/**
 * @brief Block until a B1 edge or the timeout.
 *
 * @param[in] timeout_ms Timeout in ms.
 */
void plat_button_wait(uint32_t timeout_ms);

/**
 * @brief Open USART3 at 115200 8N1 and start reception (RZFN-003).
 *
 * @return true on success.
 */
bool plat_uart_init(void);

/**
 * @brief Change the USART3 baud rate.
 * @details Restarts reception and drops buffered bytes.  On failure the old
 *          rate is restored.
 *
 * @param[in] baud 115200 or 921600.
 *
 * @return true if @p baud is now active.
 */
bool plat_uart_set_baud(uint32_t baud);

/**
 * @brief Copy received bytes out of the RX ring.
 *
 * @param[out] dst     Destination buffer.
 * @param[in]  max_len Capacity of @p dst in bytes.
 *
 * @return Bytes copied, 0 if none are buffered.
 */
uint32_t plat_uart_read(uint8_t *dst, uint32_t max_len);

/**
 * @brief Block until bytes are received or the timeout.
 *
 * @param[in] timeout_ms Timeout in ms.
 */
void plat_uart_wait(uint32_t timeout_ms);

/**
 * @brief Check that USART3 reception is enabled.
 *
 * @return true if the receiver runs.
 */
bool plat_uart_rx_running(void);

/**
 * @brief Send one frame on USART3 and wait until it has left the wire.
 * @details Blocks the caller; the ISR feeds the bytes.
 *
 * @param[in] data       Frame bytes.
 * @param[in] len        Frame length, at most 2200 bytes.
 * @param[in] timeout_ms Timeout for the ISR to drain the frame, in ms.
 *
 * @return true if sent; true for an empty frame; false if too long or timed out.
 */
bool plat_uart_send(const uint8_t *data, uint32_t len, uint32_t timeout_ms);

/**
 * @brief Read the 32-byte CONFIRMED record of the running slot (RZBT-006).
 * @note The record address is fixed in platform.c, never passed in.
 *
 * @param[out] out32 32-byte destination.
 *
 * @return true on success.
 */
bool plat_confirmed_read(uint8_t *out32);

/**
 * @brief Program the 32-byte CONFIRMED record of the running slot (RZBT-006).
 * @note The record address is fixed in platform.c, never passed in.  The
 *       caller checks the record is erased first.
 *
 * @param[in] record32 32-byte record.
 *
 * @return true if the flash write succeeded.
 */
bool plat_confirmed_write(const uint8_t *record32);

/** @} */

#endif /* RZDEMO_PLATFORM_H */
