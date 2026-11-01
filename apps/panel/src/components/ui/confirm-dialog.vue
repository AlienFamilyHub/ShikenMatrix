<script setup lang="ts" vapor>
import { onMounted, onUnmounted, watch } from "vue";
import IconWarning from "~icons/mingcute/warning-line";
import AppButton from "@/components/ui/app-button.vue";

const props = defineProps<{
  open: boolean;
  title: string;
  message: string;
  confirmLabel: string;
  busy?: boolean;
}>();

const emit = defineEmits<{ confirm: []; cancel: [] }>();

function onKeydown(event: KeyboardEvent) {
  if (!props.open)
    return;
  if (event.key === "Escape" && !props.busy) {
    event.preventDefault();
    emit("cancel");
  }
}

watch(
  () => props.open,
  (open) => {
    if (!open)
      return;
    requestAnimationFrame(() => {
      document.querySelector<HTMLElement>("[data-dialog-cancel]")?.focus();
    });
  },
);

function onBackdrop() {
  if (!props.busy)
    emit("cancel");
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <div v-if="open" class="dialog-backdrop" @click.self="onBackdrop">
    <div class="dialog" role="dialog" aria-modal="true" :aria-label="title">
      <div class="dialog__head">
        <IconWarning />
        <h2 class="dialog__title">
          {{ title }}
        </h2>
      </div>
      <p class="dialog__body">
        {{ message }}
      </p>
      <div class="dialog__actions">
        <AppButton data-dialog-cancel :disabled="busy" @click="emit('cancel')">
          取消
        </AppButton>
        <AppButton variant="primary" :disabled="busy" @click="emit('confirm')">
          {{ busy ? '处理中…' : confirmLabel }}
        </AppButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: grid;
  place-items: center;
  padding: 1rem;
  background: rgb(10 9 16 / 0.5);
  backdrop-filter: blur(3px);
  animation: fade-in 140ms var(--ease);
}

.dialog {
  width: min(430px, 100%);
  padding: 1.25rem;
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  background: var(--surface);
  box-shadow: var(--shadow-2);
  animation: dialog-in 180ms var(--ease);
}

.dialog__head {
  display: flex;
  gap: 0.6rem;
  align-items: center;
  margin-bottom: 0.5rem;
}

.dialog__head svg {
  width: 1.5em;
  height: 1.5em;
  color: var(--danger);
}

.dialog__title {
  font-size: 16px;
}

.dialog__body {
  color: var(--muted);
  font-size: 13.5px;
}

.dialog__actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 1.25rem;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
}

@keyframes dialog-in {
  from {
    opacity: 0;
    translate: 0 10px;
    scale: 0.98;
  }
}
</style>
