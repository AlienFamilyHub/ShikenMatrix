package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

class ShikenMatrixBootReceiver : BroadcastReceiver() {
  override fun onReceive(context: Context, intent: Intent) {
    when (intent.action) {
      Intent.ACTION_BOOT_COMPLETED,
      Intent.ACTION_LOCKED_BOOT_COMPLETED,
      Intent.ACTION_MY_PACKAGE_REPLACED,
      KeepAliveController.ACTION_WATCHDOG,
      -> KeepAliveController.handleSystemWakeup(context)
    }
  }
}
