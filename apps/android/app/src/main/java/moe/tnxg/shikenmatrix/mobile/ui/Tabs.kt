package moe.tnxg.shikenmatrix.mobile.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import top.yukonga.miuix.kmp.basic.Switch as MiuixSwitch
import top.yukonga.miuix.kmp.basic.Text as MiuixText
import moe.tnxg.shikenmatrix.mobile.nativebridge.DeviceSnapshot

@Composable
internal fun DashboardTab(
  connected: Boolean,
  keepAlive: Boolean,
  logs: List<String>,
  lastSnapshot: DeviceSnapshot?,
  onConnect: () -> Unit,
  onCollectAndSend: () -> Unit,
) {
  Column(
    modifier = Modifier
      .fillMaxSize()
      .verticalScroll(rememberScrollState())
      .padding(horizontal = 20.dp, vertical = 20.dp),
    verticalArrangement = Arrangement.spacedBy(16.dp),
  ) {
    Header(connected = connected, keepAlive = keepAlive)

    Panel(title = "快捷操作") {
      ActionRow {
        ActionButton(label = if (connected) "重连 Server" else "连接 Server", onClick = onConnect)
        ActionButton(label = "采集并发送", onClick = onCollectAndSend)
      }
    }

    SnapshotPanel(title = "运行日志简影", value = logs.take(10).joinToString("\n").ifBlank { "暂无日志" })
    SnapshotDetailsPanel(snapshot = lastSnapshot)
  }
}

@Composable
internal fun SettingsTab(
  serverUrl: String,
  onServerUrlChange: (String) -> Unit,
  apiKey: String,
  onApiKeyChange: (String) -> Unit,
  reportIntervalMs: String,
  onReportIntervalChange: (String) -> Unit,
  autoReport: Boolean,
  onAutoReportChange: (Boolean) -> Unit,
  onSaveConfig: () -> Unit,
  keepAlive: Boolean,
  onToggleKeepAlive: () -> Unit,
  onRequestPermissions: () -> Unit,
  onOpenBatterySettings: () -> Unit,
  onOpenLocationSettings: () -> Unit,
) {
  Column(
    modifier = Modifier
      .fillMaxSize()
      .verticalScroll(rememberScrollState())
      .padding(horizontal = 20.dp, vertical = 20.dp),
    verticalArrangement = Arrangement.spacedBy(16.dp),
  ) {
    MiuixText("设置", fontSize = 28.sp, fontWeight = FontWeight.Bold, color = Color(0xFF15171C))

    SettingsPanel(
      serverUrl = serverUrl,
      onServerUrlChange = onServerUrlChange,
      apiKey = apiKey,
      onApiKeyChange = onApiKeyChange,
      reportIntervalMs = reportIntervalMs,
      onReportIntervalChange = onReportIntervalChange,
      autoReport = autoReport,
      onAutoReportChange = onAutoReportChange,
      onSaveConfig = onSaveConfig,
      keepAlive = keepAlive,
      onToggleKeepAlive = onToggleKeepAlive,
    )

    Panel(title = "系统权限与跳转") {
      ActionRow {
        ActionButton(label = "申请基础权限", onClick = onRequestPermissions)
      }
      ActionRow {
        ActionButton(label = "电池优化", onClick = onOpenBatterySettings)
        ActionButton(label = "定位设置", onClick = onOpenLocationSettings)
      }
    }
  }
}

@Composable
internal fun PrivacyTab(
  rootGranted: Boolean,
  rootMessage: String,
  onRequestRoot: () -> Unit,
  onOpenRootManager: () -> Unit,
) {
  Column(
    modifier = Modifier
      .fillMaxSize()
      .verticalScroll(rememberScrollState())
      .padding(horizontal = 20.dp, vertical = 20.dp),
    verticalArrangement = Arrangement.spacedBy(16.dp),
  ) {
    MiuixText("关于与隐私", fontSize = 28.sp, fontWeight = FontWeight.Bold, color = Color(0xFF15171C))

    Panel(title = "Root 权限说明") {
      RootGrantStatus(rootGranted = rootGranted, rootMessage = rootMessage)
      MiuixText(
        text = "开启 Root 可以获取更多深层硬件信息（如：安装的模块列表、系统属性、真实温度等）。未开启 Root 则仅获取基础信息。",
        color = Color(0xFF646A73),
        fontSize = 13.sp,
        modifier = Modifier.padding(bottom = 8.dp),
      )
      ActionRow {
        if (!rootGranted) {
          ActionButton(label = "申请 Root", onClick = onRequestRoot)
        }
        ActionButton(label = "Root 管理器", onClick = onOpenRootManager)
      }
    }

    Panel(title = "数据收集声明") {
      MiuixText(
        text = "本应用会收集以下信息并上报至您配置的服务器：\n• 基础设备标识与型号\n• 前台运行的应用包名与窗口\n• 正在播放的媒体信息(歌曲/作者/封面)\n\n我们绝不收集：\n• 您的键盘输入\n• 浏览历史\n• 相机或麦克风数据\n\n(注意：电池、网络、粗略位置在当前版本尽管采集但将被服务端直接忽略)",
        color = Color(0xFF646A73),
        fontSize = 13.sp,
      )
    }
  }
}

@Composable
private fun SettingsPanel(
  serverUrl: String,
  onServerUrlChange: (String) -> Unit,
  apiKey: String,
  onApiKeyChange: (String) -> Unit,
  reportIntervalMs: String,
  onReportIntervalChange: (String) -> Unit,
  autoReport: Boolean,
  onAutoReportChange: (Boolean) -> Unit,
  onSaveConfig: () -> Unit,
  keepAlive: Boolean,
  onToggleKeepAlive: () -> Unit,
) {
  Panel(title = "Server 连接") {
    LabeledField(
      label = "Server WS",
      value = serverUrl,
      onValueChange = onServerUrlChange,
      keyboardType = KeyboardType.Uri,
    )
    LabeledField(
      label = "API Key",
      value = apiKey,
      onValueChange = onApiKeyChange,
      keyboardType = KeyboardType.Text,
    )
    LabeledField(
      label = "自动上报间隔(ms)",
      value = reportIntervalMs,
      onValueChange = onReportIntervalChange,
      keyboardType = KeyboardType.Number,
    )
    ActionRow {
      ActionButton(label = "保存配置", onClick = onSaveConfig)
    }
    SettingsSwitchRow(label = "后台定时上报", checked = autoReport, onCheckedChange = onAutoReportChange)
    SettingsSwitchRow(label = "后台保活 / 自启动 watchdog", checked = keepAlive, onCheckedChange = { onToggleKeepAlive() })
  }
}

@Composable
private fun SettingsSwitchRow(label: String, checked: Boolean, onCheckedChange: (Boolean) -> Unit) {
  Row(
    modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
    horizontalArrangement = Arrangement.SpaceBetween,
    verticalAlignment = Alignment.CenterVertically,
  ) {
    MiuixText(text = label, color = Color(0xFF30333A), fontSize = 15.sp)
    MiuixSwitch(checked = checked, onCheckedChange = onCheckedChange)
  }
}
