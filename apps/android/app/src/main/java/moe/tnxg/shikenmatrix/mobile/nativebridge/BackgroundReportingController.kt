package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.Context
import android.content.Intent
import android.os.Build

/** Reconciles user-enabled background reporting with Android foreground-service limits. */
object BackgroundReportingController {
    private const val STATE = "shikenmatrix_background_reporting_state"
    @Volatile private var appContext: Context? = null
    @Volatile var serviceRunning: Boolean = false
        private set
    @Volatile var serviceStatusMessage: String = "后台服务未运行"
        private set
    @Volatile var onStateChanged: (() -> Unit)? = null

    fun sync(context: Context) {
        val app = context.applicationContext
        rememberContext(app)
        val config = MobileReporterConfigStore.read(app)
        val intent = Intent(app, BackgroundReportingService::class.java)
        if (config.autoReport && config.apiKey.isNotBlank() && !isServiceSuspended(app)) {
            runCatching {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) app.startForegroundService(intent)
                else app.startService(intent)
            }.onFailure { suspendBackgroundReporting(app, "后台服务启动失败：${it.message}；请回到应用重试") }
        } else {
            app.stopService(intent)
        }
    }

    /** Only a visible foreground/config action retries an OS-suspended service. */
    fun resume(context: Context) {
        rememberContext(context.applicationContext)
        context.getSharedPreferences(STATE, Context.MODE_PRIVATE).edit()
            .putBoolean("service_suspended", false).apply()
        sync(context)
    }

    internal fun isServiceSuspended(context: Context): Boolean =
        context.getSharedPreferences(STATE, Context.MODE_PRIVATE).getBoolean("service_suspended", false)

    internal fun suspendBackgroundReporting(context: Context, message: String) {
        rememberContext(context.applicationContext)
        context.getSharedPreferences(STATE, Context.MODE_PRIVATE).edit()
            .putBoolean("service_suspended", true).apply()
        context.stopService(Intent(context, BackgroundReportingService::class.java))
        serviceState(false, message)
        BackgroundReporter.start(context.applicationContext)
    }

    internal fun serviceState(running: Boolean, message: String) {
        serviceRunning = running
        serviceStatusMessage = message
        appContext?.getSharedPreferences(STATE, Context.MODE_PRIVATE)?.edit()
            ?.putString("service_status", message)?.apply()
        onStateChanged?.invoke()
    }

    private fun rememberContext(context: Context) {
        if (appContext == null) {
            appContext = context.applicationContext
            serviceStatusMessage = context.getSharedPreferences(STATE, Context.MODE_PRIVATE)
                .getString("service_status", null) ?: serviceStatusMessage
        }
    }
}
