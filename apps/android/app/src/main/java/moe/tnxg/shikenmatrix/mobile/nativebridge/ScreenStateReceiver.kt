package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build

/**
 * 熄屏 / 亮屏的唯一监听点。
 *
 * 熄屏与亮屏仅触发新快照，不改变连接状态。监听器只在进程存活时注册，
 * 后台运行由用户启用的前台服务负责。
 */
object ScreenStateReceiver {
    private var receiver: BroadcastReceiver? = null

    fun register(context: Context, onScreenOff: () -> Unit, onScreenOn: () -> Unit) {
        if (receiver != null) return

        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_SCREEN_OFF)
            addAction(Intent.ACTION_SCREEN_ON)
        }
        val broadcastReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                when (intent.action) {
                    Intent.ACTION_SCREEN_OFF -> onScreenOff()
                    Intent.ACTION_SCREEN_ON -> onScreenOn()
                }
            }
        }
        val applicationContext = context.applicationContext
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            applicationContext.registerReceiver(
                broadcastReceiver,
                filter,
                Context.RECEIVER_NOT_EXPORTED,
            )
        } else {
            @Suppress("UnspecifiedRegisterReceiverFlag")
            applicationContext.registerReceiver(broadcastReceiver, filter)
        }
        receiver = broadcastReceiver
    }

    fun unregister(context: Context) {
        val broadcastReceiver = receiver ?: return
        runCatching { context.applicationContext.unregisterReceiver(broadcastReceiver) }
        receiver = null
    }
}
