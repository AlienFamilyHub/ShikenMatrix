import type { AppPage, ConnectionStatus, PermissionStatus } from "../types";
import { Show } from "solid-js";
import IconDashboard from "~icons/mingcute/computer-line";
import IconMonitorOn from "~icons/mingcute/eye-2-line";
import IconMonitorOff from "~icons/mingcute/eye-close-line";
import IconInfo from "~icons/mingcute/information-line";
import IconMusic from "~icons/mingcute/music-2-line";
import IconShieldOn from "~icons/mingcute/safe-shield-line";
import IconSettings from "~icons/mingcute/settings-3-line";
import IconShieldOff from "~icons/mingcute/shield-line";
import IconWifiOn from "~icons/mingcute/wifi-line";
import IconWifiOff from "~icons/mingcute/wifi-off-line";
import appIconUrl from "../assets/icon.svg";

interface AppHeaderProps {
  page: AppPage;
  status: ConnectionStatus;
  permissions: PermissionStatus;
  onPageChange: (page: AppPage) => void;
  onRequestAccessibility: () => void;
}

export function AppHeader(props: AppHeaderProps) {
  return (
    <header class="app-header">
      <div class="brand">
        <img class="brand-icon" src={appIconUrl} alt="" />
        <h1>ShikenMatrix</h1>
        <span class="version">v0.1.0</span>
      </div>

      <nav class="app-nav" aria-label="主导航">
        <button class={props.page === "monitor" ? "nav-item active" : "nav-item"} onClick={() => props.onPageChange("monitor")}>
          <IconDashboard />
          监控
        </button>
        <button class={props.page === "settings" ? "nav-item active" : "nav-item"} onClick={() => props.onPageChange("settings")}>
          <IconSettings />
          设置
        </button>
        <button class={props.page === "about" ? "nav-item active" : "nav-item"} onClick={() => props.onPageChange("about")}>
          <IconInfo />
          关于
        </button>
      </nav>

      <div class="status-pills">
        <div class="pill">
          <Show when={props.status.is_monitoring} fallback={<IconMonitorOff class="pill-icon" />}>
            <IconMonitorOn class="pill-icon success" />
          </Show>
          <span>
            监听
            {props.status.is_monitoring ? "已启动" : "未启动"}
          </span>
        </div>
        <div class="pill">
          <Show when={props.status.is_connected} fallback={<IconWifiOff class="pill-icon" />}>
            <IconWifiOn class="pill-icon success" />
          </Show>
          <span>
            上报
            {props.status.is_connected ? "已就绪" : (props.status.is_reporting ? "运行中" : "未启动")}
          </span>
        </div>
        <Show when={props.permissions.accessibility_required}>
          <Show
            when={!props.permissions.accessibility}
            fallback={(
              <div class="pill">
                <IconShieldOn class="pill-icon success" />
                <span>辅助功能</span>
              </div>
            )}
          >
            <button class="pill clickable" onClick={() => props.onRequestAccessibility()} title="点击请求辅助功能权限">
              <IconShieldOff class="pill-icon danger" />
              <span>辅助功能</span>
            </button>
          </Show>
        </Show>
        <div class="pill">
          <Show when={props.permissions.media} fallback={<IconMusic class="pill-icon danger" />}>
            <IconMusic class="pill-icon success" />
          </Show>
          <span>媒体控制</span>
        </div>
      </div>
    </header>
  );
}
