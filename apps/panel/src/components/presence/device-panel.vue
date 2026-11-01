<script setup lang="ts" vapor>
import type { DeviceInfo } from "@/lib/types";
import { computed } from "vue";
import IconBattery from "~icons/mingcute/battery-4-line";
import IconSmartphone from "~icons/mingcute/cellphone-line";
import IconLocation from "~icons/mingcute/location-line";
import IconShield from "~icons/mingcute/shield-line";
import IconSignal from "~icons/mingcute/signal-line";
import IconWifi from "~icons/mingcute/wifi-line";
import IconWifiOff from "~icons/mingcute/wifi-off-line";
import EmptyState from "@/components/ui/empty-state.vue";
import PanelCard from "@/components/ui/panel-card.vue";
import { anyPresent, present } from "@/lib/format";

const props = defineProps<{ device: DeviceInfo | null; online: boolean }>();

const hasDevice = computed(() => {
  const device = props.device;
  if (!device)
    return false;
  return anyPresent(
    device.battery_level,
    device.battery_charging,
    device.network_wifi,
    device.network_cellular,
    device.network_vpn,
    device.latitude,
    device.longitude,
  );
});

const battery = computed(() => props.device?.battery_level ?? null);
const charging = computed(() => props.device?.battery_charging === true);
const location = computed(() => {
  const device = props.device;
  if (!device || !present(device.latitude) || !present(device.longitude))
    return null;
  return `${(device.latitude as number).toFixed(4)}, ${(device.longitude as number).toFixed(4)}`;
});
</script>

<template>
  <PanelCard title="设备信息">
    <template #icon>
      <IconSmartphone />
    </template>

    <div v-if="hasDevice && device" class="metrics">
      <div v-if="battery !== null" class="metric">
        <IconBattery />
        <span class="metric__text">
          <span class="metric__value">{{ battery }}%</span>
          <span class="metric__label">{{ charging ? '充电中' : '电量' }}</span>
        </span>
      </div>

      <div v-if="present(device.network_wifi)" class="metric">
        <IconWifi v-if="device.network_wifi" />
        <IconWifiOff v-else />
        <span class="metric__text">
          <span class="metric__value">{{ device.network_wifi ? '已连接' : '未连接' }}</span>
          <span class="metric__label">Wi-Fi</span>
        </span>
      </div>

      <div v-if="present(device.network_cellular)" class="metric">
        <IconSignal />
        <span class="metric__text">
          <span class="metric__value">{{ device.network_cellular ? '已连接' : '未连接' }}</span>
          <span class="metric__label">蜂窝网络</span>
        </span>
      </div>

      <div v-if="present(device.network_vpn)" class="metric">
        <IconShield />
        <span class="metric__text">
          <span class="metric__value">{{ device.network_vpn ? '已开启' : '已关闭' }}</span>
          <span class="metric__label">VPN</span>
        </span>
      </div>

      <div v-if="location" class="metric metric--wide">
        <IconLocation />
        <span class="metric__text">
          <span class="metric__value">{{ location }}</span>
          <span class="metric__label">粗略位置</span>
        </span>
      </div>
    </div>

    <EmptyState
      v-else
      title="未上报设备信息"
      :hint="online ? 'Android 客户端会同步电量、网络与位置。' : '设备离线，暂无设备数据。'"
    >
      <IconSmartphone />
    </EmptyState>
  </PanelCard>
</template>

<style scoped>
.metrics {
  display: grid;
  gap: 0.6rem;
  grid-template-columns: repeat(auto-fit, minmax(132px, 1fr));
}

.metric {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  padding: 0.6rem 0.7rem;
  border-radius: var(--r-md);
  background: var(--surface-muted);
}

.metric svg {
  width: 1.25em;
  height: 1.25em;
  color: var(--muted);
}

.metric__text {
  display: grid;
  gap: 0.05rem;
  min-width: 0;
}

.metric__value {
  font-size: 14px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.metric__label {
  color: var(--faint);
  font-size: 11.5px;
}

.metric--wide {
  grid-column: 1 / -1;
}
</style>
