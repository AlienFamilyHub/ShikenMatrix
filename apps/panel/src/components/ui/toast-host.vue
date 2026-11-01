<script setup lang="ts" vapor>
import IconCheckCircle from "~icons/mingcute/check-circle-line";
import IconClose from "~icons/mingcute/close-line";
import IconInformation from "~icons/mingcute/information-line";
import IconWarning from "~icons/mingcute/warning-line";
import IconButton from "@/components/ui/icon-button.vue";
import { dismissToast, toasts } from "@/lib/toast";
</script>

<template>
  <div class="toast-host" role="status" aria-live="polite">
    <div v-for="toast in toasts" :key="toast.id" class="toast" :class="[`toast--${toast.tone}`]">
      <IconCheckCircle v-if="toast.tone === 'success'" class="toast__icon" />
      <IconWarning v-else-if="toast.tone === 'error'" class="toast__icon" />
      <IconInformation v-else class="toast__icon" />
      <span>{{ toast.message }}</span>
      <IconButton compact label="关闭提示" @click="dismissToast(toast.id)">
        <IconClose />
      </IconButton>
    </div>
  </div>
</template>

<style scoped>
.toast-host {
  position: fixed;
  left: 50%;
  bottom: calc(1.1rem + env(safe-area-inset-bottom, 0px));
  z-index: 70;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  translate: -50% 0;
  width: min(420px, calc(100vw - 2rem));
}

.toast {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  padding: 0.6rem 0.9rem;
  border: 1px solid var(--border);
  border-radius: var(--r-full);
  background: var(--surface);
  box-shadow: var(--shadow-2);
  font-size: 13.5px;
  animation: toast-in 180ms var(--ease);
}

.toast > span {
  flex: 1;
  min-width: 0;
}

.toast__icon {
  width: 1.15em;
  height: 1.15em;
  flex: none;
}

.toast--success .toast__icon {
  color: var(--success);
}

.toast--error .toast__icon {
  color: var(--danger);
}

.toast--info .toast__icon {
  color: var(--accent);
}

@keyframes toast-in {
  from {
    opacity: 0;
    translate: 0 8px;
  }
}
</style>
