package moe.tnxg.shikenmatrix.mobile

import android.Manifest
import android.app.AlertDialog
import android.content.Intent
import android.net.Uri
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import moe.tnxg.shikenmatrix.mobile.nativebridge.BackgroundReporter
import moe.tnxg.shikenmatrix.mobile.nativebridge.DeviceSnapshotCollector
import moe.tnxg.shikenmatrix.mobile.nativebridge.MobileReporterConfigStore
import moe.tnxg.shikenmatrix.mobile.nativebridge.BackgroundReportingController
import moe.tnxg.shikenmatrix.mobile.ui.ShikenMatrixScreen
import top.yukonga.miuix.kmp.theme.MiuixTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        // Restore configured reporting; screen state is metadata, not connectivity.
        BackgroundReportingController.sync(applicationContext)
        BackgroundReporter.start(applicationContext)

        setContent {
            MiuixTheme {
                ShikenMatrixScreen(
                    activity = this,
                    initialConfig = MobileReporterConfigStore.read(applicationContext),
                    saveConfig = { config -> MobileReporterConfigStore.save(applicationContext, config) },
                    collectSnapshot = { DeviceSnapshotCollector(applicationContext).collectSnapshot() },
                    configureBackgroundReporter = { serverUrl, apiKey, intervalMs, enabled ->
                        BackgroundReporter.configure(applicationContext, serverUrl, apiKey, intervalMs, enabled)
                        BackgroundReportingController.resume(applicationContext)
                    },
                    requestRuntimePermissions = ::requestRuntimePermissions,
                    requestReportingNotificationPermission = ::requestReportingNotificationPermission,
                    openSettings = ::openSettings,
                )
            }
        }
    }

    override fun onResume() {
        super.onResume()
        BackgroundReporter.start(applicationContext)
        BackgroundReportingController.resume(applicationContext)
    }

    private fun requestReportingNotificationPermission() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
            checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) {
            AlertDialog.Builder(this)
                .setTitle("允许后台上报通知？")
                .setMessage("定时上报会运行低优先级通知的前台服务，以尽量在熄屏时更新歌曲和设备状态。" +
                    "通知权限仅用于显示提示；拒绝或暂不授权都不会阻止上报。")
                .setPositiveButton("继续") { _, _ ->
                    requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 34952)
                }
                .setNegativeButton("暂不", null)
                .show()
        }
    }

    private fun requestRuntimePermissions() {
        requestPermissions(arrayOf(Manifest.permission.ACCESS_COARSE_LOCATION), 34951)
    }

    private fun openSettings(kind: SettingsKind) {
        val intent = when (kind) {
            SettingsKind.UsageAccess -> Intent(Settings.ACTION_USAGE_ACCESS_SETTINGS)
            SettingsKind.NotificationListener -> Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS)
            SettingsKind.Location -> Intent(Settings.ACTION_LOCATION_SOURCE_SETTINGS)
            SettingsKind.BatteryOptimization -> Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS)
            SettingsKind.AppDetails -> Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS)
                .setData(Uri.parse("package:$packageName"))
        }.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        startActivity(intent)
    }

    enum class SettingsKind {
        UsageAccess,
        NotificationListener,
        Location,
        AppDetails,
        BatteryOptimization,
    }
}
