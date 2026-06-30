package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.Context
import android.net.Uri
import android.os.SystemClock
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

    @Volatile
    private var lastSuccessfulReportAtMs: Long = 0L
    private val sentAssetIds = mutableSetOf<String>()
    private var executor: ScheduledExecutorService? = null

    @Volatile
    private var sessionConfig: ReporterSessionConfig? = null

    @Volatile
    private var sessionContext: Context? = null

    @Volatile
    private var shouldReconnect = false

    @Volatile
    private var reconnectScheduled = false
    private val reconnectStrategy = ReconnectStrategy()

    /**
     * 后台采集到的快照通过回写 UI 状态，实现"主动自动更新"，
     * 而不是依赖用户手按按钮。
     */
    @Volatile
    var onSnapshotCollected: ((DeviceSnapshot) -> Unit)? = null

    @Volatile
    var onConnectionChanged: ((Boolean) -> Unit)? = null

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

        val applicationContext = context.applicationContext
        val config = ReporterSessionConfig(
            serverUrl = persistedConfig.serverUrl,
            apiKey = persistedConfig.apiKey,
            intervalMs = persistedConfig.intervalMs,
        )

        val activeExecutor = executor
        if (
            shouldReconnect &&
            sessionConfig == config &&
            activeExecutor != null &&
            !activeExecutor.isShutdown
        ) {
            sessionContext = applicationContext
            websocket ?: connect(applicationContext, config)
            return
        }

        stop()
        sessionContext = applicationContext
        sessionConfig = config
        shouldReconnect = true
        reconnectScheduled = false
        reconnectStrategy.reset()
        val collector = DeviceSnapshotCollector(applicationContext)
        executor = Executors.newSingleThreadScheduledExecutor().also { scheduledExecutor ->
            scheduledExecutor.scheduleAtFixedRate(
                {
                    runCatching {
                        val socket = websocket ?: connect(applicationContext, config)
                        val snapshot = collector.collectSnapshot()
                        val stateKey = snapshot.stableStateKey()
                        val now = SystemClock.elapsedRealtime()
                        onSnapshotCollected?.invoke(snapshot)

                        // 黑屏后状态常常不变，但服务端仍需要周期性收到当前快照来证明链路存活。
                        val reportStale = now - lastSuccessfulReportAtMs >= config.maxSilentReportMs
                        if ((stateKey != lastReportedStateKey || reportStale) && socket != null) {
                            val sent = sendSnapshot(socket, snapshot, DeviceIdentity.deviceId(applicationContext))
                            if (sent) {
                                lastReportedStateKey = stateKey
                                lastSuccessfulReportAtMs = now
                            } else {
                                if (websocket == socket) {
                                    websocket = null
                                }
                                lastReportedStateKey = null
                                scheduleReconnect()
                            }
                        }
                    }
                },
                0,
                config.intervalMs,
                TimeUnit.MILLISECONDS,
            )
        }
        connect(applicationContext, config)
    }

    fun stop() {
        shouldReconnect = false
        reconnectScheduled = false
        executor?.shutdownNow()
        executor = null
        websocket?.close(1000, "stopped")
        websocket = null
        onConnectionChanged?.invoke(false)
        lastReportedStateKey = null
        lastSuccessfulReportAtMs = 0L
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

    /**
     * 健康检查：在系统从 Doze/休眠中唤醒我们时调用。
     * - 若 WS 已断开 (null 或失败的输出缓冲持续堆积) 则立即重连；
     * - 否则保持现状，OkHttp 的 pingInterval 仍在维持 keepalive。
     */
    fun healthCheck() {
        if (!shouldReconnect) return
        val cfg = sessionConfig ?: return
        val ctx = sessionContext ?: return
        val current = websocket
        // null 已断；queueSize 持续堆积意味着对端不读——半开，强制重连
        val needsReconnect = current == null || current.queueSize() > 256 * 1024L
        if (needsReconnect) {
            current?.close(1001, "health-check")
            websocket = null
            reconnectScheduled = false
            reconnectStrategy.reset()
            connect(ctx, cfg)
        }
    }

    fun reportSnapshot(context: Context, snapshot: DeviceSnapshot): Boolean {
        val cfg = sessionConfig ?: MobileReporterConfigStore.read(context).let { persistedConfig ->
            if (!persistedConfig.autoReport || persistedConfig.apiKey.isBlank()) return false
            ReporterSessionConfig(
                serverUrl = persistedConfig.serverUrl,
                apiKey = persistedConfig.apiKey,
                intervalMs = persistedConfig.intervalMs,
            )
        }
        val ctx = sessionContext ?: context.applicationContext
        val socket = websocket ?: connect(ctx, cfg) ?: return false
        val sent = sendSnapshot(socket, snapshot, DeviceIdentity.deviceId(ctx))
        if (!sent && websocket == socket) {
            websocket = null
            lastReportedStateKey = null
            scheduleReconnect()
        }
        return sent
    }

    private fun connect(context: Context, config: ReporterSessionConfig): WebSocket? {
        val deviceId = DeviceIdentity.deviceId(context)
        val authedUrl = Uri.parse(config.serverUrl).buildUpon()
            .appendQueryParameter("key", config.apiKey)
            .appendQueryParameter("client", "android-service")
            .appendQueryParameter("deviceId", deviceId)
            .build()
            .toString()
        val request = Request.Builder().url(authedUrl).build()
        websocket = client.newWebSocket(
            request,
            object : WebSocketListener() {
                override fun onOpen(webSocket: WebSocket, response: Response) {
                    reconnectScheduled = false
                    reconnectStrategy.reset()
                    lastReportedStateKey = null
                    onConnectionChanged?.invoke(true)
                }

                override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
                    webSocket.close(1000, null)
                }

                override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                    if (websocket == webSocket) {
                        websocket = null
                        lastReportedStateKey = null
                        onConnectionChanged?.invoke(false)
                    }
                    scheduleReconnect()
                }

                override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                    if (websocket == webSocket) {
                        websocket = null
                        lastReportedStateKey = null
                        onConnectionChanged?.invoke(false)
                    }
                    scheduleReconnect()
                }
            },
        )
        return websocket
    }

    private fun scheduleReconnect() {
        if (!shouldReconnect) return
        if (reconnectScheduled) return
        val ctx = sessionContext ?: return
        val cfg = sessionConfig ?: return
        val scheduledExecutor = executor ?: return
        val delay = reconnectStrategy.nextDelayMs()
        if (delay < 0) return
        reconnectScheduled = true
        scheduledExecutor.schedule({
            reconnectScheduled = false
            if (shouldReconnect && websocket == null) {
                connect(ctx, cfg)
            }
        }, delay, TimeUnit.MILLISECONDS)
    }

    private fun sendSnapshot(socket: WebSocket, snapshot: DeviceSnapshot, deviceId: String): Boolean {
        snapshot.assets.filterNot { asset -> asset.id in sentAssetIds }.forEach { asset ->
            val metaSent = socket.send(
                JSONObject()
                    .put("type", "upload_artwork_meta")
                    .put("content_item_identifier", asset.id)
                    .put("mime_type", asset.mimeType)
                    .toString(),
            )
            val assetSent = socket.send(asset.bytes.toByteString())
            if (!metaSent || !assetSent) {
                return false
            }
            sentAssetIds.add(asset.id)
        }

        return socket.send(
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
    ) {
        val maxSilentReportMs: Long = maxOf(intervalMs * 3, 60_000L)
    }
}
