package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/** Best-effort foreground-service recovery without alarms or wake locks. */
class ReportingBootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_BOOT_COMPLETED && intent.action != Intent.ACTION_MY_PACKAGE_REPLACED) return
        val config = MobileReporterConfigStore.read(context)
        if (!config.autoReport || config.apiKey.isBlank()) return
        BackgroundReportingController.sync(context.applicationContext)
        BackgroundReporter.start(context.applicationContext)
    }
}
