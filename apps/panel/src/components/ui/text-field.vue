<script setup lang="ts" vapor>
import { computed, ref } from "vue";
import IconEyeClose from "~icons/mingcute/eye-close-line";
import IconEye from "~icons/mingcute/eye-line";

const props = defineProps<{
  label: string;
  value: string;
  type?: string;
  hint?: string;
  error?: string | null;
  placeholder?: string;
  autocomplete?: string;
  inputmode?: "text" | "numeric" | "decimal" | "url" | "email";
  revealable?: boolean;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  "update:value": [value: string];
  "submit": [];
}>();

const revealed = ref(false);

function onInput(event: Event) {
  emit("update:value", (event.target as HTMLInputElement).value);
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" && !event.isComposing)
    emit("submit");
}

const inputType = computed(() => {
  if (props.type !== "password")
    return props.type ?? "text";
  return revealed.value ? "text" : "password";
});
</script>

<template>
  <label class="field">
    <span class="field__label">{{ label }}</span>
    <span class="field__control">
      <input
        class="input" :class="{ 'input--with-reveal': revealable }"
        :type="inputType"
        :value="value"
        :placeholder="placeholder"
        :autocomplete="autocomplete"
        :inputmode="inputmode"
        :disabled="disabled"
        :aria-invalid="error ? 'true' : undefined"
        @input="onInput"
        @keydown="onKeydown"
      >
      <button
        v-if="revealable"
        class="field__reveal"
        type="button"
        :aria-label="revealed ? '隐藏密码' : '显示密码'"
        @click="revealed = !revealed"
      >
        <IconEyeClose v-if="revealed" />
        <IconEye v-else />
      </button>
    </span>
    <span v-if="error" class="field__error">{{ error }}</span>
    <span v-else-if="hint" class="field__hint">{{ hint }}</span>
  </label>
</template>

<style scoped>
.field {
  display: grid;
  gap: 0.3rem;
  min-width: 0;
}

.field__label {
  color: var(--muted);
  font-size: 12.5px;
  font-weight: 570;
}

.field__control {
  position: relative;
  display: flex;
  align-items: center;
}

.input {
  width: 100%;
  min-height: 42px;
  padding: 0 0.85rem;
  border: 1px solid var(--border-strong);
  border-radius: var(--r-sm);
  background: var(--surface);
  font-size: 14px;
  transition:
    border-color 150ms var(--ease),
    box-shadow 150ms var(--ease);
}

.input::placeholder {
  color: var(--faint);
}

.input:hover:not(:disabled) {
  border-color: var(--accent-weak);
}

.input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}

.input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.input--with-reveal {
  padding-right: 2.9rem;
}

.input[aria-invalid="true"] {
  border-color: var(--danger);
}

.field__hint {
  color: var(--faint);
  font-size: 12.5px;
}

.field__error {
  color: var(--danger);
  font-size: 12.5px;
}

.field__reveal {
  position: absolute;
  right: 0.25rem;
  display: grid;
  place-items: center;
  width: 36px;
  height: 36px;
  border-radius: var(--r-sm);
  color: var(--faint);
}

.field__reveal:hover {
  color: var(--text);
  background: var(--surface-strong);
}

.field__reveal svg {
  width: 1.15em;
  height: 1.15em;
}
</style>
