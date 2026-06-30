package moe.tnxg.shikenmatrix.mobile.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import top.yukonga.miuix.kmp.basic.Button as MiuixButton
import top.yukonga.miuix.kmp.basic.Card as MiuixCard
import top.yukonga.miuix.kmp.basic.Text as MiuixText
import top.yukonga.miuix.kmp.basic.TextField as MiuixTextField

@Composable
internal fun Header(connected: Boolean, keepAlive: Boolean) {
  Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
    MiuixText(
      text = "ShikenMatrix",
      fontSize = 32.sp,
      fontWeight = FontWeight.Bold,
      color = Color(0xFF15171C),
    )
    MiuixText(
      text = "Kotlin Compose reporter · media artwork and app icon telemetry",
      fontSize = 14.sp,
      color = Color(0xFF6B7280),
    )
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.padding(top = 4.dp)) {
      StatusPill(label = if (connected) "WS connected" else "WS offline", active = connected)
      StatusPill(label = if (keepAlive) "Watchdog on" else "Watchdog off", active = keepAlive)
    }
  }
}

@Composable
internal fun RootGrantStatus(rootGranted: Boolean, rootMessage: String) {
  if (!rootGranted) return

  Column(
    modifier = Modifier
      .fillMaxWidth()
      .padding(bottom = 12.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
    verticalArrangement = Arrangement.spacedBy(4.dp),
  ) {
    MiuixText(
      text = "✓",
      color = Color(0xFF16A34A),
      fontSize = 64.sp,
      fontWeight = FontWeight.Bold,
    )
    MiuixText(
      text = "Root 已授权",
      color = Color(0xFF087A3E),
      fontSize = 18.sp,
      fontWeight = FontWeight.SemiBold,
    )
    MiuixText(text = rootMessage, color = Color(0xFF4B5563), fontSize = 12.sp)
  }
}

@Composable
internal fun ActionRow(content: @Composable RowScope.() -> Unit) {
  Row(
    modifier = Modifier.fillMaxWidth(),
    horizontalArrangement = Arrangement.spacedBy(10.dp),
    content = content,
  )
}

@Composable
internal fun RowScope.ActionButton(label: String, onClick: () -> Unit) {
  MiuixButton(
    onClick = onClick,
    modifier = Modifier
      .height(44.dp)
      .weight(1f),
  ) {
    MiuixText(text = label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
  }
}

@Composable
internal fun LabeledField(
  label: String,
  value: String,
  onValueChange: (String) -> Unit,
  keyboardType: KeyboardType,
) {
  Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
    MiuixText(text = label, color = Color(0xFF646A73), fontSize = 13.sp)
    MiuixTextField(
      value = value,
      onValueChange = onValueChange,
      modifier = Modifier.fillMaxWidth(),
      keyboardOptions = KeyboardOptions(keyboardType = keyboardType),
      singleLine = true,
    )
  }
}

@Composable
internal fun SnapshotPanel(title: String, value: String) {
  Panel(title = title) {
    MiuixText(
      text = value,
      modifier = Modifier.fillMaxWidth(),
      color = Color(0xFF20242B),
      fontSize = 12.sp,
      fontFamily = FontFamily.Monospace,
    )
  }
}

@Composable
internal fun Panel(title: String, content: @Composable ColumnScope.() -> Unit) {
  MiuixCard(
    modifier = Modifier.fillMaxWidth(),
    insideMargin = PaddingValues(16.dp),
  ) {
    MiuixText(
      text = title,
      color = Color(0xFF15171C),
      fontSize = 17.sp,
      fontWeight = FontWeight.Bold,
      modifier = Modifier.padding(bottom = 12.dp),
    )
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
      content()
    }
  }
}

@Composable
private fun StatusPill(label: String, active: Boolean) {
  MiuixText(
    text = label,
    modifier = Modifier
      .clip(RoundedCornerShape(8.dp))
      .background(if (active) Color(0xFFE6F6ED) else Color(0xFFFFECEB))
      .padding(horizontal = 10.dp, vertical = 6.dp),
    color = if (active) Color(0xFF087A3E) else Color(0xFFB42318),
    fontSize = 12.sp,
    fontWeight = FontWeight.SemiBold,
  )
}
