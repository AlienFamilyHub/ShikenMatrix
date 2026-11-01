<script setup lang="ts" vapor>
import type { ClientKind, ClientSnapshot, FeedState } from "@/lib/types";
import { computed, onMounted, onUnmounted, reactive, ref } from "vue";
import IconCellphone from "~icons/mingcute/cellphone-line";
import IconMonitor from "~icons/mingcute/monitor-line";
import IconLive from "~icons/mingcute/signal-line";
import ActivityPanel from "@/components/presence/activity-panel.vue";
import ConnectionPanel from "@/components/presence/connection-panel.vue";
import DevicePanel from "@/components/presence/device-panel.vue";
import MediaPanel from "@/components/presence/media-panel.vue";
import PageHeading from "@/components/ui/page-heading.vue";
import StatusBadge from "@/components/ui/status-badge.vue";
import StatusDot from "@/components/ui/status-dot.vue";

const KINDS: ClientKind[] = ["desktop", "mobile"];

interface DeviceFeed {
  snapshot: ClientSnapshot | null;
  feed: FeedState;
  updatedAt: number | null;
}

const feeds = reactive<Record<ClientKind, DeviceFeed>>({
  desktop: { snapshot: null, feed: "connecting", updatedAt: null },
  mobile: { snapshot: null, feed: "connecting", updatedAt: null },
});

const active = ref<ClientKind>("desktop");
const userPicked = ref(false);
const now = ref(Date.now());

let clock: number | undefined;
const sources: EventSource[] = [];

function connect(kind: ClientKind) {
  const source = new EventSource(`/api/share/${kind}/events`);

  source.addEventListener("open", () => {
    feeds[kind].feed = "live";
  });

  source.addEventListener(kind, (event) => {
    try {
      const snapshot = JSON.parse((event as MessageEvent<string>).data) as ClientSnapshot;
      feeds[kind].snapshot = snapshot;
      feeds[kind].feed = "live";
      feeds[kind].updatedAt = Date.now();
      // Until the user picks a tab, follow a device that actually reports in.
      // Only take over while the selected device has no live data, otherwise
      // two online devices would swap the visible tab on every event.
      if (
        !userPicked.value
        && snapshot.status.is_online
        && feeds[active.value].snapshot?.status.is_online !== true
      ) {
        active.value = kind;
      }
    } catch {
      /* keep the last good snapshot when a malformed event slips through */
    }
  });

  source.addEventListener("error", () => {
    feeds[kind].feed = source.readyState === EventSource.CLOSED
      ? "offline"
      : "connecting";
  });

  sources.push(source);
}

function select(kind: ClientKind, focus = false) {
  userPicked.value = true;
  active.value = kind;
  if (focus) {
    document.querySelector<HTMLElement>(`[data-device-tab="${kind}"]`)?.focus();
  }
}

function onTabKeydown(event: KeyboardEvent, kind: ClientKind) {
  const index = KINDS.indexOf(kind);
  let next: ClientKind | undefined;
  if (event.key === "ArrowRight")
    next = KINDS[(index + 1) % KINDS.length];
  else if (event.key === "ArrowLeft")
    next = KINDS[(index - 1 + KINDS.length) % KINDS.length];
  else if (event.key === "Home")
    next = KINDS[0];
  else if (event.key === "End")
    next = KINDS[KINDS.length - 1];
  if (!next)
    return;
  event.preventDefault();
  select(next, true);
}

onMounted(() => {
  for (const kind of KINDS) connect(kind);
  clock = window.setInterval(() => {
    now.value = Date.now();
  }, 5_000);
});

onUnmounted(() => {
  for (const source of sources) source.close();
  if (clock !== undefined)
    window.clearInterval(clock);
});

const current = computed(() => feeds[active.value]);
const online = computed(() => current.value.snapshot?.status.is_online === true);
const snapshot = computed(() => current.value.snapshot);
const hasSnapshot = computed(() => current.value.snapshot !== null);
const kindOf = (kind: ClientKind) => feeds[kind];

const updatedLabel = computed(() => {
  const updatedAt = current.value.updatedAt;
  if (!updatedAt)
    return "等待设备数据";
  const seconds = Math.max(0, Math.round((now.value - updatedAt) / 1000));
  if (seconds < 10)
    return "刚刚更新";
  if (seconds < 60)
    return `${seconds} 秒前更新`;
  return `${Math.round(seconds / 60)} 分钟前更新`;
});

/** Header presence follows transport connectivity, not screen state. */
const headerBadge = computed(() => {
  if (online.value)
    return { tone: "success" as const, label: updatedLabel.value };
  return {
    tone: undefined,
    label: hasSnapshot.value ? "设备离线" : "等待设备数据",
  };
});
</script>

<template>
  <PageHeading
    eyebrow="Presence"
    title="设备状态"
    description="实时同步 Desktop 与 Android 客户端的连接状态、前台活动、媒体播放与设备信息。"
  >
    <template #icon>
      <IconLive />
    </template>

    <template #aside>
      <StatusBadge
        :tone="headerBadge.tone"
        :pulse="online"
        dot
        fixed
      >
        {{ headerBadge.label }}
      </StatusBadge>
    </template>
  </PageHeading>

  <div class="device-tabs" role="tablist" aria-label="选择设备">
    <button
      v-for="kind in KINDS"
      :id="`device-tab-${kind}`"
      :key="kind"
      :data-device-tab="kind"
      class="device-tab"
      type="button"
      role="tab"
      aria-controls="device-panel"
      :aria-selected="active === kind ? 'true' : 'false'"
      :tabindex="active === kind ? 0 : -1"
      @click="select(kind)"
      @keydown="onTabKeydown($event, kind)"
    >
      <IconCellphone v-if="kind === 'mobile'" />
      <IconMonitor v-else />
      {{ kind === 'mobile' ? 'Android' : 'Desktop' }}
      <StatusDot :online="kindOf(kind).snapshot?.status.is_online" />
      <StatusBadge v-if="kindOf(kind).snapshot === null">
        等待
      </StatusBadge>
    </button>
  </div>

  <div
    id="device-panel"
    role="tabpanel"
    :aria-labelledby="`device-tab-${active}`"
    class="presence-grid"
  >
    <div class="presence-col">
      <MediaPanel :media="snapshot?.media ?? null" :online="online" />
      <ActivityPanel :activity="snapshot?.activity ?? null" :online="online" />
    </div>

    <div class="presence-col presence-col--aside">
      <ConnectionPanel :kind="active" :snapshot="snapshot" :feed="current.feed" />
      <DevicePanel :device="snapshot?.device ?? null" :online="online" />
    </div>
  </div>
</template>

<style scoped>
.device-tabs {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1.25rem;
  padding-bottom: 0.2rem;
  overflow-x: auto;
  scrollbar-width: none;
}

.device-tabs::-webkit-scrollbar {
  display: none;
}

.device-tab {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  min-height: var(--tap);
  padding: 0 0.95rem;
  border: 1px solid var(--border);
  border-radius: var(--r-full);
  background: var(--surface);
  color: var(--muted);
  font-size: 13.5px;
  font-weight: 560;
  white-space: nowrap;
  transition:
    border-color 150ms var(--ease),
    color 150ms var(--ease),
    box-shadow 150ms var(--ease);
}

.device-tab:hover {
  color: var(--text);
  border-color: var(--border-strong);
}

.device-tab[aria-selected="true"] {
  border-color: color-mix(in oklab, var(--accent) 45%, var(--border));
  background: var(--surface);
  color: var(--text);
  box-shadow: var(--shadow-1);
}

.device-tab svg {
  width: 1.15em;
  height: 1.15em;
}

.presence-grid {
  display: grid;
  gap: 1rem;
  grid-template-columns: minmax(0, 1fr);
}

.presence-col {
  display: grid;
  gap: 1rem;
  align-content: start;
}

@media (min-width: 900px) {
  .presence-grid {
    grid-template-columns: minmax(0, 1.45fr) minmax(310px, 1fr);
    align-items: start;
  }

  .presence-col--aside {
    position: sticky;
    top: 5.5rem;
  }
}
</style>
