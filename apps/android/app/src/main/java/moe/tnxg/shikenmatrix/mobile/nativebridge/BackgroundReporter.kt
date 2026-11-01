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
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.ScheduledExecutorService
import java.util.concurrent.TimeUnit

/**
 * 上报会话。
 *
 * 自动上报与屏幕状态相互独立；熄屏继续采集，只有连接断开才离线。
 * 前台服务负责后台运行；不持有 wakelock，不使用保活闹钟。
 */
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
    private val confirmedCachedAssetIds = ConcurrentHashMap.newKeySet<String>()
    private val pendingAssetProbes = ConcurrentHashMap<String, AssetCacheProbe>()
    @Volatile
    private var executor: ScheduledExecutorService? = null

    private val sendLock = Any()

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
    @Volatile
    var isConnected: Boolean = false
        private set

    private fun publishConnection(connected: Boolean) {
        isConnected = connected
        onConnectionChanged?.invoke(connected)
    }

    fun configure(
        context: Context,
        serverUrl: String,
        apiKey: String,
        intervalMs: Long,
        enabled: Boolean,
    ) {
        MobileReporterConfigStore.save(
            context,
            MobileReporterConfigStore.read(context).copy(
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

    @Synchronized
    fun start(context: Context) {
        val persistedConfig = MobileReporterConfigStore.read(context)
        if (!persistedConfig.autoReport || persistedConfig.apiKey.isBlank()) {
            stop()
            return
        }

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
        ScreenStateReceiver.register(
            applicationContext,
            onScreenOff = { requestReport() },
            onScreenOn = { handleScreenOn(applicationContext) },
        )

        executor = Executors.newSingleThreadScheduledExecutor().also { scheduledExecutor ->
            val collector = DeviceSnapshotCollector(applicationContext)
            scheduledExecutor.scheduleAtFixedRate(
                {
                    if (executor === scheduledExecutor) {
                        collectAndReport(applicationContext, config, collector, force = false)
                    }
                },
                config.intervalMs,
                config.intervalMs,
                TimeUnit.MILLISECONDS,
            )
        }
        connect(applicationContext, config)
    }

    /** Both periodic collection and screen transitions run off the Android main thread. */
    private fun collectAndReport(
        context: Context,
        config: ReporterSessionConfig,
        collector: DeviceSnapshotCollector,
        force: Boolean,
    ) {
        if (sessionConfig != config || !shouldReconnect) return
        runCatching {
            val socket = websocket ?: connect(context, config) ?: return
            if (!isConnected) return
            val snapshot = collector.collectSnapshot()
            if (sessionConfig != config || !shouldReconnect) return
            onSnapshotCollected?.invoke(snapshot)
            val stateKey = snapshot.stableStateKey()
            val now = SystemClock.elapsedRealtime()
            val stale = now - lastSuccessfulReportAtMs >= config.maxSilentReportMs
            if (force || stale || stateKey != lastReportedStateKey) {
                if (sendSnapshot(socket, snapshot, DeviceIdentity.deviceId(context))) {
                    lastReportedStateKey = stateKey
                    lastSuccessfulReportAtMs = now
                } else if (websocket === socket) {
                    socket.cancel()
                    connectionEnded(socket)
                }
            }
        }
    }

    @Synchronized
    fun requestReport() {
        val context = sessionContext ?: return
        val config = sessionConfig ?: return
        val worker = executor ?: return
        if (worker.isShutdown || immediateReportQueued) return
        immediateReportQueued = true
        worker.execute {
            if (executor === worker) {
                immediateReportQueued = false
                collectAndReport(context, config, DeviceSnapshotCollector(context), force = true)
            }
        }
    }

    @Synchronized
    fun stop() {
        shouldReconnect = false
        reconnectScheduled = false
        executor?.shutdownNow()
        executor = null
        websocket?.close(1000, "stopped")
        websocket = null
        sessionContext?.let { ScreenStateReceiver.unregister(it) }
        publishConnection(false)
        lastReportedStateKey = null
        lastSuccessfulReportAtMs = 0L
        confirmedCachedAssetIds.clear()
        pendingAssetProbes.clear()
        immediateReportQueued = false
        sessionConfig = null
        sessionContext = null
    }

    fun disable(context: Context) {
        MobileReporterConfigStore.setAutoReport(context, false)
        stop()
    }

    fun isEnabled(context: Context): Boolean =
        MobileReporterConfigStore.read(context).autoReport

    @Volatile
    private var immediateReportQueued = false

    fun handleScreenOn(context: Context) {
        // 不依赖进程内的旧标记：Activity 恢复也能修复漏掉的亮屏广播。
        start(context.applicationContext)
        requestReport()
    }

    fun reportSnapshot(context: Context, snapshot: DeviceSnapshot): Boolean {
        if (!shouldReconnect) return false
        val cfg = sessionConfig ?: return false
        val ctx = sessionContext ?: context.applicationContext
        val socket = websocket ?: connect(ctx, cfg) ?: return false
        val sent = sendSnapshot(socket, snapshot, DeviceIdentity.deviceId(ctx))
        if (!sent && websocket == socket) {
            socket.cancel()
            connectionEnded(socket)
        }
        return sent
    }

    @Synchronized
    private fun connect(context: Context, config: ReporterSessionConfig): WebSocket? {
        if (!shouldReconnect) return null
        websocket?.let { return it }
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
                    synchronized(this@BackgroundReporter) {
                        if (websocket !== webSocket || !shouldReconnect) {
                            webSocket.close(1000, "superseded")
                            return
                        }
                        reconnectScheduled = false
                        reconnectStrategy.reset()
                        lastReportedStateKey = null
                        confirmedCachedAssetIds.clear()
                        publishConnection(true)
                        requestReport()
                    }
                }

                override fun onMessage(webSocket: WebSocket, text: String) {
                    if (websocket !== webSocket) return
                    val payload = runCatching { JSONObject(text) }.getOrNull() ?: return
                    when (payload.optString("type")) {
                        "asset_cache_status" -> {
                            val assetId = payload.optString("content_item_identifier")
                            if (assetId.isBlank()) return
                            pendingAssetProbes.remove(assetId)?.complete(payload.optBoolean("cached"))
                        }
                    }
                }

                override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
                    webSocket.close(1000, null)
                }

                override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                    connectionEnded(webSocket)
                }

                override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                    connectionEnded(webSocket)
                }
            },
        )
        return websocket
    }

    @Synchronized
    private fun connectionEnded(socket: WebSocket) {
        if (websocket !== socket) return
        websocket = null
        lastReportedStateKey = null
        publishConnection(false)
        scheduleReconnect()
    }

    @Synchronized
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
            if (executor === scheduledExecutor && shouldReconnect && websocket == null) {
                connect(ctx, cfg)
            }
        }, delay, TimeUnit.MILLISECONDS)
    }

    private fun sendSnapshot(
        socket: WebSocket,
        snapshot: DeviceSnapshot,
        deviceId: String,
    ): Boolean = synchronized(sendLock) {
        if (socket !== websocket || !shouldReconnect) return false
        snapshot.assets.forEach { asset ->
            if (asset.id in confirmedCachedAssetIds) return@forEach
            val cacheProbe = AssetCacheProbe()
            pendingAssetProbes[asset.id] = cacheProbe
            val querySent = socket.send(
                JSONObject()
                    .put("type", "asset_cache_query")
                    .put("content_item_identifier", asset.id)
                    .put("mime_type", asset.mimeType)
                    .toString(),
            )
            if (!querySent) {
                pendingAssetProbes.remove(asset.id)
                return false
            }

            // 必须等服务端确认 miss 后再传二进制，否则服务端无法把 binary 绑定到正确资源。
            if (!cacheProbe.await()) {
                pendingAssetProbes.remove(asset.id)
                return false
            }

            if (!cacheProbe.cached) {
                val assetSent = socket.send(asset.bytes.toByteString())
                if (!assetSent) {
                    return false
                }
            }
            confirmedCachedAssetIds.add(asset.id)
        }

        if (socket !== websocket || !shouldReconnect) return false
        socket.send(
            JSONObject()
                .put("type", "android_snapshot")
                .put("deviceId", deviceId)
                .put("snapshot", snapshot.json)
                .toString(),
        )
    }


    private class AssetCacheProbe {
        private val latch = CountDownLatch(1)

        @Volatile
        var cached: Boolean = false
            private set

        fun complete(value: Boolean) {
            cached = value
            latch.countDown()
        }

        fun await(): Boolean = latch.await(2, TimeUnit.SECONDS)
    }

    private data class ReporterSessionConfig(
        val serverUrl: String,
        val apiKey: String,
        val intervalMs: Long,
    ) {
        val maxSilentReportMs: Long = maxOf(intervalMs * 3, 60_000L)
    }
}
