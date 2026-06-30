package moe.tnxg.shikenmatrix.mobile.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import moe.tnxg.shikenmatrix.mobile.MainActivity
import moe.tnxg.shikenmatrix.mobile.nativebridge.BackgroundReporter
import moe.tnxg.shikenmatrix.mobile.nativebridge.DeviceSnapshot
import moe.tnxg.shikenmatrix.mobile.nativebridge.MobileReporterConfig
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import kotlin.concurrent.thread

@Composable
internal fun ShikenMatrixScreen(
  activity: MainActivity,
  initialConfig: MobileReporterConfig,
  saveConfig: (MobileReporterConfig) -> Unit,
  connectWebSocket: (
    String,
    String,
    () -> Unit,
    () -> Unit,
    (String) -> Unit,
    (String) -> Unit,
  ) -> Unit,
  closeWebSocket: () -> Unit,
  collectSnapshot: () -> DeviceSnapshot,
  sendSnapshot: (DeviceSnapshot) -> Boolean,
  configureBackgroundReporter: (String, String, Long, Boolean) -> Unit,
  startKeepAlive: () -> Unit,
  stopKeepAlive: () -> Unit,
  isKeepAliveEnabled: () -> Boolean,
  requestRuntimePermissions: () -> Unit,
  openSettings: (MainActivity.SettingsKind) -> Unit,
  requestRoot: () -> Any,
  openRootManager: () -> Any,
) {
  var serverUrl by remember { mutableStateOf(initialConfig.serverUrl) }
  var apiKey by remember { mutableStateOf(initialConfig.apiKey) }
  var reportIntervalMs by remember { mutableStateOf(initialConfig.intervalMs.toString()) }
  var connected by remember { mutableStateOf(false) }
  var keepAlive by remember { mutableStateOf(isKeepAliveEnabled()) }
  var autoReport by remember { mutableStateOf(initialConfig.autoReport) }
  var lastSnapshot by remember { mutableStateOf<DeviceSnapshot?>(null) }
  var selectedTab by remember { mutableIntStateOf(0) }
  var rootGranted by remember { mutableStateOf(false) }
  var rootMessage by remember { mutableStateOf("尚未申请 Root 权限") }
  val logs = remember { mutableStateListOf<String>() }
  val snapshotScope = rememberCoroutineScope()

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
    onDispose {
      BackgroundReporter.onSnapshotCollected = previousObserver
      closeWebSocket()
    }
  }

  // 进入 Dashboard 时若尚无采集结果，自动采集一次；并在前台 Dashboard 周期主动采集，
// 不再依赖用户手按"采集并发送"。
  LaunchedEffect(selectedTab) {
    if (selectedTab != 0) return@LaunchedEffect
    if (lastSnapshot == null) {
      snapshotScope.launch(Dispatchers.IO) {
        runCatching { collectSnapshot() }.onSuccess { snapshot ->
          activity.runOnUiThread { lastSnapshot = snapshot }
        }
      }
    }
    while (true) {
      kotlinx.coroutines.delay(10_000L)
      snapshotScope.launch(Dispatchers.IO) {
        runCatching { collectSnapshot() }.onSuccess { snapshot ->
          activity.runOnUiThread { lastSnapshot = snapshot }
        }
      }
    }
  }

  Column(
    modifier = Modifier
      .fillMaxSize()
      .background(Color(0xFFF6F7FB))
      .padding(top = WindowInsets.systemBars.asPaddingValues().calculateTopPadding())
  ) {
    Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
      when (selectedTab) {
        0 -> DashboardTab(
          connected = connected,
          keepAlive = keepAlive,
          logs = logs,
          lastSnapshot = lastSnapshot,
          onConnect = onConnect@{
            if (apiKey.isBlank()) {
              appendLog("连接失败：请先填写并保存 API Key")
              return@onConnect
            }
            saveConfig(currentConfig())
            connectWebSocket(
              serverUrl,
              apiKey,
              {
                connected = true
                configureBackgroundReporter(serverUrl, apiKey, normalizedInterval(), autoReport)
                startKeepAlive()
                keepAlive = true
                appendLog("WS 已连接并发送 hello")
              },
              {
                connected = false
                appendLog("WS 已断开")
              },
              { appendLog("server: $it") },
              { appendLog("WS 连接错误：$it") },
            )
          },
          onCollectAndSend = {
            thread(name = "snapshot-collector") {
              runCatching { collectSnapshot() }
                .onSuccess { snapshot ->
                  activity.runOnUiThread {
                    lastSnapshot = snapshot
                    appendLog("已采集设备快照")
                    appendLog(if (sendSnapshot(snapshot)) "已发送快照" else "未发送：WS 未连接")
                  }
                }
                .onFailure { error ->
                  activity.runOnUiThread { appendLog("采集失败：${error.message ?: "未知错误"}") }
                }
            }
          },
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
            appendLog(if (enabled) "已启用后台定时上报" else "已关闭后台定时上报")
          },
          onSaveConfig = {
            saveConfig(currentConfig())
            appendLog("已保存移动端上报配置")
          },
          keepAlive = keepAlive,
          onToggleKeepAlive = {
            if (keepAlive) {
              stopKeepAlive()
              keepAlive = false
              autoReport = false
              appendLog("已停止后台保活与 watchdog")
            } else {
              startKeepAlive()
              keepAlive = true
              appendLog("已启动后台保活与 watchdog")
            }
          },
          onRequestPermissions = {
            requestRuntimePermissions()
            openSettings(MainActivity.SettingsKind.UsageAccess)
            openSettings(MainActivity.SettingsKind.NotificationListener)
            appendLog("已触发普通权限申请与特殊权限设置入口")
          },
          onOpenBatterySettings = { openSettings(MainActivity.SettingsKind.BatteryOptimization) },
          onOpenLocationSettings = { openSettings(MainActivity.SettingsKind.Location) },
        )
        2 -> PrivacyTab(
          rootGranted = rootGranted,
          rootMessage = rootMessage,
          onRequestRoot = {
            thread(name = "root-request") {
              val result = requestRoot()
              val isGranted = (result as? JSONObject)?.optBoolean("granted", false) == true
              val message = (result as? JSONObject)?.optString("message").orEmpty()
              activity.runOnUiThread {
                rootGranted = isGranted
                rootMessage = message.ifBlank { if (isGranted) "Root 已授权" else "Root 未授权" }
                appendLog("Root 申请结果：$result")
              }
            }
          },
          onOpenRootManager = {
            appendLog("Root 管理器打开结果：${openRootManager()}")
          },
        )
      }
    }

    ShikenNavigationBar(selectedTab = selectedTab, onTabSelect = { selectedTab = it })
  }
}
