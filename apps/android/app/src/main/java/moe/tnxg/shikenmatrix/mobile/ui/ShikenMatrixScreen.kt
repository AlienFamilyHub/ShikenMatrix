package moe.tnxg.shikenmatrix.mobile.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.calculateEndPadding
import androidx.compose.foundation.layout.calculateStartPadding
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.unit.dp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import moe.tnxg.shikenmatrix.mobile.nativebridge.BackgroundReportingController
import moe.tnxg.shikenmatrix.mobile.MainActivity
import moe.tnxg.shikenmatrix.mobile.nativebridge.BackgroundReporter
import moe.tnxg.shikenmatrix.mobile.nativebridge.DeviceSnapshot
import moe.tnxg.shikenmatrix.mobile.nativebridge.MobileReporterConfig
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import kotlin.concurrent.thread

@Composable
internal fun ShikenMatrixScreen(
    activity: MainActivity,
    initialConfig: MobileReporterConfig,
    saveConfig: (MobileReporterConfig) -> Unit,
    collectSnapshot: () -> DeviceSnapshot,
    configureBackgroundReporter: (String, String, Long, Boolean) -> Unit,
    requestRuntimePermissions: () -> Unit,
    requestReportingNotificationPermission: () -> Unit,
    openSettings: (MainActivity.SettingsKind) -> Unit,
) {
    var serverUrl by remember { mutableStateOf(initialConfig.serverUrl) }
    var apiKey by remember { mutableStateOf(initialConfig.apiKey) }
    var reportIntervalMs by remember { mutableStateOf(initialConfig.intervalMs.toString()) }
    var connected by remember { mutableStateOf(BackgroundReporter.isConnected) }
    var autoReport by remember { mutableStateOf(initialConfig.autoReport) }
    var lastSnapshot by remember { mutableStateOf<DeviceSnapshot?>(null) }
    var selectedTab by remember { mutableIntStateOf(0) }
    var permissionStatus by remember { mutableStateOf(activity.probePermissionStatus()) }
    var backgroundServiceRunning by remember { mutableStateOf(BackgroundReportingController.serviceRunning) }
    var backgroundServiceStatus by remember { mutableStateOf(BackgroundReportingController.serviceStatusMessage) }
    val logs = remember { mutableStateListOf<String>() }
    val dashboardScrollState = rememberLazyListState()
    val settingsScrollState = rememberScrollState()
    val aboutScrollState = rememberScrollState()

    val lifecycleOwner = LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_RESUME) {
                permissionStatus = activity.probePermissionStatus()
                connected = BackgroundReporter.isConnected
                backgroundServiceRunning = BackgroundReportingController.serviceRunning
                backgroundServiceStatus = BackgroundReportingController.serviceStatusMessage
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }

    fun appendLog(message: String) {
        val timestamp = SimpleDateFormat("HH:mm:ss", Locale.getDefault()).format(Date())
        logs.add(0, "$timestamp $message")
        while (logs.size > 80) {
            logs.removeLast()
        }
    }

    fun normalizedInterval(): Long =
        (reportIntervalMs.toLongOrNull() ?: 15_000L).coerceAtLeast(5_000L)

    fun currentConfig(enabled: Boolean = autoReport): MobileReporterConfig =
        MobileReporterConfig(
            serverUrl = serverUrl,
            apiKey = apiKey,
            intervalMs = normalizedInterval(),
            autoReport = enabled,
        )

    DisposableEffect(Unit) {
        val previousObserver = BackgroundReporter.onSnapshotCollected
        BackgroundReporter.onSnapshotCollected = { snapshot ->
            // 从后台线程回写前台 UI —— 后台采集即自动刷新 Dashboard，无需手按
            activity.runOnUiThread { lastSnapshot = snapshot }
        }
        val previousConnectionObserver = BackgroundReporter.onConnectionChanged
        BackgroundReporter.onConnectionChanged = { isConnected ->
            activity.runOnUiThread { connected = isConnected }
        }
        connected = BackgroundReporter.isConnected
        val previousServiceObserver = BackgroundReportingController.onStateChanged
        BackgroundReportingController.onStateChanged = {
            activity.runOnUiThread {
                backgroundServiceRunning = BackgroundReportingController.serviceRunning
                backgroundServiceStatus = BackgroundReportingController.serviceStatusMessage
                permissionStatus = activity.probePermissionStatus()
            }
        }
        onDispose {
            BackgroundReporter.onSnapshotCollected = previousObserver
            BackgroundReporter.onConnectionChanged = previousConnectionObserver
            BackgroundReportingController.onStateChanged = previousServiceObserver
        }
    }

    // Foreground preview sampling must not outlive the visible/resumed Dashboard.
    LaunchedEffect(selectedTab, lifecycleOwner) {
        if (selectedTab != 0) return@LaunchedEffect
        lifecycleOwner.lifecycle.repeatOnLifecycle(Lifecycle.State.RESUMED) {
            while (true) {
                runCatching {
                    withContext(Dispatchers.IO) { collectSnapshot() }
                }.onSuccess { lastSnapshot = it }
                kotlinx.coroutines.delay(10_000L)
            }
        }
    }

    val layoutDirection = LocalLayoutDirection.current
    val safeDrawingPadding = WindowInsets.safeDrawing.asPaddingValues()
    val tabContentPadding = PaddingValues(
        start = safeDrawingPadding.calculateStartPadding(layoutDirection) + 20.dp,
        top = safeDrawingPadding.calculateTopPadding() + 20.dp,
        end = safeDrawingPadding.calculateEndPadding(layoutDirection) + 20.dp,
        bottom = 20.dp,
    )
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFFF6F7FB))
            .imePadding()
    ) {
        Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
            when (selectedTab) {
                0 -> DashboardTab(
                    connected = connected,
                    logs = logs,
                    lastSnapshot = lastSnapshot,
                    onConnect = onConnect@{
                        if (apiKey.isBlank()) {
                            appendLog("连接失败：请先填写并保存 API Key")
                            return@onConnect
                        }
                        autoReport = true
                        saveConfig(currentConfig(enabled = true))
                        configureBackgroundReporter(serverUrl, apiKey, normalizedInterval(), true)
                        appendLog("后台上报已启动，熄屏继续更新")
                        requestReportingNotificationPermission()
                    },
                    onCollectAndSend = {
                        thread(name = "snapshot-collector") {
                            runCatching {
                                val snapshot = collectSnapshot()
                                val sent = BackgroundReporter.reportSnapshot(activity.applicationContext, snapshot)
                                activity.runOnUiThread {
                                    lastSnapshot = snapshot
                                    appendLog("已采集设备快照")
                                    appendLog(if (sent) "已通过后台连接发送快照" else "未发送：后台连接未就绪")
                                }
                            }.onFailure { error ->
                                activity.runOnUiThread { appendLog("采集或发送失败：${error.message ?: "未知错误"}") }
                            }
                        }
                    },
                    contentPadding = tabContentPadding,
                    scrollState = dashboardScrollState,
                )

                1 -> SettingsTab(
                    serverUrl = serverUrl,
                    onServerUrlChange = { serverUrl = it },
                    apiKey = apiKey,
                    onApiKeyChange = { apiKey = it },
                    reportIntervalMs = reportIntervalMs,
                    onReportIntervalChange = { reportIntervalMs = it },
                    autoReport = autoReport,
                    onAutoReportChange = onAutoReportChange@{ enabled ->
                        if (enabled && apiKey.isBlank()) {
                            appendLog("无法启用后台上报：请先填写并保存 API Key")
                            return@onAutoReportChange
                        }
                        autoReport = enabled
                        saveConfig(currentConfig(enabled))
                        configureBackgroundReporter(serverUrl, apiKey, normalizedInterval(), enabled)
                        appendLog(if (enabled) "已启用定时上报（熄屏继续更新，屏幕状态不等于连接状态）" else "已关闭定时上报")
                        if (enabled) requestReportingNotificationPermission()
                    },
                    backgroundServiceRunning = backgroundServiceRunning,
                    backgroundServiceStatus = backgroundServiceStatus,
                    onRequestReportingNotificationPermission = requestReportingNotificationPermission,
                    onSaveConfig = {
                        saveConfig(currentConfig())
                        configureBackgroundReporter(serverUrl, apiKey, normalizedInterval(), autoReport)
                        appendLog("已保存移动端上报配置")
                    },
                    onRequestRuntimePermissions = {
                        requestRuntimePermissions()
                        appendLog("已请求运行时权限：位置")
                    },
                    onOpenUsageAccess = {
                        openSettings(MainActivity.SettingsKind.UsageAccess)
                        appendLog("已打开使用情况访问设置")
                    },
                    onOpenNotificationListener = {
                        openSettings(MainActivity.SettingsKind.NotificationListener)
                        appendLog("已打开通知监听器设置")
                    },
                    onOpenLocationSettings = { openSettings(MainActivity.SettingsKind.Location) },
                    onOpenBatteryOptimizationSettings = { openSettings(MainActivity.SettingsKind.BatteryOptimization) },
                    onOpenAppDetails = { openSettings(MainActivity.SettingsKind.AppDetails) },
                    permissionStatus = permissionStatus,
                    contentPadding = tabContentPadding,
                    scrollState = settingsScrollState,
                )

                2 -> AboutTab(
                    contentPadding = tabContentPadding,
                    scrollState = aboutScrollState,
                )
            }
        }

        ShikenNavigationBar(selectedTab = selectedTab, onTabSelect = { selectedTab = it })
    }
}
