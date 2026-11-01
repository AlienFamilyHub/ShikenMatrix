<script setup lang="ts" vapor>
import type { MediaInfo } from "@/lib/types";
import { computed, onUnmounted, ref, watch } from "vue";
import IconMusic from "~icons/mingcute/music-2-line";
import EmptyState from "@/components/ui/empty-state.vue";
import PanelCard from "@/components/ui/panel-card.vue";
import StatusBadge from "@/components/ui/status-badge.vue";
import { anyPresent, formatDuration, present } from "@/lib/format";

const props = defineProps<{ media: MediaInfo | null; online: boolean }>();

const elapsed = ref<number | null>(null);
let ticker: number | undefined;

function stopTicker() {
  if (ticker !== undefined) {
    window.clearInterval(ticker);
    ticker = undefined;
  }
}

function startTicker() {
  stopTicker();
  const media = props.media;
  if (!media?.playing)
    return;
  const base = media.elapsed_time ?? 0;
  const startedAt = Date.now();
  ticker = window.setInterval(() => {
    const next = base + (Date.now() - startedAt) / 1000;
    elapsed.value = media.duration
      ? Math.min(next, media.duration)
      : next;
  }, 1000);
}

watch(
  () => props.media,
  (media) => {
    elapsed.value = media?.elapsed_time ?? null;
    startTicker();
  },
  { immediate: true },
);

onUnmounted(stopTicker);

const hasMedia = computed(() => {
  const media = props.media;
  if (!media)
    return false;
  return anyPresent(
    media.title,
    media.artist,
    media.album,
    media.artwork_url,
    media.duration,
    media.elapsed_time,
    media.playing,
  );
});

const title = computed(() => props.media?.title ?? null);
const artist = computed(() => props.media?.artist ?? null);
const album = computed(() => props.media?.album ?? null);
const artwork = computed(() => props.media?.artwork_url ?? null);
const playing = computed(() => props.media?.playing === true);
const elapsedLabel = computed(() => formatDuration(elapsed.value));
const totalLabel = computed(() => formatDuration(props.media?.duration ?? null));

const percent = computed(() => {
  const duration = props.media?.duration;
  if (typeof duration !== "number" || duration <= 0 || elapsed.value === null)
    return 0;
  return Math.min(100, Math.max(0, (elapsed.value / duration) * 100));
});

const hasProgress = computed(
  () =>
    present(props.media?.duration) && (props.media?.duration ?? 0) > 0 && elapsed.value !== null,
);
</script>

<template>
  <PanelCard title="媒体播放">
    <template #icon>
      <IconMusic />
    </template>

    <template #trailing>
      <StatusBadge v-if="hasMedia" :tone="playing ? 'success' : 'accent'" dot>
        {{ playing ? '播放中' : '已暂停' }}
      </StatusBadge>
    </template>

    <div v-if="hasMedia" class="media">
      <div class="media__body">
        <img v-if="artwork" class="media__art" :src="artwork" alt="">
        <div v-else class="media__art media__art--empty">
          <IconMusic />
        </div>
        <div class="media__text">
          <span v-if="title" class="media__title">{{ title }}</span>
          <span v-if="artist" class="media__artist">{{ artist }}</span>
          <span v-if="album" class="media__album">{{ album }}</span>
        </div>
      </div>

      <div v-if="hasProgress" class="media__progress">
        <div
          class="progress"
          role="progressbar"
          aria-label="播放进度"
          aria-valuemin="0"
          aria-valuemax="100"
          :aria-valuenow="Math.round(percent)"
        >
          <div class="progress__fill" :style="{ width: `${percent}%` }" />
        </div>
        <div class="media__times">
          <span>{{ elapsedLabel ?? '0:00' }}</span>
          <span v-if="totalLabel">{{ totalLabel }}</span>
        </div>
      </div>
    </div>

    <EmptyState
      v-else
      title="暂无媒体播放"
      :hint="online ? '客户端开始播放音乐后，这里会显示曲目、封面与实时进度。' : '设备离线，恢复连接后会自动同步播放状态。'"
    >
      <IconMusic />
    </EmptyState>
  </PanelCard>
</template>

<style scoped>
.media__body {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.media__art {
  width: 112px;
  height: 112px;
  flex: none;
  border-radius: var(--r-md);
  background: var(--surface-strong);
  object-fit: cover;
  box-shadow: var(--shadow-1);
}

.media__art--empty {
  display: grid;
  place-items: center;
  color: var(--faint);
}

.media__art--empty svg {
  width: 2rem;
  height: 2rem;
  opacity: 0.6;
}

.media__text {
  display: grid;
  gap: 0.15rem;
  min-width: 0;
  align-content: center;
}

.media__title {
  font-size: 1.15rem;
  font-weight: 620;
  letter-spacing: -0.01em;
}

.media__artist {
  color: var(--muted);
  font-size: 13.5px;
}

.media__album {
  color: var(--faint);
  font-size: 12.5px;
}

.media__progress {
  display: grid;
  gap: 0.35rem;
  margin-top: 0.9rem;
}

.progress {
  height: 6px;
  overflow: hidden;
  border-radius: var(--r-full);
  background: var(--surface-strong);
}

.progress__fill {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--accent), var(--accent-weak));
  transition: width 1s linear;
}

.media__times {
  display: flex;
  justify-content: space-between;
  color: var(--faint);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

@media (max-width: 420px) {
  .media__art {
    width: 88px;
    height: 88px;
  }
}
</style>
