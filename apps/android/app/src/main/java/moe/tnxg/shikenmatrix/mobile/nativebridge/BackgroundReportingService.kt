package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.Manifest
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.content.pm.PackageManager
import android.content.pm.ServiceInfo
import android.location.LocationManager
import android.os.Build
import android.os.IBinder
import moe.tnxg.shikenmatrix.mobile.MainActivity

/** User-enabled continuous device-presence monitoring. */
class BackgroundReportingService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val config = MobileReporterConfigStore.read(this)
        if (!config.autoReport || config.apiKey.isBlank() || BackgroundReportingController.isServiceSuspended(this)) {
            stopSelf()
            BackgroundReporter.start(applicationContext)
            return START_NOT_STICKY
        }
        try {
            val notification = buildNotification()
            var locationRestricted = false
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                val monitoringType = ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE
                val locationEnabled = (getSystemService(LOCATION_SERVICE) as LocationManager).isLocationEnabled
                val locationGranted = checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) == PackageManager.PERMISSION_GRANTED
                // Location is optional: background/while-in-use restrictions must not block media reporting.
                if (locationGranted && locationEnabled) {
                    try {
                        startForeground(NOTIFICATION_ID, notification, monitoringType or ServiceInfo.FOREGROUND_SERVICE_TYPE_LOCATION)
                    } catch (_: SecurityException) {
                        locationRestricted = true
                        startForeground(NOTIFICATION_ID, notification, monitoringType)
                    }
                } else {
                    startForeground(NOTIFICATION_ID, notification, monitoringType)
                }
            } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val locationGranted = checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) == PackageManager.PERMISSION_GRANTED
                startForeground(NOTIFICATION_ID, notification, if (locationGranted) ServiceInfo.FOREGROUND_SERVICE_TYPE_LOCATION else 0)
            } else {
                startForeground(NOTIFICATION_ID, notification)
            }
            BackgroundReportingController.serviceState(true, if (locationRestricted) {
                "后台服务运行中；位置前台权限受系统限制，歌曲、电量和网络继续更新"
            } else "后台服务运行中；熄屏继续更新真实状态")
            BackgroundReporter.start(applicationContext)
        } catch (error: Exception) {
            BackgroundReportingController.suspendBackgroundReporting(applicationContext, "后台服务启动失败：${error.message}；打开应用可重试")
            stopSelf()
            return START_NOT_STICKY
        }
        return START_STICKY
    }

    override fun onDestroy() {
        if (BackgroundReportingController.serviceRunning) {
            BackgroundReportingController.serviceState(false, "后台服务已停止；采集连接仍按上报开关重新评估")
        }
        BackgroundReporter.start(applicationContext)
        super.onDestroy()
    }

    override fun onTimeout(startId: Int, fgsType: Int) {
        // Durable suspension prevents a restart loop after an OS service limit.
        BackgroundReportingController.suspendBackgroundReporting(applicationContext, "系统前台服务时限已到；已暂停服务，回到应用后可重试")
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    private fun buildNotification(): Notification {
        val manager = getSystemService(NOTIFICATION_SERVICE) as NotificationManager
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            manager.createNotificationChannel(NotificationChannel(CHANNEL_ID, "设备状态后台上报", NotificationManager.IMPORTANCE_LOW).apply {
                description = "用户主动开启的设备状态、媒体与电量持续采集上报"
                setShowBadge(false)
            })
        }
        val openApp = PendingIntent.getActivity(this, 0, Intent(this, MainActivity::class.java), PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) Notification.Builder(this, CHANNEL_ID)
            else Notification.Builder(this)
        return builder
            .setSmallIcon(android.R.drawable.stat_notify_sync)
            .setContentTitle("设备状态后台上报已开启")
            .setContentText("熄屏继续更新歌曲、电量与网络")
            .setContentIntent(openApp)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .build()
    }

    private companion object {
        const val CHANNEL_ID = "background_reporting"
        const val NOTIFICATION_ID = 4317
    }
}
