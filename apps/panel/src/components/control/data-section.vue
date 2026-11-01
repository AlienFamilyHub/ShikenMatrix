<script setup lang="ts" vapor>
import type { DataSummary } from "@/lib/types";
import { computed, onMounted, ref } from "vue";
import IconDelete from "~icons/mingcute/delete-line";
import IconRefresh from "~icons/mingcute/refresh-1-line";
import IconDatabase from "~icons/mingcute/storage-line";
import AppButton from "@/components/ui/app-button.vue";
import ConfirmDialog from "@/components/ui/confirm-dialog.vue";
import ErrorNote from "@/components/ui/error-note.vue";
import SectionCard from "@/components/ui/section-card.vue";
import StateNote from "@/components/ui/state-note.vue";
import { api } from "@/lib/api";
import { formatCount } from "@/lib/format";
import { notify } from "@/lib/toast";

type DangerAction = "clear" | "reset";

const ACTIONS: Record<DangerAction, { title: string; message: string; confirmLabel: string }> = {
  clear: {
    title: "清空活动日志？",
    message: "将永久删除所有已记录的活动事件，此操作不可撤销。",
    confirmLabel: "确认清空",
  },
  reset: {
    title: "重置运行时统计？",
    message: "累计上报消息数等运行时统计将归零，此操作不可撤销。",
    confirmLabel: "确认重置",
  },
};

const data = ref<DataSummary | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);
const pending = ref<DangerAction | null>(null);
const running = ref(false);

const pendingCopy = computed(() => (pending.value ? ACTIONS[pending.value] : null));

async function refresh() {
  try {
    data.value = await api.getData();
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error
      ? err.message
      : "加载数据失败";
  } finally {
    loading.value = false;
  }
}

async function confirm() {
  const action = pending.value;
  if (!action || running.value)
    return;
  running.value = true;
  error.value = null;
  try {
    if (action === "clear")
      await api.clearActivity();
    else await api.resetStats();
    pending.value = null;
    notify(action === "clear"
      ? "活动日志已清空"
      : "运行时统计已重置");
    await refresh();
  } catch (err) {
    const message = err instanceof Error
      ? err.message
      : "操作失败";
    error.value = message;
    notify(message, "error");
  } finally {
    running.value = false;
  }
}

onMounted(refresh);
</script>

<template>
  <SectionCard
    id="data"
    title="数据管理"
    description="查看已采集的数据，并在需要时清理"
    footer-hint="危险操作会立即生效"
  >
    <template #icon>
      <IconDatabase />
    </template>

    <StateNote v-if="loading">
      正在读取数据统计…
    </StateNote>

    <template v-else>
      <div class="stat-grid">
        <div class="stat">
          <span class="stat__value">{{ formatCount(data?.total_events) }}</span>
          <span class="stat__label">活动事件总数</span>
        </div>
        <div class="stat">
          <span class="stat__value">{{ formatCount(data?.total_messages) }}</span>
          <span class="stat__label">累计上报消息</span>
        </div>
        <div class="stat">
          <span class="stat__value">{{ formatCount(data?.window_info_count) }}</span>
          <span class="stat__label">窗口上报</span>
        </div>
        <div class="stat">
          <span class="stat__value">{{ formatCount(data?.media_playback_count) }}</span>
          <span class="stat__label">媒体播放</span>
        </div>
        <div class="stat">
          <span class="stat__value">{{ formatCount(data?.artwork_uploads) }}</span>
          <span class="stat__label">封面资源</span>
        </div>
        <div class="stat">
          <span class="stat__value">{{ formatCount(data?.upstream_errors) }}</span>
          <span class="stat__label">上游错误</span>
        </div>
      </div>

      <div class="danger-list">
        <div class="danger-row">
          <span class="danger-row__text">
            <span class="danger-row__title">清空活动日志</span>
            <span class="danger-row__hint">删除全部历史活动事件，不影响客户端连接</span>
          </span>
          <AppButton variant="danger" size="sm" @click="pending = 'clear'">
            <IconDelete />清空日志
          </AppButton>
        </div>
        <div class="danger-row">
          <span class="danger-row__text">
            <span class="danger-row__title">重置运行时统计</span>
            <span class="danger-row__hint">上报计数归零，已保存的数据不受影响</span>
          </span>
          <AppButton variant="danger" size="sm" @click="pending = 'reset'">
            <IconRefresh />重置统计
          </AppButton>
        </div>
      </div>

      <ErrorNote v-if="error">
        {{ error }}
      </ErrorNote>
    </template>

    <template #footer>
      <AppButton size="sm" :disabled="loading" @click="refresh">
        <IconRefresh />刷新数据
      </AppButton>
    </template>

    <ConfirmDialog
      :open="pending !== null"
      :title="pendingCopy?.title ?? ''"
      :message="pendingCopy?.message ?? ''"
      :confirm-label="pendingCopy?.confirmLabel ?? '确认'"
      :busy="running"
      @confirm="confirm"
      @cancel="pending = null"
    />
  </SectionCard>
</template>

<style scoped>
.stat-grid {
  display: grid;
  gap: 0.6rem;
  grid-template-columns: repeat(auto-fit, minmax(132px, 1fr));
}

.stat {
  display: grid;
  gap: 0.05rem;
  padding: 0.7rem 0.85rem;
  border-radius: var(--r-md);
  background: var(--surface-muted);
}

.stat__value {
  font-size: 1.3rem;
  font-weight: 640;
  font-variant-numeric: tabular-nums;
}

.stat__label {
  color: var(--faint);
  font-size: 11.5px;
}

.danger-list {
  display: grid;
  gap: 0.5rem;
}

.danger-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
  padding: 0.7rem 0.85rem;
  border: 1px solid color-mix(in oklab, var(--danger) 24%, var(--border));
  border-radius: var(--r-md);
  background: color-mix(in oklab, var(--danger) 4%, var(--surface));
}

.danger-row__text {
  display: grid;
  gap: 0.1rem;
  flex: 1 1 200px;
  min-width: 0;
}

.danger-row__title {
  font-size: 13.5px;
  font-weight: 560;
}

.danger-row__hint {
  color: var(--faint);
  font-size: 12px;
}
</style>
