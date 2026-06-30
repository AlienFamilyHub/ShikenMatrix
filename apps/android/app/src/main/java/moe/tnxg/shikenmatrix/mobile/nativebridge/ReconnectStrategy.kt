package moe.tnxg.shikenmatrix.mobile.nativebridge

import kotlin.math.min

/**
 * 指数退避重连策略，带上限与抖动，避免在 server 不可达时打满 CPU/网络。
 */
class ReconnectStrategy(
    private val baseDelayMs: Long = 1_000L,
    private val maxDelayMs: Long = 30_000L,
) {
    private var attempts = 0

    fun nextDelayMs(): Long {
        val expo = baseDelayMs shl attempts.coerceAtMost(10)
        val capped = min(expo, maxDelayMs)
        attempts += 1
        val jitter = (Math.random() * capped * 0.2).toLong() - capped / 10
        return (capped + jitter).coerceAtLeast(100L)
    }

    fun reset() {
        attempts = 0
    }
}