package moe.tnxg.shikenmatrix.mobile.ui

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import androidx.compose.ui.text.style.TextOverflow
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

        SummaryItem(
            title = foreground.readable("label", "未知应用"),
            subtitle = foreground.readable("packageName", "未知包名"),
            badge = "前台应用",
            image = foreground?.optJSONObject("appIcon")?.assetBytes(snapshot),
            fallback = foreground.assetLabel("appIcon"),
        )

        SummaryItem(
            title = media.readable("title", "未检测到媒体"),
            subtitle = listOf(
                media.readable("artist", "未知作者"),
                media.readable("album", "未知专辑"),
            ).joinToString(" · "),
            badge = media.playbackStateLabel(),
            image = media?.optJSONObject("artwork")?.assetBytes(snapshot),
            fallback = media.assetLabel("artwork"),
        )

        DeviceStatusRow(
            battery = "${battery?.optInt("level", -1)?.takeIf { it >= 0 } ?: "未知"}%",
            charging = battery?.optBoolean("charging", false).yesNo(),
            network = network.networkLabel(),
        )
    }
}

@Composable
private fun SummaryItem(
    title: String,
    subtitle: String,
    badge: String,
    image: ImageBitmap?,
    fallback: String,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(Color(0xFFF7F8FB))
            .padding(12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SnapshotImage(image = image, fallback = fallback)
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            MiuixText(
                text = badge,
                color = Color(0xFF646A73),
                fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold,
            )
            MiuixText(
                text = title,
                color = Color(0xFF20242B),
                fontSize = 15.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            MiuixText(
                text = subtitle,
                color = Color(0xFF646A73),
                fontSize = 12.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun SnapshotImage(image: ImageBitmap?, fallback: String) {
    if (image != null) {
        Image(
            bitmap = image,
            contentDescription = null,
            modifier = Modifier
                .size(48.dp)
                .clip(RoundedCornerShape(10.dp)),
        )
    } else {
        Box(
            modifier = Modifier
                .size(48.dp)
                .clip(RoundedCornerShape(10.dp))
                .background(Color(0xFFE7EAF0)),
            contentAlignment = Alignment.Center,
        ) {
            MiuixText(
                text = fallback.take(3).ifBlank { "无" },
                color = Color(0xFF646A73),
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun DeviceStatusRow(battery: String, charging: String, network: String) {
    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        StatusChip(label = "电量", value = battery, modifier = Modifier.weight(1f))
        StatusChip(label = "充电", value = charging, modifier = Modifier.weight(1f))
        StatusChip(label = "网络", value = network, modifier = Modifier.weight(1f))
    }
}

@Composable
private fun StatusChip(label: String, value: String, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(10.dp))
            .background(Color(0xFFF7F8FB))
            .padding(horizontal = 10.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        MiuixText(
            text = label,
            color = Color(0xFF646A73),
            fontSize = 11.sp,
        )
        MiuixText(
            text = value,
            color = Color(0xFF20242B),
            fontSize = 13.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
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
