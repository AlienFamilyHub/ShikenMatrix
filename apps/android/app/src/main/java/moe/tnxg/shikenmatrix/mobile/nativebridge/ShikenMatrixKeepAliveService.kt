package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.annotation.SuppressLint
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import moe.tnxg.shikenmatrix.mobile.R

class ShikenMatrixKeepAliveService : Service() {
    private var wakeLock: PowerManager.WakeLock? = null

    override fun onCreate() {
        super.onCreate()
        startForeground(NOTIFICATION_ID, notification())
        acquireWakeLock()
        KeepAliveController.refreshWatchdogIfEnabled(this)
        BackgroundReporter.start(this)
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        KeepAliveController.refreshWatchdogIfEnabled(this)
        BackgroundReporter.start(this)
        // 进程被系统从 Doze 唤醒拉起 —— 主动做一次健康检查
        BackgroundReporter.healthCheck()
        return START_STICKY
    }

    override fun onDestroy() {
        releaseWakeLock()
        BackgroundReporter.stop()
        super.onDestroy()
    }

    @SuppressLint("WakelockTimeout")
    private fun acquireWakeLock() {
        if (wakeLock?.isHeld == true) return

        wakeLock = getSystemService(PowerManager::class.java)
            .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "$packageName:ShikenMatrixKeepAlive")
            .apply {
                setReferenceCounted(false)
                acquire()
            }
    }

    private fun releaseWakeLock() {
        wakeLock?.takeIf { it.isHeld }?.release()
        wakeLock = null
    }

    private fun notification(): Notification {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "ShikenMatrix Keep Alive",
                NotificationManager.IMPORTANCE_LOW,
            )
            getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
        }

        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
        } else {
            Notification.Builder(this)
        }

        return builder
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentTitle("ShikenMatrix 正在保活")
            .setContentText("正在采集前台程序、媒体、电量、网络与粗略位置状态")
            .setOngoing(true)
            .build()
    }

    companion object {
        private const val CHANNEL_ID = "shikenmatrix_keep_alive"
        private const val NOTIFICATION_ID = 34950
    }
}
