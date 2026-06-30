package moe.tnxg.shikenmatrix.mobile

import android.Manifest
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import moe.tnxg.shikenmatrix.mobile.nativebridge.BackgroundReporter
import moe.tnxg.shikenmatrix.mobile.nativebridge.DeviceIdentity
import moe.tnxg.shikenmatrix.mobile.nativebridge.DeviceSnapshot
import moe.tnxg.shikenmatrix.mobile.nativebridge.DeviceSnapshotCollector
import moe.tnxg.shikenmatrix.mobile.nativebridge.KeepAliveController
import moe.tnxg.shikenmatrix.mobile.nativebridge.MobileReporterConfigStore
import moe.tnxg.shikenmatrix.mobile.nativebridge.ReconnectStrategy
import moe.tnxg.shikenmatrix.mobile.nativebridge.RootManager
import moe.tnxg.shikenmatrix.mobile.ui.ShikenMatrixScreen
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import org.json.JSONObject
import top.yukonga.miuix.kmp.theme.MiuixTheme
import okio.ByteString.Companion.toByteString
import java.util.concurrent.TimeUnit

class MainActivity : ComponentActivity() {
  private val httpClient = OkHttpClient.Builder()
    .retryOnConnectionFailure(true)
    .pingInterval(20, TimeUnit.SECONDS) // 应用层 WS 心跳，防 NAT/代理 60s 空闲断连
    .connectTimeout(15, TimeUnit.SECONDS)
    .readTimeout(0, TimeUnit.SECONDS) // WS 长连接不设读超时
    .callTimeout(0, TimeUnit.SECONDS)
    .build()
  private var websocket: WebSocket? = null
  private var deviceId: String = ""
  private val mainHandler = Handler(Looper.getMainLooper())

  private var connectInvocation: ConnectInvocation? = null
  private val reconnectStrategy = ReconnectStrategy()
  private var shouldReconnect = false

  private data class ConnectInvocation(
    val serverUrl: String,
    val apiKey: String,
    val onConnected: () -> Unit,
    val onDisconnected: () -> Unit,
    val onMessage: (String) -> Unit,
    val onError: (String) -> Unit,
  )

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    deviceId = DeviceIdentity.deviceId(applicationContext)
    enableEdgeToEdge()
    setContent {
      MiuixTheme {
        ShikenMatrixScreen(
          activity = this,
          initialConfig = MobileReporterConfigStore.read(applicationContext),
          saveConfig = { config -> MobileReporterConfigStore.save(applicationContext, config) },
          connectWebSocket = ::connectWebSocket,
          closeWebSocket = ::closeWebSocket,
          collectSnapshot = { DeviceSnapshotCollector(applicationContext).collectSnapshot() },
          sendSnapshot = ::sendSnapshot,
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

  override fun onDestroy() {
    closeWebSocket()
    super.onDestroy()
  }

  private fun connectWebSocket(
    serverUrl: String,
    apiKey: String,
    onConnected: () -> Unit,
    onDisconnected: () -> Unit,
    onMessage: (String) -> Unit,
    onError: (String) -> Unit,
  ) {
    closeWebSocket()
    connectInvocation = ConnectInvocation(
      serverUrl = serverUrl,
      apiKey = apiKey,
      onConnected = onConnected,
      onDisconnected = onDisconnected,
      onMessage = onMessage,
      onError = onError,
    )
    reconnectStrategy.reset()
    shouldReconnect = true
    openSocket(connectInvocation!!)
  }

  private fun openSocket(inv: ConnectInvocation) {
    val request = Request.Builder().url(inv.serverUrl).build()
    websocket = httpClient.newWebSocket(
      request,
      object : WebSocketListener() {
        override fun onOpen(webSocket: WebSocket, response: Response) {
          webSocket.send(
            JSONObject()
              .put("type", "mobile_hello")
              .put("client", "android-compose")
              .put("deviceId", deviceId)
              .put("keyId", inv.apiKey)
              .toString(),
          )
          reconnectStrategy.reset()
          runOnUiThread(inv.onConnected)
        }

        override fun onMessage(webSocket: WebSocket, text: String) {
          runOnUiThread { inv.onMessage(text) }
        }

        override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
          webSocket.close(1000, null)
        }

        override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
          runOnUiThread(inv.onDisconnected)
          scheduleReconnect(inv)
        }

        override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
          runOnUiThread {
            inv.onError(t.message ?: "WebSocket 连接失败")
            inv.onDisconnected()
          }
          scheduleReconnect(inv)
        }
      },
    )
  }

  private fun scheduleReconnect(inv: ConnectInvocation) {
    if (!shouldReconnect) return
    val delay = reconnectStrategy.nextDelayMs()
    if (delay < 0) return
    mainHandler.postDelayed({ if (shouldReconnect) openSocket(inv) }, delay)
  }

  private fun closeWebSocket() {
    shouldReconnect = false
    mainHandler.removeMessages(0)
    websocket?.close(1000, "activity stopped")
    websocket = null
    connectInvocation = null
  }

  private fun startKeepAlive() {
    KeepAliveController.enable(this)
  }

  private fun stopKeepAlive() {
    BackgroundReporter.disable(this)
    KeepAliveController.disable(this)
  }

  private fun sendSnapshot(snapshot: DeviceSnapshot): Boolean {
    val socket = websocket ?: return false
    snapshot.assets.forEach { asset ->
      socket.send(
        JSONObject()
          .put("type", "upload_artwork_meta")
          .put("content_item_identifier", asset.id)
          .put("mime_type", asset.mimeType)
          .toString(),
      )
      socket.send(asset.bytes.toByteString())
    }

    return socket.send(
      JSONObject()
        .put("type", "android_snapshot")
        .put("deviceId", deviceId)
        .put("snapshot", snapshot.json)
        .toString(),
    )
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
