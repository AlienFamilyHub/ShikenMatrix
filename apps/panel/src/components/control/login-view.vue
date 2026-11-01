<script setup lang="ts" vapor>
import { computed, ref } from "vue";
import IconLogin from "~icons/mingcute/exit-line";
import IconShield from "~icons/mingcute/safe-shield-line";
import AppButton from "@/components/ui/app-button.vue";
import TextField from "@/components/ui/text-field.vue";
import { api, ApiError } from "@/lib/api";

const emit = defineEmits<{ success: [] }>();

const username = ref("");
const password = ref("");
const submitting = ref(false);
const error = ref<string | null>(null);

const canSubmit = computed(
  () => username.value.trim().length > 0 && password.value.length > 0 && !submitting.value,
);

function onPassword(value: string) {
  password.value = value;
  error.value = null;
}

async function submit() {
  if (!canSubmit.value)
    return;
  submitting.value = true;
  error.value = null;
  try {
    await api.login(username.value.trim(), password.value);
    emit("success");
  } catch (err) {
    if (err instanceof ApiError && err.status === 401) {
      error.value = "用户名或密码不正确";
    } else {
      error.value = err instanceof Error
        ? err.message
        : "登录失败";
    }
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="auth-wrap">
    <form class="auth-card" @submit.prevent="submit">
      <div class="auth-head">
        <span class="auth-mark"><IconShield /></span>
        <h1 class="auth-title">
          控制面板登录
        </h1>
        <p class="auth-sub">
          输入管理员凭据后即可管理转发、接入与账户设置。
        </p>
      </div>

      <div class="auth-form">
        <TextField
          label="用户名"
          autocomplete="username"
          :value="username"
          :disabled="submitting"
          @update:value="username = $event"
        />
        <TextField
          label="密码"
          type="password"
          revealable
          autocomplete="current-password"
          :value="password"
          :disabled="submitting"
          :error="error"
          @update:value="onPassword"
          @submit="submit"
        />

        <AppButton variant="primary" block type="submit" :disabled="!canSubmit">
          <IconLogin />{{ submitting ? '登录中…' : '登录' }}
        </AppButton>
      </div>

      <p class="auth-hint">
        初始密码在服务端首次启动时打印至控制台。
      </p>
    </form>
  </div>
</template>

<style scoped>
.auth-wrap {
  display: grid;
  place-items: center;
  min-height: min(70vh, 560px);
  padding: 1rem 0;
}

.auth-card {
  display: grid;
  gap: 1.1rem;
  width: min(420px, 100%);
  padding: clamp(1.25rem, 4vw, 1.75rem);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  background: var(--surface);
  box-shadow: var(--shadow-2);
}

.auth-head {
  display: grid;
  justify-items: center;
  gap: 0.4rem;
  text-align: center;
}

.auth-mark {
  display: grid;
  place-items: center;
  width: 44px;
  height: 44px;
  margin-bottom: 0.2rem;
  border-radius: 14px;
  background: linear-gradient(140deg, var(--accent), var(--accent-weak));
  color: var(--on-accent);
}

.auth-mark svg {
  width: 1.4em;
  height: 1.4em;
}

.auth-title {
  font-size: 1.25rem;
}

.auth-sub {
  color: var(--muted);
  font-size: 13px;
  max-width: 32ch;
}

.auth-form {
  display: grid;
  gap: 0.85rem;
}

.auth-hint {
  color: var(--faint);
  font-size: 12px;
  text-align: center;
}
</style>
