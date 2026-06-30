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

  /**
   * Doze 维护窗口通常以 1, 2, 4, 8 分钟递增展开。
   * 设为 4 分钟能在不频繁打扰用户的前提下保证后台最快在 ~8 分钟内被唤醒，
   * 同时避免触发系统对"高频精确闹钟"的限制。
   */
  private const val WATCHDOG_INTERVAL_MS = 4 * 60 * 1000L

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
    // 每次被系统唤醒后重新排一次，让闹钟持续在 Doze 维护窗口触发
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

  /**
   * 一次性排程，在 Doze 模式下也尽量在最近一次维护窗口触发。
   * 不用 setInexactRepeating——它在 Doze 下完全不触发。
   */
  private fun scheduleWatchdog(context: Context) {
    val alarmManager = context.getSystemService(AlarmManager::class.java)
    val triggerAt = SystemClock.elapsedRealtime() + WATCHDOG_INTERVAL_MS
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      // Android 12 (API 31) 起需要 SCHEDULE_EXACT_ALARM，setAndAllowWhileIdle 不受此限。
      alarmManager.setAndAllowWhileIdle(
        AlarmManager.ELAPSED_REALTIME_WAKEUP,
        triggerAt,
        watchdogIntent(context),
      )
    } else {
      alarmManager.setExact(
        AlarmManager.ELAPSED_REALTIME_WAKEUP,
        triggerAt,
        watchdogIntent(context),
      )
    }
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