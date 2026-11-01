<script setup lang="ts" vapor>
import type { UpstreamSettings } from "@/lib/types";
import { computed, onMounted, ref } from "vue";
import IconSave from "~icons/mingcute/check-line";
import IconCloud from "~icons/mingcute/cloud-line";
import IconRefresh from "~icons/mingcute/refresh-1-line";
import AppButton from "@/components/ui/app-button.vue";
import ErrorNote from "@/components/ui/error-note.vue";
import SectionCard from "@/components/ui/section-card.vue";
import SegmentedControl from "@/components/ui/segmented-control.vue";
import StateNote from "@/components/ui/state-note.vue";
import SwitchRow from "@/components/ui/switch-row.vue";
import TextField from "@/components/ui/text-field.vue";
import { api } from "@/lib/api";
import { notify } from "@/lib/toast";

const EMPTY: UpstreamSettings = {
  protocol: "native",
  enable_media_reporting: true,
  native_ws_url: "",
  native_token: "",
  mix_space_endpoint: "",
  mix_space_method: "POST",
  mix_space_token: "",
  s3_enabled: false,
  s3_bucket: "",
  s3_region: "us-east-1",
  s3_access_key: "",
  s3_secret_key: "",
  s3_endpoint: "",
  s3_custom_domain: "",
  s3_key_template: "uploads/{year}/{month}/{filename}",
};

const PROTOCOLS = [
  { value: "native", label: "Native" },
  { value: "mix_space", label: "Mix-Space" },
];

const METHODS = ["POST", "PUT", "PATCH"].map(method => ({ value: method, label: method }));

const settings = ref<UpstreamSettings>({ ...EMPTY });
const baseline = ref<string | null>(null);
const loading = ref(true);
const saving = ref(false);
const saved = ref(false);
const error = ref<string | null>(null);

// URL hints live in script scope: Volar's Vapor template codegen drops the
// component scope for the rest of a file when a template attribute value
// contains two consecutive slashes.
const PLACEHOLDER = {
  nativeWs: "wss://example.com/ws",
  mixSpace: "https://api.example.com",
  s3: "https://s3.amazonaws.com",
  cdn: "https://cdn.example.com",
};

const dirty = computed(
  () => baseline.value !== null && JSON.stringify(settings.value) !== baseline.value,
);

function update(patch: Partial<UpstreamSettings>) {
  settings.value = {
    ...settings.value,
    ...patch,
  };
  saved.value = false;
}

function onProtocol(value: string) {
  update({ protocol: value as UpstreamSettings["protocol"] });
}

function onMethod(value: string) {
  update({ mix_space_method: value });
}

function replace(next: UpstreamSettings) {
  settings.value = next;
  baseline.value = JSON.stringify(next);
}

async function load() {
  loading.value = true;
  error.value = null;
  try {
    replace(await api.getUpstream());
  } catch (err) {
    error.value = err instanceof Error
      ? err.message
      : "加载上游配置失败";
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
    replace(await api.saveUpstream(settings.value));
    saved.value = true;
    notify("上游转发配置已保存");
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
  settings.value = JSON.parse(baseline.value) as UpstreamSettings;
  saved.value = false;
}

onMounted(load);
</script>

<template>
  <SectionCard
    id="upstream"
    title="上游转发"
    description="配置状态数据与媒体封面转发到哪个上游服务"
    :dirty="dirty"
    :saved="saved"
    footer-hint="修改后需保存才会生效"
  >
    <template #icon>
      <IconCloud />
    </template>

    <StateNote v-if="loading">
      正在加载上游配置…
    </StateNote>

    <template v-else>
      <SegmentedControl
        label="协议"
        :options="PROTOCOLS"
        :value="settings.protocol"
        @update:value="onProtocol"
      />

      <SwitchRow
        label="启用媒体上报"
        hint="向上游转发正在播放的曲目信息"
        :checked="settings.enable_media_reporting"
        @toggle="update({ enable_media_reporting: $event })"
      />

      <div v-if="settings.protocol === 'native'" class="field-group">
        <span class="field-group__title">Native</span>
        <TextField
          label="WebSocket URL"
          :value="settings.native_ws_url"
          :placeholder="PLACEHOLDER.nativeWs"
          inputmode="url"
          @update:value="update({ native_ws_url: $event })"
        />
        <TextField
          label="Token"
          type="password"
          revealable
          :value="settings.native_token"
          @update:value="update({ native_token: $event })"
        />
      </div>

      <div v-else class="field-group">
        <span class="field-group__title">Mix-Space</span>
        <TextField
          label="Endpoint"
          :value="settings.mix_space_endpoint"
          :placeholder="PLACEHOLDER.mixSpace"
          inputmode="url"
          @update:value="update({ mix_space_endpoint: $event })"
        />
        <SegmentedControl
          label="请求方法"
          :options="METHODS"
          :value="settings.mix_space_method"
          @update:value="onMethod"
        />
        <TextField
          label="Token"
          type="password"
          revealable
          :value="settings.mix_space_token"
          @update:value="update({ mix_space_token: $event })"
        />
      </div>

      <SwitchRow
        label="S3 存储"
        hint="把封面等媒体资源上传到对象存储"
        :checked="settings.s3_enabled"
        @toggle="update({ s3_enabled: $event })"
      />

      <div v-if="settings.s3_enabled" class="form-grid">
        <TextField
          label="Bucket"
          :value="settings.s3_bucket"
          @update:value="update({ s3_bucket: $event })"
        />
        <TextField
          label="Region"
          :value="settings.s3_region"
          placeholder="us-east-1"
          @update:value="update({ s3_region: $event })"
        />
        <TextField
          label="Access Key"
          :value="settings.s3_access_key"
          autocomplete="off"
          @update:value="update({ s3_access_key: $event })"
        />
        <TextField
          label="Secret Key"
          type="password"
          revealable
          autocomplete="off"
          :value="settings.s3_secret_key"
          @update:value="update({ s3_secret_key: $event })"
        />
        <TextField
          label="Endpoint"
          :value="settings.s3_endpoint"
          :placeholder="PLACEHOLDER.s3"
          inputmode="url"
          @update:value="update({ s3_endpoint: $event })"
        />
        <TextField
          label="Custom Domain"
          :value="settings.s3_custom_domain"
          :placeholder="PLACEHOLDER.cdn"
          inputmode="url"
          @update:value="update({ s3_custom_domain: $event })"
        />
        <TextField
          label="Key Template"
          hint="支持 {year} {month} {filename} 等占位符"
          :value="settings.s3_key_template"
          @update:value="update({ s3_key_template: $event })"
        />
      </div>

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

<style scoped>
.form-grid {
  display: grid;
  gap: 0.8rem;
  grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
}

.field-group {
  display: grid;
  gap: 0.7rem;
  padding: 0.85rem;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface-muted);
}

.field-group__title {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  color: var(--muted);
  font-size: 12px;
  font-weight: 620;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}
</style>
