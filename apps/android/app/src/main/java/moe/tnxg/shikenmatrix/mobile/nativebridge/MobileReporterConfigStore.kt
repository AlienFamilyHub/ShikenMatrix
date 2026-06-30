package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.Context

data class MobileReporterConfig(
    val serverUrl: String = "ws://127.0.0.1:4317/mobile",
    val apiKey: String = "",
    val intervalMs: Long = 15_000L,
    val autoReport: Boolean = false,
)

object MobileReporterConfigStore {
    private const val PREFS = "shikenmatrix_mobile_reporter"
    private const val KEY_SERVER_URL = "server_url"
    private const val KEY_API_KEY = "api_key"
    private const val KEY_INTERVAL_MS = "interval_ms"
    private const val KEY_AUTO_REPORT = "auto_report"

    fun read(context: Context): MobileReporterConfig {
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        return MobileReporterConfig(
            serverUrl = prefs.getString(KEY_SERVER_URL, null)?.takeIf(String::isNotBlank)
                ?: "ws://127.0.0.1:4317/mobile",
            apiKey = prefs.getString(KEY_API_KEY, "").orEmpty(),
            intervalMs = prefs.getLong(KEY_INTERVAL_MS, 15_000L).coerceAtLeast(5_000L),
            autoReport = prefs.getBoolean(KEY_AUTO_REPORT, false),
        )
    }

    fun save(context: Context, config: MobileReporterConfig) {
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            .edit()
            .putString(KEY_SERVER_URL, config.serverUrl.trim())
            .putString(KEY_API_KEY, config.apiKey.trim())
            .putLong(KEY_INTERVAL_MS, config.intervalMs.coerceAtLeast(5_000L))
            .putBoolean(KEY_AUTO_REPORT, config.autoReport)
            .apply()
    }

    fun setAutoReport(context: Context, enabled: Boolean) {
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            .edit()
            .putBoolean(KEY_AUTO_REPORT, enabled)
            .apply()
    }
}
