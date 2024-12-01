/**
 * @file    debug_log.c
 * @brief   Debug text on USART6 in the family line format.
 * @details USART3 carries HWLINK only, so Zephyr's console and log go to USART6
 *          (PG14, Arduino D1, 115200 8N1; see the board overlay).  Logging is
 *          deferred: LOG_INF() and friends copy the message and Zephyr's log
 *          thread writes it, so no application thread waits on the UART.
 *          Every line uses the format BOOT and the other firmwares use:
 *
 *              [   5012] R-ZEP I health: passed, CONFIRMED written
 *
 *          printk() output (level NONE) is written unchanged.
 * @author  Sandesh Ghimire <sandesh@soccentric.com>
 * @date    2026-11-06
 * @target  NUCLEO-F767ZI
 * @srs     RZDBG-001..RZDBG-003
 */

#include <string.h>

#include <zephyr/init.h>
#include <zephyr/logging/log.h>
#include <zephyr/logging/log_output.h>
#include <zephyr/logging/log_output_custom.h>
#include <zephyr/sys/cbprintf.h>
#include <zephyr/sys/printk.h>

#include "platform.h"

/* The application's only log module; other files LOG_MODULE_DECLARE it. */
LOG_MODULE_REGISTER(app, CONFIG_LOG_DEFAULT_LEVEL);

#define DEBUG_LOG_TAG  "R-ZEP" /**< Firmware tag in every line. */
#define LINE_MAX       160U    /**< Formatted line size, CR LF included. */

/** @brief One formatted output line. */
struct line_buf {
	char text[LINE_MAX]; /**< Line text, not NUL-terminated. */
	size_t len;          /**< Bytes used in text. */
};

/**
 * @brief Append one character to a line_buf, cbprintf output callback.
 * @details Drops characters that would not leave room for CR LF.
 *
 * @param[in]     c   Character to append.
 * @param[in,out] ctx struct line_buf being filled.
 *
 * @return @p c, so formatting continues.
 */
static int line_put(int c, void *ctx)
{
	struct line_buf *line = ctx;

	/* Keep two bytes for CR LF. */
	if (line->len < LINE_MAX - 2U) {
		line->text[line->len++] = (char)c;
	}
	return c;
}

/**
 * @brief Format one log message as "[ms] R-ZEP L text" CR LF.
 * @details Runs in Zephyr's log thread.  Level NONE (printk) is written as is.
 *
 * @param[in] output Log backend output.
 * @param[in] msg    Message to format.
 * @param[in] flags  Unused.
 */
static void debug_log_format(const struct log_output *output,
			     struct log_msg *msg, uint32_t flags)
{
	static const char level_char[] = "?EWID";
	struct line_buf line = { .len = 0U };
	uint8_t level = log_msg_get_level(msg);

	ARG_UNUSED(flags);

	if (level != LOG_LEVEL_NONE) {
		uint32_t ms = (uint32_t)(log_output_timestamp_to_us(
					 log_msg_get_timestamp(msg)) / 1000U);
		int n = snprintk(line.text, LINE_MAX - 2U,
				 "[%7u] " DEBUG_LOG_TAG " %c ", ms,
				 level_char[level <= 4U ? level : 0U]);

		line.len = (n > 0) ? MIN((size_t)n, LINE_MAX - 3U) : 0U;
	}

	size_t len;
	uint8_t *package = log_msg_get_package(msg, &len);

	if (len > 0U) {
		(void)cbpprintf(line_put, &line, package);
	}
	if (level != LOG_LEVEL_NONE) {
		line.text[line.len++] = '\r';
		line.text[line.len++] = '\n';
	}
	log_output_write(output->func, (uint8_t *)line.text, line.len,
			 output->control_block->ctx);
}

/**
 * @brief Install debug_log_format() as the custom log formatter.
 *
 * @return 0.
 */
static int debug_log_init(void)
{
	log_custom_output_msg_set(debug_log_format);
	return 0;
}

SYS_INIT(debug_log_init, PRE_KERNEL_1, 0);

void plat_log(uint8_t level, const uint8_t *text, uint32_t len)
{
	char line[PLAT_LOG_TEXT_MAX + 1U];
	uint32_t n = MIN(len, PLAT_LOG_TEXT_MAX);

	memcpy(line, text, n);
	line[n] = '\0';
	switch (level) {
	case 'E':
		LOG_ERR("%s", line);
		break;
	case 'W':
		LOG_WRN("%s", line);
		break;
	case 'D':
		LOG_DBG("%s", line);
		break;
	default:
		LOG_INF("%s", line);
		break;
	}
}
