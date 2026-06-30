package moe.tnxg.shikenmatrix.mobile.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import top.yukonga.miuix.kmp.basic.NavigationBar
import top.yukonga.miuix.kmp.basic.NavigationBarDisplayMode
import top.yukonga.miuix.kmp.basic.NavigationBarItem
import top.yukonga.miuix.kmp.icon.MiuixIcons
import top.yukonga.miuix.kmp.icon.extended.Contacts
import top.yukonga.miuix.kmp.icon.extended.Settings
import top.yukonga.miuix.kmp.icon.extended.VerticalSplit

@Composable
internal fun ShikenNavigationBar(selectedTab: Int, onTabSelect: (Int) -> Unit) {
  val items = listOf(
    NavigationItemSpec("控制台", MiuixIcons.VerticalSplit),
    NavigationItemSpec("设置", MiuixIcons.Settings),
    NavigationItemSpec("隐私", MiuixIcons.Contacts),
  )

  NavigationBar(
    showDivider = false,
    defaultWindowInsetsPadding = true,
    mode = NavigationBarDisplayMode.IconAndText,
  ) {
    items.forEachIndexed { index, item ->
      NavigationBarItem(
        selected = selectedTab == index,
        onClick = { onTabSelect(index) },
        icon = item.icon,
        label = item.label,
      )
    }
  }
}

private data class NavigationItemSpec(
  val label: String,
  val icon: ImageVector,
)
