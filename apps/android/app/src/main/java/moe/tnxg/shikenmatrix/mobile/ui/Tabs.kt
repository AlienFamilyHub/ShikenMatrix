package moe.tnxg.shikenmatrix.mobile.ui

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.calculateEndPadding
import androidx.compose.foundation.layout.calculateStartPadding
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import top.yukonga.miuix.kmp.basic.Button as MiuixButton
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
    contentPadding: PaddingValues,
    scrollState: LazyListState = rememberLazyListState(),
) {
    val layoutDirection = LocalLayoutDirection.current
    val headerProgress by animateFloatAsState(
        targetValue = if (scrollState.firstVisibleItemIndex > 0) {
            1f
        } else {
            (scrollState.firstVisibleItemScrollOffset / 72f).coerceIn(0f, 1f)
        },
        label = "dashboardHeaderProgress",
    )
    val expandedTitleTop = contentPadding.calculateTopPadding()
    val collapsedTitleTop = (expandedTitleTop - 44.dp).coerceAtLeast(0.dp)
    val animatedTitleTop = lerp(expandedTitleTop, collapsedTitleTop, headerProgress)

    // 顶部纯色背景：高度跟随当前标题位置，确保色块始终把标题下方滚动内容死死遮住
    val scrimBarHeight = animatedTitleTop + 16.dp

    Box(modifier = Modifier.fillMaxSize()) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            state = scrollState,
            contentPadding = PaddingValues(
                start = contentPadding.calculateStartPadding(layoutDirection),
                top = contentPadding.calculateTopPadding() + 48.dp,
                end = contentPadding.calculateEndPadding(layoutDirection),
                bottom = contentPadding.calculateBottomPadding(),
            ),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            item {
                HeaderSupportingContent(connected = connected, keepAlive = keepAlive, alpha = 1f)
            }
            item {
                Panel(title = "快捷操作") {
                    ActionRow {
                        ActionButton(label = if (connected) "重连 Server" else "连接 Server", onClick = onConnect)
                        ActionButton(label = "采集并发送", onClick = onCollectAndSend)
                    }
                }
            }
            item {
                SnapshotPanel(title = "运行日志简影", value = logs.take(10).joinToString("\n").ifBlank { "暂无日志" })
            }
            item {
                SnapshotDetailsPanel(snapshot = lastSnapshot)
            }
        }

        // 标题收起时出现的纯色背景块，死死遮住下方滚动内容
        Box(
            modifier = Modifier
                .align(Alignment.TopCenter)
                .fillMaxWidth()
                .height(scrimBarHeight)
                .background(Color(0xFFF6F7FB)),
        )

        HeaderTitle(
            collapseProgress = headerProgress,
            modifier = Modifier
                .align(Alignment.TopStart)
                .padding(
                    start = contentPadding.calculateStartPadding(layoutDirection),
                    end = contentPadding.calculateEndPadding(layoutDirection),
                )
                .offset(y = animatedTitleTop),
        )
    }
}

private fun lerp(start: Dp, stop: Dp, fraction: Float): Dp =
    start + (stop - start) * fraction.coerceIn(0f, 1f)

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
    onRequestRuntimePermissions: () -> Unit,
    onOpenUsageAccess: () -> Unit,
    onOpenNotificationListener: () -> Unit,
    onOpenBatterySettings: () -> Unit,
    onOpenLocationSettings: () -> Unit,
    permissionStatus: PermissionStatus,
    contentPadding: PaddingValues,
    scrollState: ScrollState = rememberScrollState(),
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(scrollState)
            .padding(contentPadding),
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
            PermissionRow(
                label = "粗略位置",
                granted = permissionStatus.coarseLocation,
                actionLabel = "申请",
                onClick = onRequestRuntimePermissions,
            )
            PermissionRow(
                label = "通知权限",
                granted = permissionStatus.postNotifications,
                actionLabel = "申请",
                onClick = onRequestRuntimePermissions,
            )
            PermissionRow(
                label = "使用情况访问",
                granted = permissionStatus.usageAccess,
                actionLabel = "去设置",
                onClick = onOpenUsageAccess,
            )
            PermissionRow(
                label = "通知监听器",
                granted = permissionStatus.notificationListener,
                actionLabel = "去设置",
                onClick = onOpenNotificationListener,
            )
            PermissionRow(
                label = "电池优化白名单",
                granted = permissionStatus.batteryOptimizationIgnored,
                actionLabel = "去设置",
                onClick = onOpenBatterySettings,
            )
            PermissionRow(
                label = "系统定位开关",
                granted = permissionStatus.locationEnabled,
                actionLabel = "去设置",
                onClick = onOpenLocationSettings,
            )
            if (permissionStatus.allGranted) {
                MiuixText(
                    text = "所有权限均已就绪 ✓",
                    color = Color(0xFF087A3E),
                    fontSize = 12.sp,
                    modifier = Modifier.padding(top = 4.dp),
                )
            }
        }
    }
}

@Composable
private fun PermissionRow(
    label: String,
    granted: Boolean,
    actionLabel: String,
    onClick: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 2.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            MiuixText(text = label, color = Color(0xFF30333A), fontSize = 15.sp)
            PermissionBadge(granted = granted)
        }
        MiuixButton(
            onClick = onClick,
            enabled = !granted,
        ) {
            MiuixText(
                text = if (granted) "已授权" else actionLabel,
                fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold,
                color = if (granted) Color(0xFF8E8E93) else Color.White,
            )
        }
    }
}

@Composable
private fun PermissionBadge(granted: Boolean) {
    MiuixText(
        text = if (granted) "已授权" else "未授权",
        modifier = Modifier
            .clip(RoundedCornerShape(6.dp))
            .background(if (granted) Color(0xFFE6F6ED) else Color(0xFFFFECEB))
            .padding(horizontal = 8.dp, vertical = 3.dp),
        color = if (granted) Color(0xFF087A3E) else Color(0xFFB42318),
        fontSize = 11.sp,
        fontWeight = FontWeight.SemiBold,
    )
}

@Composable
internal fun PrivacyTab(
    rootGranted: Boolean,
    rootMessage: String,
    onRequestRoot: () -> Unit,
    onOpenRootManager: () -> Unit,
    contentPadding: PaddingValues,
    scrollState: ScrollState = rememberScrollState(),
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(scrollState)
            .padding(contentPadding),
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
        PasswordField(
            label = "API Key",
            value = apiKey,
            onValueChange = onApiKeyChange,
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
        SettingsSwitchRow(
            label = "后台保活 / 自启动 watchdog",
            checked = keepAlive,
            onCheckedChange = { onToggleKeepAlive() })
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
