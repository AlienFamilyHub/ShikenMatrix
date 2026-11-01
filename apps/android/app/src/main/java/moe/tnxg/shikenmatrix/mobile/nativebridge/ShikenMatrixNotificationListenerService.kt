package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.ComponentName
import android.media.MediaMetadata
import android.media.session.MediaController
import android.media.session.MediaSession
import android.media.session.MediaSessionManager
import android.media.session.PlaybackState
import android.os.Handler
import android.os.Looper
import android.service.notification.NotificationListenerService

class ShikenMatrixNotificationListenerService : NotificationListenerService() {
    private val handler = Handler(Looper.getMainLooper())
    private val controllers = mutableMapOf<MediaSession.Token, ObservedController>()
    private var sessionManager: MediaSessionManager? = null
    private val sessionsChanged = MediaSessionManager.OnActiveSessionsChangedListener {
        updateControllers(it.orEmpty())
    }

    override fun onListenerConnected() {
        super.onListenerConnected()
        clearControllers()
        val manager = getSystemService(MediaSessionManager::class.java) ?: return
        sessionManager = manager
        runCatching {
            val component = ComponentName(this, javaClass)
            manager.addOnActiveSessionsChangedListener(sessionsChanged, component, handler)
            updateControllers(manager.getActiveSessions(component))
        }
        BackgroundReportingController.sync(applicationContext)
        BackgroundReporter.start(applicationContext)
        BackgroundReporter.requestReport()
    }

    override fun onListenerDisconnected() {
        clearControllers()
        BackgroundReporter.requestReport()
        super.onListenerDisconnected()
    }

    override fun onDestroy() {
        clearControllers()
        super.onDestroy()
    }

    private fun updateControllers(active: List<MediaController>) {
        val iterator = controllers.entries.iterator()
        while (iterator.hasNext()) {
            val entry = iterator.next()
            if (active.none { it.sessionToken == entry.key }) {
                entry.value.controller.unregisterCallback(entry.value.callback)
                iterator.remove()
            }
        }
        active.forEach { controller ->
            if (controller.sessionToken !in controllers) {
                val callback = object : MediaController.Callback() {
                    override fun onMetadataChanged(metadata: MediaMetadata?) {
                        BackgroundReporter.requestReport()
                    }

                    override fun onPlaybackStateChanged(state: PlaybackState?) {
                        BackgroundReporter.requestReport()
                    }

                    override fun onSessionDestroyed() {
                        BackgroundReporter.requestReport()
                    }
                }
                controller.registerCallback(callback, handler)
                controllers[controller.sessionToken] = ObservedController(controller, callback)
            }
        }
        BackgroundReporter.requestReport()
    }

    private fun clearControllers() {
        sessionManager?.let { manager ->
            runCatching { manager.removeOnActiveSessionsChangedListener(sessionsChanged) }
        }
        sessionManager = null
        controllers.values.forEach {
            it.controller.unregisterCallback(it.callback)
        }
        controllers.clear()
    }

    private data class ObservedController(
        val controller: MediaController,
        val callback: MediaController.Callback,
    )
}
