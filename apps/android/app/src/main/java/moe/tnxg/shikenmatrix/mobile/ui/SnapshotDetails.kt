package moe.tnxg.shikenmatrix.mobile.ui

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONObject
import top.yukonga.miuix.kmp.basic.Text as MiuixText
import moe.tnxg.shikenmatrix.mobile.nativebridge.DeviceSnapshot

@Composable
internal fun SnapshotDetailsPanel(snapshot: DeviceSnapshot?) {
  Panel(title = "当前获取的信息") {
    if (snapshot == null) {
      MiuixText(text = "暂无采集结果", color = Color(0xFF646A73), fontSize = 13.sp)
      return@Panel
    }

    val json = snapshot.json
    val foreground = json.optJSONObject("foregroundApp")
    val media = json.optJSONObject("media")
    val battery = json.optJSONObject("battery")
    val network = json.optJSONObject("network")

    InfoGroup(title = "前台应用") {
      InfoRowWithIcon(
        label = "图标",
        iconAsset = foreground?.optJSONObject("appIcon")?.assetBytes(snapshot),
        fallback = foreground.assetLabel("appIcon"),
      )
      InfoRow(label = "名称", value = foreground.readable("label", "未知"))
      InfoRow(label = "包名", value = foreground.readable("packageName", "未知"))
    }

    InfoGroup(title = "媒体播放") {
      InfoRowWithIcon(
        label = "封面",
        iconAsset = media?.optJSONObject("artwork")?.assetBytes(snapshot),
        fallback = media.assetLabel("artwork"),
      )
      InfoRow(label = "标题", value = media.readable("title", "未检测到"))
      InfoRow(label = "作者", value = media.readable("artist", "未知"))
      InfoRow(label = "专辑", value = media.readable("album", "未知"))
      InfoRow(label = "状态", value = media.playbackStateLabel())
    }

    InfoGroup(title = "设备状态") {
      InfoRow(label = "电量", value = "${battery?.optInt("level", -1)?.takeIf { it >= 0 } ?: "未知"}%")
      InfoRow(label = "充电", value = battery?.optBoolean("charging", false).yesNo())
      InfoRow(label = "网络", value = network.networkLabel())
    }
  }
}

@Composable
private fun InfoGroup(title: String, content: @Composable ColumnScope.() -> Unit) {
  Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
    MiuixText(text = title, color = Color(0xFF15171C), fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
    content()
  }
}

@Composable
private fun InfoRow(label: String, value: String) {
  Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
    MiuixText(text = label, color = Color(0xFF646A73), fontSize = 13.sp)
    MiuixText(text = value, color = Color(0xFF20242B), fontSize = 13.sp)
  }
}

@Composable
private fun InfoRowWithIcon(label: String, iconAsset: ImageBitmap?, fallback: String) {
  Row(
    modifier = Modifier.fillMaxWidth(),
    horizontalArrangement = Arrangement.SpaceBetween,
    verticalAlignment = Alignment.CenterVertically,
  ) {
    MiuixText(text = label, color = Color(0xFF646A73), fontSize = 13.sp)
    if (iconAsset != null) {
      Image(
        bitmap = iconAsset,
        contentDescription = label,
        modifier = Modifier
          .size(40.dp)
          .clip(RoundedCornerShape(8.dp)),
      )
    } else {
      MiuixText(text = fallback, color = Color(0xFF20242B), fontSize = 13.sp)
    }
  }
}

@Composable
private fun JSONObject?.assetBytes(snapshot: DeviceSnapshot): ImageBitmap? {
  val identifier = this?.optString("contentItemIdentifier").orEmpty()
  if (identifier.isBlank()) return null
  val bytes = snapshot.assets.firstOrNull { it.id == identifier }?.bytes ?: return null
  val bitmap = remember(bytes) {
    runCatching { BitmapFactory.decodeByteArray(bytes, 0, bytes.size) }.getOrNull()
  } ?: return null
  return remember(bitmap) { bitmap.asImageBitmap() }
}

private fun JSONObject?.readable(key: String, fallback: String): String =
  this?.optString(key)?.takeIf(String::isNotBlank) ?: fallback

private fun JSONObject?.assetLabel(key: String): String {
  val asset = this?.optJSONObject(key) ?: return "无"
  val identifier = asset.optString("contentItemIdentifier")
  return if (identifier.isBlank()) "无" else "${asset.optInt("width")}x${asset.optInt("height")}"
}

private fun JSONObject?.playbackStateLabel(): String =
  when (this?.optInt("state", 0)) {
    3 -> "播放中"
    2 -> "暂停"
    1 -> "停止"
    else -> "未知"
  }

private fun JSONObject?.networkLabel(): String {
  if (this == null || !optBoolean("connected", false)) return "离线"
  return when {
    optBoolean("wifi", false) -> "Wi-Fi"
    optBoolean("cellular", false) -> "蜂窝网络"
    optBoolean("vpn", false) -> "VPN"
    else -> "已连接"
  }
}

private fun Boolean?.yesNo(): String =
  when (this) {
    true -> "是"
    false -> "否"
    null -> "未知"
  }