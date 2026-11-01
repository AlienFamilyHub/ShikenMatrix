<script setup lang="ts" vapor>
import type { AccessSettings } from "@/lib/types";
import { computed, onMounted, ref } from "vue";
import IconSave from "~icons/mingcute/check-line";
import IconLan from "~icons/mingcute/link-line";
import IconRefresh from "~icons/mingcute/refresh-1-line";
import AppButton from "@/components/ui/app-button.vue";
import ErrorNote from "@/components/ui/error-note.vue";
import SectionCard from "@/components/ui/section-card.vue";
import StateNote from "@/components/ui/state-note.vue";
import SwitchRow from "@/components/ui/switch-row.vue";
import TextField from "@/components/ui/text-field.vue";
import { api } from "@/lib/api";
import { notify } from "@/lib/toast";

const DEFAULT: AccessSettings = {
  accept_desktop: true,
  accept_mobile: true,
  activity_log_limit: 120,
};

const settings = ref<AccessSettings>({ ...DEFAULT });
const limitText = ref(String(DEFAULT.activity_log_limit));
const baseline = ref<string | null>(null);
const loading = ref(true);
const saving = ref(false);
const saved = ref(false);
const error = ref<string | null>(null);

const dirty = computed(
  () => baseline.value !== null && JSON.stringify(settings.value) !== baseline.value,
);

function replace(next: AccessSettings) {
  settings.value = next;
  limitText.value = String(next.activity_log_limit);
  baseline.value = JSON.stringify(next);
}

function update(patch: Partial<AccessSettings>) {
  settings.value = {
    ...settings.value,
    ...patch,
  };
  saved.value = false;
}

function onLimitInput(value: string) {
  limitText.value = value;
  const parsed = Math.floor(Number(value));
  update({
    activity_log_limit: Number.isFinite(parsed) && parsed > 0
      ? parsed
      : 1,
  });
}

async function load() {
  loading.value = true;
  error.value = null;
  try {
    replace(await api.getAccess());
  } catch (err) {
    error.value = err instanceof Error
      ? err.message
      : "加载接入控制失败";
  } finally {
    loading.value = false;
  }
}

async function save() {
  if (saving.value || !dirty.value)
    return;
  saving.value = true;
  error.value = null;
  try {
    replace(await api.saveAccess(settings.value));
    saved.value = true;
    notify("接入控制配置已保存");
  } catch (err) {
    const message = err instanceof Error
      ? err.message
      : "保存失败";
    error.value = message;
    notify(message, "error");
  } finally {
    saving.value = false;
  }
}

function revert() {
  if (baseline.value === null)
    return;
  replace(JSON.parse(baseline.value) as AccessSettings);
  saved.value = false;
}

onMounted(load);
</script>

<template>
  <SectionCard
    id="access"
    title="接入控制"
    description="决定哪类客户端可以上报数据，以及日志保留策略"
    :dirty="dirty"
    :saved="saved"
    footer-hint="断开后需要重启客户端才能重新接入"
  >
    <template #icon>
      <IconLan />
    </template>

    <StateNote v-if="loading">
      正在加载接入配置…
    </StateNote>

    <template v-else>
      <SwitchRow
        label="接受 Desktop 连接"
        hint="允许桌面客户端接入并上报状态"
        :checked="settings.accept_desktop"
        @toggle="update({ accept_desktop: $event })"
      />
      <SwitchRow
        label="接受 Android 连接"
        hint="允许移动端客户端接入并上报状态"
        :checked="settings.accept_mobile"
        @toggle="update({ accept_mobile: $event })"
      />

      <TextField
        label="活动日志保留条数"
        type="number"
        inputmode="numeric"
        hint="默认 120 条，超出后自动滚动覆盖（1–10000）"
        :value="limitText"
        @update:value="onLimitInput"
      />

      <ErrorNote v-if="error">
        {{ error }}
      </ErrorNote>
    </template>

    <template #footer>
      <AppButton variant="ghost" size="sm" :disabled="!dirty || saving" @click="revert">
        <IconRefresh />撤销修改
      </AppButton>
      <AppButton variant="primary" size="sm" :disabled="!dirty || saving" @click="save">
        <IconSave />{{ saving ? '保存中…' : '保存' }}
      </AppButton>
    </template>
  </SectionCard>
</template>
