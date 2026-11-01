<script setup lang="ts" vapor>
import type { ClientKind, ClientSnapshot, FeedState } from "@/lib/types";
import { computed } from "vue";
import IconCellphone from "~icons/mingcute/cellphone-line";
import IconMonitor from "~icons/mingcute/monitor-line";
import IconTime from "~icons/mingcute/time-line";
import PanelCard from "@/components/ui/panel-card.vue";
import StatusBadge from "@/components/ui/status-badge.vue";
import StatusDot from "@/components/ui/status-dot.vue";
import {
  formatAbsoluteTime,
  formatCount,
  formatRelativeTime,
} from "@/lib/format";

const props = defineProps<{
  kind: ClientKind;
  snapshot: ClientSnapshot | null;
  feed: FeedState;
}>();

const online = computed(() => props.snapshot?.status.is_online === true);
const screenOff = computed(() => props.kind === "mobile"
  ? props.snapshot?.status.screen_off ?? null
  : null);
const deviceInfo = computed(() => props.snapshot?.status.device_info ?? null);
const relative = computed(() => formatRelativeTime(props.snapshot?.status.last_activity_at ?? null));
const absolute = computed(() => formatAbsoluteTime(props.snapshot?.status.last_activity_at ?? null));
const messages = computed(() => props.snapshot?.stats.total_messages ?? null);

const feedLabel = computed(() => {
  if (props.feed === "live")
    return "实时连接";
  if (props.feed === "connecting")
    return "连接中…";
  return "已断开，重试中";
});
</script>

<template>
  <PanelCard title="连接状态">
    <template #icon>
      <IconTime />
    </template>

    <template #trailing>
      <StatusBadge
        v-if="feed !== 'live'"
        :tone="feed === 'connecting' ? 'accent' : 'warning'"
        dot
      >
        {{ feedLabel }}
      </StatusBadge>
    </template>

    <div class="connection">
      <div class="connection__headline">
        <StatusDot :online="online" large />
        <span class="connection__state">{{ online ? '在线' : '离线' }}</span>
        <span class="connection__kind">
          <StatusBadge>
            <IconCellphone v-if="kind === 'mobile'" />
            <IconMonitor v-else />
            {{ kind === 'mobile' ? 'Android' : 'Desktop' }}
          </StatusBadge>
        </span>
      </div>

      <div class="connection__rows">
        <div v-if="deviceInfo" class="connection__row">
          <span class="connection__label">设备</span>
          <span class="connection__value">{{ deviceInfo }}</span>
        </div>

        <div v-if="screenOff !== null" class="connection__row">
          <span class="connection__label">屏幕状态</span>
          <span class="connection__value">{{ screenOff ? '屏幕已熄屏' : '屏幕已点亮' }}</span>
        </div>

        <div v-if="online && relative" class="connection__row">
          <span class="connection__label">最后同步</span>
          <span class="connection__value" :title="absolute ?? undefined">{{ relative }}</span>
        </div>

        <div v-if="!online && absolute" class="connection__row">
          <span class="connection__label">最后同步</span>
          <span class="connection__value">{{ absolute }}</span>
        </div>

        <div v-if="!online && !absolute && snapshot === null" class="connection__row">
          <span class="connection__label">等待</span>
          <span class="connection__value">尚未收到该设备的数据</span>
        </div>
      </div>

      <div class="connection__foot">
        <span>{{ feed === 'live' ? '实时同步中' : feedLabel }}</span>
        <span class="connection__count">累计 {{ formatCount(messages) }} 条</span>
      </div>
    </div>
  </PanelCard>
</template>

<style scoped>
.connection {
  display: grid;
  gap: 0.85rem;
}

.connection__headline {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.connection__state {
  font-size: 1.35rem;
  font-weight: 640;
  letter-spacing: -0.015em;
}

.connection__kind {
  margin-left: auto;
}

.connection__rows {
  display: grid;
  gap: 0.55rem;
}

.connection__row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 1rem;
  font-size: 13.5px;
}

.connection__label {
  color: var(--faint);
  font-size: 12.5px;
}

.connection__value {
  min-width: 0;
  text-align: right;
  overflow-wrap: anywhere;
  font-variant-numeric: tabular-nums;
}

.connection__foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding-top: 0.7rem;
  border-top: 1px dashed var(--border);
  color: var(--faint);
  font-size: 12.5px;
}

.connection__count {
  color: var(--text);
  font-variant-numeric: tabular-nums;
  font-weight: 570;
}
</style>
