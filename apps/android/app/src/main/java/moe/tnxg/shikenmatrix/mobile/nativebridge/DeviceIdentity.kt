package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.Context
import android.os.Build
import android.provider.Settings
import java.security.MessageDigest
import java.util.UUID

/**
 * 生成并持久化一个稳定且唯一的设备标识。
 *
 * 优先使用 ANDROID_ID —— 在未恢复出厂设置时它在同一设备+同一签名/用户区间内稳定；
 * 当 ANDROID_ID 不可靠/为空时回退到安装期生成的随机 UUID。
 * 之后将该值持久化到私有 SharedPreferences，确保 APP 重装/升级仍使用同一标识。
 */
object DeviceIdentity {
  private const val PREFS = "shikenmatrix_device_identity"
  private const val KEY_DEVICE_ID = "device_id"

  @Volatile
  private var cached: String? = null

  fun deviceId(context: Context): String {
    cached?.let { return it }

    val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
    prefs.getString(KEY_DEVICE_ID, null)?.let { id ->
      cached = id
      return id
    }

    val androidId = runCatching {
      Settings.Secure.getString(context.contentResolver, Settings.Secure.ANDROID_ID)
    }.getOrNull().orEmpty().trim()

    val base = if (androidId.isNotBlank() && !androidId.equals("9774d56d682e549c", ignoreCase = true)) {
      // 把 ANDROID_ID + 厂商/型号 做 hash 以避免直接泄露 ANDROID_ID 原值
      fingerprint(androidId + "|" + Build.MANUFACTURER + "|" + Build.MODEL)
    } else {
      // 退化为安装期随机 UUID，仍稳定持久
      UUID.randomUUID().toString().replace("-", "")
    }

    val id = "${fingerprint(Build.MANUFACTURER + "|" + Build.MODEL).take(8)}-$base"
    prefs.edit().putString(KEY_DEVICE_ID, id).apply()
    cached = id
    return id
  }

  private fun fingerprint(value: String): String =
    MessageDigest.getInstance("SHA-256")
      .digest(value.toByteArray(Charsets.UTF_8))
      .joinToString("") { byte -> "%02x".format(byte) }
}