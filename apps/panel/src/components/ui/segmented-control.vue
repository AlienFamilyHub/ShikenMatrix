<script setup lang="ts" vapor>
export interface SegmentOption {
  value: string;
  label: string;
}

defineProps<{ label: string; options: SegmentOption[]; value: string }>();

const emit = defineEmits<{ "update:value": [value: string] }>();
</script>

<template>
  <div class="field">
    <span class="field__label">{{ label }}</span>
    <div class="seg" role="radiogroup" :aria-label="label">
      <button
        v-for="option in options"
        :key="option.value"
        type="button"
        role="radio"
        class="seg__item"
        :aria-checked="value === option.value ? 'true' : 'false'"
        @click="emit('update:value', option.value)"
      >
        {{ option.label }}
      </button>
    </div>
  </div>
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

.seg {
  display: inline-flex;
  gap: 2px;
  width: fit-content;
  padding: 3px;
  border-radius: var(--r-full);
  background: var(--surface-strong);
}

.seg__item {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-height: 32px;
  padding: 0 0.7rem;
  border-radius: var(--r-full);
  color: var(--muted);
  font-size: 13px;
  font-weight: 560;
  transition:
    background-color 150ms var(--ease),
    color 150ms var(--ease);
}

.seg__item:hover {
  color: var(--text);
}

.seg__item[aria-checked="true"] {
  background: var(--surface);
  color: var(--text);
  box-shadow: var(--shadow-1);
}

html[data-theme="dark"] .seg {
  background: var(--surface-muted);
}

html[data-theme="dark"] .seg__item[aria-checked="true"] {
  background: var(--surface-strong);
  color: var(--text);
  box-shadow: none;
}
</style>
