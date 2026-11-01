<script setup lang="ts" vapor>
import type { ActivityInfo } from "@/lib/types";
import { computed } from "vue";
import IconAppstore from "~icons/mingcute/appstore-line";
import IconApps from "~icons/mingcute/layout-grid-line";
import EmptyState from "@/components/ui/empty-state.vue";
import PanelCard from "@/components/ui/panel-card.vue";
import { anyPresent } from "@/lib/format";

const props = defineProps<{ activity: ActivityInfo | null; online: boolean }>();

const hasActivity = computed(() => {
  const activity = props.activity;
  if (!activity)
    return false;
  return anyPresent(activity.process_name, activity.title, activity.icon_url, activity.app_id);
});

const processName = computed(() => props.activity?.process_name ?? null);
const windowTitle = computed(() => props.activity?.title ?? null);
const appId = computed(() => props.activity?.app_id ?? null);
const iconUrl = computed(() => props.activity?.icon_url ?? null);

// Prefer the human-readable label (window title / app label) and keep the
// process or package name as supporting text.
const headline = computed(() => windowTitle.value ?? processName.value);
const subhead = computed(() =>
  windowTitle.value && processName.value && windowTitle.value !== processName.value
    ? processName.value
    : null,
);
const showAppId = computed(
  () => appId.value !== null && appId.value !== processName.value && appId.value !== headline.value,
);
</script>

<template>
  <PanelCard title="当前活动">
    <template #icon>
      <IconApps />
    </template>

    <div v-if="hasActivity" class="activity">
      <img v-if="iconUrl" class="activity__icon" :src="iconUrl" alt="">
      <div v-else class="activity__icon activity__icon--empty">
        <IconAppstore />
      </div>
      <div class="activity__text">
        <span v-if="headline" class="activity__name">{{ headline }}</span>
        <span v-if="subhead" class="activity__window">{{ subhead }}</span>
        <code v-if="showAppId" class="activity__appid">{{ appId }}</code>
      </div>
    </div>

    <EmptyState
      v-else
      title="未捕获活动窗口"
      :hint="online ? '切换前台窗口后，客户端会立即上报应用与窗口标题。' : '设备离线，暂无活动数据。'"
    >
      <IconApps />
    </EmptyState>
  </PanelCard>
</template>

<style scoped>
.activity {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.activity__icon {
  width: 44px;
  height: 44px;
  flex: none;
  border-radius: 12px;
  background: var(--surface-strong);
  object-fit: cover;
}

.activity__icon--empty {
  display: grid;
  place-items: center;
  color: var(--faint);
}

.activity__icon--empty svg {
  width: 1.4em;
  height: 1.4em;
  opacity: 0.7;
}

.activity__text {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.activity__name {
  font-size: 14.5px;
  font-weight: 600;
}

.activity__window {
  color: var(--muted);
  font-size: 13px;
  overflow-wrap: anywhere;
}

.activity__appid {
  justify-self: start;
  margin-top: 0.2rem;
  padding: 0.1rem 0.4rem;
  border-radius: 6px;
  background: var(--surface-strong);
  color: var(--faint);
  font-size: 11.5px;
}
</style>
