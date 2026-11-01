<script setup lang="ts" vapor>
import { computed, ref } from "vue";
import IconSave from "~icons/mingcute/check-line";
import IconLock from "~icons/mingcute/lock-line";
import AppButton from "@/components/ui/app-button.vue";
import ErrorNote from "@/components/ui/error-note.vue";
import SectionCard from "@/components/ui/section-card.vue";
import TextField from "@/components/ui/text-field.vue";
import { api } from "@/lib/api";
import { notify } from "@/lib/toast";

const current = ref("");
const next = ref("");
const confirm = ref("");
const saving = ref(false);
const error = ref<string | null>(null);

const mismatch = computed(() => confirm.value.length > 0 && next.value !== confirm.value);
const canSubmit = computed(
  () =>
    current.value.length > 0
    && next.value.length > 0
    && confirm.value.length > 0
    && !mismatch.value
    && !saving.value,
);

async function submit() {
  if (!canSubmit.value)
    return;
  saving.value = true;
  error.value = null;
  try {
    await api.changePassword(current.value, next.value);
    current.value = "";
    next.value = "";
    confirm.value = "";
    notify("密码已更新");
  } catch (err) {
    const message = err instanceof Error
      ? err.message
      : "更新失败";
    error.value = message;
    notify(message, "error");
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <SectionCard
    id="security"
    title="账户安全"
    description="修改管理员登录密码"
    footer-hint="密码修改后现有登录仍然有效"
  >
    <template #icon>
      <IconLock />
    </template>

    <div class="form-grid">
      <TextField
        label="当前密码"
        type="password"
        revealable
        autocomplete="current-password"
        :value="current"
        @update:value="current = $event"
      />
      <TextField
        label="新密码"
        type="password"
        revealable
        autocomplete="new-password"
        :value="next"
        @update:value="next = $event"
      />
      <TextField
        label="确认新密码"
        type="password"
        revealable
        autocomplete="new-password"
        :error="mismatch ? '两次输入的密码不一致' : null"
        :value="confirm"
        @update:value="confirm = $event"
        @submit="submit"
      />
    </div>

    <ErrorNote v-if="error">
      {{ error }}
    </ErrorNote>

    <template #footer>
      <AppButton variant="primary" size="sm" :disabled="!canSubmit" @click="submit">
        <IconSave />{{ saving ? '更新中…' : '更新密码' }}
      </AppButton>
    </template>
  </SectionCard>
</template>

<style scoped>
.form-grid {
  display: grid;
  gap: 0.8rem;
  grid-template-columns: minmax(0, 1fr);
}
</style>
