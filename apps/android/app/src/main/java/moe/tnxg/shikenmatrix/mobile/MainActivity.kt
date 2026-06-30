package moe.tnxg.shikenmatrix.mobile

import android.Manifest
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import moe.tnxg.shikenmatrix.mobile.nativebridge.BackgroundReporter
import moe.tnxg.shikenmatrix.mobile.nativebridge.DeviceSnapshotCollector
import moe.tnxg.shikenmatrix.mobile.nativebridge.KeepAliveController
import moe.tnxg.shikenmatrix.mobile.nativebridge.MobileReporterConfigStore
import moe.tnxg.shikenmatrix.mobile.nativebridge.RootManager
import moe.tnxg.shikenmatrix.mobile.ui.ShikenMatrixScreen
import top.yukonga.miuix.kmp.theme.MiuixTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            MiuixTheme {
                ShikenMatrixScreen(
                    activity = this,
                    initialConfig = MobileReporterConfigStore.read(applicationContext),
                    saveConfig = { config -> MobileReporterConfigStore.save(applicationContext, config) },
                    collectSnapshot = { DeviceSnapshotCollector(applicationContext).collectSnapshot() },
                    configureBackgroundReporter = { serverUrl, apiKey, intervalMs, enabled ->
                        BackgroundReporter.configure(applicationContext, serverUrl, apiKey, intervalMs, enabled)
                    },
                    startKeepAlive = ::startKeepAlive,
                    stopKeepAlive = ::stopKeepAlive,
                    isKeepAliveEnabled = { KeepAliveController.isEnabled(applicationContext) },
                    requestRuntimePermissions = ::requestRuntimePermissions,
                    openSettings = ::openSettings,
                    requestRoot = { RootManager(applicationContext).requestRoot() },
                    openRootManager = { RootManager(applicationContext).openRootManager() },
                )
            }
        }
    }

    private fun startKeepAlive() {
        KeepAliveController.enable(this)
    }

    private fun stopKeepAlive() {
        BackgroundReporter.disable(this)
        KeepAliveController.disable(this)
    }

    private fun requestRuntimePermissions() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            requestPermissions(
                arrayOf(
                    Manifest.permission.ACCESS_COARSE_LOCATION,
                    Manifest.permission.POST_NOTIFICATIONS,
                ),
                34951,
            )
            return
        }

        requestPermissions(arrayOf(Manifest.permission.ACCESS_COARSE_LOCATION), 34951)
    }

    private fun openSettings(kind: SettingsKind) {
        val intent = when (kind) {
            SettingsKind.UsageAccess -> Intent(Settings.ACTION_USAGE_ACCESS_SETTINGS)
            SettingsKind.NotificationListener -> Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS)
            SettingsKind.BatteryOptimization -> Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS)
            SettingsKind.Location -> Intent(Settings.ACTION_LOCATION_SOURCE_SETTINGS)
            SettingsKind.AppDetails -> Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS)
                .setData(Uri.parse("package:$packageName"))
        }.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        startActivity(intent)
    }

    enum class SettingsKind {
        UsageAccess,
        NotificationListener,
        BatteryOptimization,
        Location,
        AppDetails,
    }
}
