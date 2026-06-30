package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.app.AlarmManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.SystemClock

object KeepAliveController {
  private const val PREFS = "shikenmatrix_keep_alive"
  private const val KEY_ENABLED = "enabled"
  private const val WATCHDOG_INTERVAL_MS = 60_000L

  const val ACTION_WATCHDOG = "moe.tnxg.shikenmatrix.mobile.action.WATCHDOG"

  fun enable(context: Context) {
    context.keepAlivePrefs()
      .edit()
      .putBoolean(KEY_ENABLED, true)
      .apply()

    startServiceIfEnabled(context)
    scheduleWatchdog(context)
  }

  fun disable(context: Context) {
    context.keepAlivePrefs()
      .edit()
      .putBoolean(KEY_ENABLED, false)
      .apply()

    cancelWatchdog(context)
    context.stopService(Intent(context, ShikenMatrixKeepAliveService::class.java))
  }

  fun isEnabled(context: Context): Boolean =
    context.keepAlivePrefs().getBoolean(KEY_ENABLED, false)

  fun handleSystemWakeup(context: Context) {
    if (!isEnabled(context)) return

    startServiceIfEnabled(context)
    scheduleWatchdog(context)
  }

  fun refreshWatchdogIfEnabled(context: Context) {
    if (!isEnabled(context)) return

    scheduleWatchdog(context)
  }

  fun startServiceIfEnabled(context: Context): Boolean {
    if (!isEnabled(context)) return false

    return runCatching {
      val intent = Intent(context, ShikenMatrixKeepAliveService::class.java)
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
        context.startForegroundService(intent)
      } else {
        context.startService(intent)
      }
      true
    }.getOrElse {
      false
    }
  }

  private fun scheduleWatchdog(context: Context) {
    val alarmManager = context.getSystemService(AlarmManager::class.java)
    alarmManager.setInexactRepeating(
      AlarmManager.ELAPSED_REALTIME_WAKEUP,
      SystemClock.elapsedRealtime() + WATCHDOG_INTERVAL_MS,
      WATCHDOG_INTERVAL_MS,
      watchdogIntent(context),
    )
  }

  private fun cancelWatchdog(context: Context) {
    context.getSystemService(AlarmManager::class.java).cancel(watchdogIntent(context))
  }

  private fun watchdogIntent(context: Context): PendingIntent =
    PendingIntent.getBroadcast(
      context,
      0,
      Intent(context, ShikenMatrixBootReceiver::class.java).setAction(ACTION_WATCHDOG),
      PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )

  private fun Context.keepAlivePrefs() =
    getSharedPreferences(PREFS, Context.MODE_PRIVATE)
}
