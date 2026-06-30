package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.Context
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import org.json.JSONObject
import okio.ByteString.Companion.toByteString
import java.util.concurrent.Executors
import java.util.concurrent.ScheduledExecutorService
import java.util.concurrent.TimeUnit

object BackgroundReporter {
  private val client = OkHttpClient.Builder()
    .retryOnConnectionFailure(true)
    .pingInterval(20, TimeUnit.SECONDS) // WS 心跳，防 NAT/代理断连
    .connectTimeout(15, TimeUnit.SECONDS)
    .readTimeout(0, TimeUnit.SECONDS)
    .callTimeout(0, TimeUnit.SECONDS)
    .build()

  @Volatile
  private var websocket: WebSocket? = null
  @Volatile
  private var lastReportedStateKey: String? = null
  private val sentAssetIds = mutableSetOf<String>()
  private var executor: ScheduledExecutorService? = null

  @Volatile
  private var sessionConfig: ReporterSessionConfig? = null
  @Volatile
  private var sessionContext: Context? = null
  @Volatile
  private var shouldReconnect = false
  private val reconnectStrategy = ReconnectStrategy()

  /**
   * 后台采集到的快照通过回写 UI 状态，实现"主动自动更新"，
   * 而不是依赖用户手按按钮。
   */
  @Volatile
  var onSnapshotCollected: ((DeviceSnapshot) -> Unit)? = null

  fun configure(
    context: Context,
    serverUrl: String,
    apiKey: String,
    intervalMs: Long,
    enabled: Boolean,
  ) {
    MobileReporterConfigStore.save(
      context,
      MobileReporterConfig(
        serverUrl = serverUrl,
        apiKey = apiKey,
        intervalMs = intervalMs,
        autoReport = enabled,
      ),
    )

    if (enabled) {
      start(context)
    } else {
      stop()
    }
  }

  fun start(context: Context) {
    val persistedConfig = MobileReporterConfigStore.read(context)
    if (!persistedConfig.autoReport || persistedConfig.apiKey.isBlank()) return

    val config = ReporterSessionConfig(
      serverUrl = persistedConfig.serverUrl,
      apiKey = persistedConfig.apiKey,
      intervalMs = persistedConfig.intervalMs,
    )

    stop()
    sessionContext = context.applicationContext
    sessionConfig = config
    shouldReconnect = true
    reconnectStrategy.reset()
    connect(context.applicationContext, config)
    val collector = DeviceSnapshotCollector(context.applicationContext)
    executor = Executors.newSingleThreadScheduledExecutor().also { scheduledExecutor ->
      scheduledExecutor.scheduleAtFixedRate(
        {
          runCatching {
            val socket = websocket ?: connect(context.applicationContext, config)
            val snapshot = collector.collectSnapshot()
            val stateKey = snapshot.stableStateKey()
            onSnapshotCollected?.invoke(snapshot)
            if (stateKey != lastReportedStateKey && socket != null) {
              sendSnapshot(socket, snapshot, DeviceIdentity.deviceId(context.applicationContext))
              lastReportedStateKey = stateKey
            }
          }
        },
        0,
        config.intervalMs,
        TimeUnit.MILLISECONDS,
      )
    }
  }

  fun stop() {
    shouldReconnect = false
    executor?.shutdownNow()
    executor = null
    websocket?.close(1000, "stopped")
    websocket = null
    lastReportedStateKey = null
    sentAssetIds.clear()
    sessionConfig = null
    sessionContext = null
  }

  fun disable(context: Context) {
    MobileReporterConfigStore.setAutoReport(context, false)
    stop()
  }

  fun isEnabled(context: Context): Boolean =
    MobileReporterConfigStore.read(context).autoReport

  fun startKeepAliveServiceIfEnabled(context: Context) {
    KeepAliveController.startServiceIfEnabled(context)
  }

  private fun connect(context: Context, config: ReporterSessionConfig): WebSocket? {
    val request = Request.Builder().url(config.serverUrl).build()
    val deviceId = DeviceIdentity.deviceId(context)
    websocket = client.newWebSocket(
      request,
      object : WebSocketListener() {
        override fun onOpen(webSocket: WebSocket, response: Response) {
          webSocket.send(
            JSONObject()
              .put("type", "mobile_hello")
              .put("client", "android-service")
              .put("deviceId", deviceId)
              .put("keyId", config.apiKey)
              .toString(),
          )
          reconnectStrategy.reset()
        }

        override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
          webSocket.close(1000, null)
        }

        override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
          if (websocket == webSocket) {
            websocket = null
          }
          scheduleReconnect()
        }

        override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
          if (websocket == webSocket) {
            websocket = null
          }
          scheduleReconnect()
        }
      },
    )
    return websocket
  }

  private fun scheduleReconnect() {
    if (!shouldReconnect) return
    val ctx = sessionContext ?: return
    val cfg = sessionConfig ?: return
    val delay = reconnectStrategy.nextDelayMs()
    if (delay < 0) return
    executor?.schedule({
      if (shouldReconnect && websocket == null) {
        connect(ctx, cfg)
      }
    }, delay, TimeUnit.MILLISECONDS)
  }

  private fun sendSnapshot(socket: WebSocket, snapshot: DeviceSnapshot, deviceId: String) {
    snapshot.assets.filterNot { asset -> asset.id in sentAssetIds }.forEach { asset ->
      socket.send(
        JSONObject()
          .put("type", "upload_artwork_meta")
          .put("content_item_identifier", asset.id)
          .put("mime_type", asset.mimeType)
          .toString(),
      )
      socket.send(asset.bytes.toByteString())
      sentAssetIds.add(asset.id)
    }

    socket.send(
      JSONObject()
        .put("type", "android_snapshot")
        .put("deviceId", deviceId)
        .put("snapshot", snapshot.json)
        .toString(),
    )
  }

  private data class ReporterSessionConfig(
    val serverUrl: String,
    val apiKey: String,
    val intervalMs: Long,
  )
}
