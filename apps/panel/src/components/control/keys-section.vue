<script setup lang="ts" vapor>
import type { ClientKeyEntry } from "@/lib/types";
import { onMounted, ref } from "vue";
import IconAdd from "~icons/mingcute/add-line";
import IconCopy from "~icons/mingcute/copy-line";
import IconDelete from "~icons/mingcute/delete-line";
import IconKey from "~icons/mingcute/key-1-line";
import AppButton from "@/components/ui/app-button.vue";
import ErrorNote from "@/components/ui/error-note.vue";
import IconButton from "@/components/ui/icon-button.vue";
import SectionCard from "@/components/ui/section-card.vue";
import StateNote from "@/components/ui/state-note.vue";
import TextField from "@/components/ui/text-field.vue";
import { api } from "@/lib/api";
import { formatUnixTime } from "@/lib/format";
import { notify } from "@/lib/toast";

const keys = ref<ClientKeyEntry[]>([]);
const loading = ref(true);
const creating = ref(false);
const revoking = ref(false);
const description = ref("");
const newKey = ref<string | null>(null);
const confirmingId = ref<number | null>(null);
const error = ref<string | null>(null);

async function refresh() {
  try {
    keys.value = await api.getClientKeys();
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error
      ? err.message
      : "加载客户端密钥失败";
  } finally {
    loading.value = false;
  }
}

async function create() {
  const label = description.value.trim();
  if (!label || creating.value)
    return;
  creating.value = true;
  error.value = null;
  try {
    const created = await api.createClientKey(label);
    newKey.value = created.api_key;
    description.value = "";
    await refresh();
    notify("客户端密钥已创建");
  } catch (err) {
    const message = err instanceof Error
      ? err.message
      : "创建失败";
    error.value = message;
    notify(message, "error");
  } finally {
    creating.value = false;
  }
}

async function revoke(id: number) {
  if (revoking.value)
    return;
  revoking.value = true;
  error.value = null;
  try {
    await api.deleteClientKey(id);
    confirmingId.value = null;
    await refresh();
    notify("客户端密钥已吊销");
  } catch (err) {
    const message = err instanceof Error
      ? err.message
      : "吊销失败";
    error.value = message;
    notify(message, "error");
  } finally {
    revoking.value = false;
  }
}

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    notify("已复制到剪贴板");
  } catch {
    notify("复制失败，请手动选中文本复制", "error");
  }
}

onMounted(refresh);
</script>

<template>
  <SectionCard
    id="keys"
    title="客户端密钥"
    description="为 Desktop / Android 客户端签发上报凭据"
  >
    <template #icon>
      <IconKey />
    </template>

    <StateNote v-if="loading">
      正在加载客户端密钥…
    </StateNote>

    <template v-else>
      <div class="key-create">
        <TextField
          label="描述"
          placeholder="例如：我的笔记本"
          :value="description"
          :disabled="creating"
          @update:value="description = $event"
          @submit="create"
        />
        <AppButton
          variant="primary"
          :disabled="creating || description.trim().length === 0"
          @click="create"
        >
          <IconAdd />{{ creating ? '生成中…' : '生成密钥' }}
        </AppButton>
      </div>
      <p class="hint">
        用于区分不同设备，便于日后吊销。
      </p>

      <div v-if="newKey" class="key-new">
        <span class="key-new__label">新密钥仅显示一次，请立即复制并填入客户端</span>
        <code class="key-new__value">{{ newKey }}</code>
        <div class="key-new__actions">
          <AppButton size="sm" @click="copy(newKey)">
            <IconCopy />复制
          </AppButton>
          <AppButton variant="ghost" size="sm" @click="newKey = null">
            我已保存
          </AppButton>
        </div>
      </div>

      <p v-if="keys.length === 0" class="hint">
        还没有客户端密钥。
      </p>

      <ul v-else class="key-list">
        <li v-for="key in keys" :key="key.id" class="key-row">
          <span class="key-row__text">
            <span class="key-row__desc">{{ key.description }}</span>
            <span class="key-row__meta">
              {{ key.api_key.slice(0, 10) }}… · {{ formatUnixTime(key.created_at) ?? '创建时间未知' }}
            </span>
          </span>

          <template v-if="confirmingId === key.id">
            <span class="key-row__confirm">吊销后该设备立即失效</span>
            <AppButton variant="ghost" size="sm" :disabled="revoking" @click="confirmingId = null">
              取消
            </AppButton>
            <AppButton variant="danger" size="sm" :disabled="revoking" @click="revoke(key.id)">
              {{ revoking ? '吊销中…' : '确认吊销' }}
            </AppButton>
          </template>

          <IconButton
            v-else
            compact
            :label="`吊销 ${key.description} 的密钥`"
            @click="confirmingId = key.id"
          >
            <IconDelete />
          </IconButton>
        </li>
      </ul>

      <ErrorNote v-if="error">
        {{ error }}
      </ErrorNote>
    </template>
  </SectionCard>
</template>

<style scoped>
.key-create {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: flex-end;
}

.key-create :deep(.field) {
  flex: 1 1 220px;
}

.hint {
  color: var(--faint);
  font-size: 12.5px;
}

.key-new {
  display: grid;
  gap: 0.5rem;
  padding: 0.75rem 0.85rem;
  border: 1px dashed color-mix(in oklab, var(--accent) 45%, var(--border));
  border-radius: var(--r-md);
  background: var(--accent-soft);
}

.key-new__label {
  color: var(--muted);
  font-size: 12.5px;
  font-weight: 570;
}

.key-new__value {
  padding: 0.45rem 0.55rem;
  overflow-wrap: anywhere;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  font-family: var(--font-mono);
  font-size: 12.5px;
}

.key-new__actions {
  display: flex;
  gap: 0.5rem;
}

.key-list {
  display: grid;
  gap: 0.4rem;
}

.key-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.55rem 0.7rem;
  border-radius: var(--r-md);
  background: var(--surface-muted);
}

.key-row__text {
  display: grid;
  gap: 0.05rem;
  flex: 1;
  min-width: 0;
}

.key-row__desc {
  font-size: 13.5px;
  font-weight: 550;
  overflow-wrap: anywhere;
}

.key-row__meta {
  color: var(--faint);
  font-size: 11.5px;
  font-family: var(--font-mono);
}

.key-row__confirm {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--danger);
  font-size: 12.5px;
}
</style>
