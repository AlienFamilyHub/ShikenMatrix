package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.PowerManager

class ShikenMatrixBootReceiver : BroadcastReceiver() {
  override fun onReceive(context: Context, intent: Intent) {
    when (intent.action) {
      Intent.ACTION_BOOT_COMPLETED,
      Intent.ACTION_LOCKED_BOOT_COMPLETED,
      Intent.ACTION_MY_PACKAGE_REPLACED,
      KeepAliveController.ACTION_WATCHDOG,
      -> handleWakeup(context)
    }
  }

  private fun handleWakeup(context: Context) {
    if (!KeepAliveController.isEnabled(context)) return

    // 系统从 Doze 维护窗口把我们唤起 —— 拿 PARTIAL_WAKE_LOCK 几秒，
    // 保证在 broadcast 期间 CPU 不睡，能完成 UI/连接重建握手。
    val pm = context.getSystemService(Context.POWER_SERVICE) as PowerManager
    val wakeLock = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "shikenmatrix:watchdog").apply {
      setReferenceCounted(false)
      acquire(10_000L)
    }

    try {
      KeepAliveController.handleSystemWakeup(context)
      BackgroundReporter.healthCheck()
    } finally {
      if (wakeLock.isHeld) {
        runCatching { wakeLock.release() }
      }
    }
  }
}