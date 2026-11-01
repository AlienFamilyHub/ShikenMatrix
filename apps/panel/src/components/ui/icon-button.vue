<script setup lang="ts" vapor>
defineProps<{
  label: string;
  compact?: boolean;
  type?: "button" | "submit";
  disabled?: boolean;
}>();
</script>

<template>
  <button
    class="icon-btn"
    :class="{ 'icon-btn--compact': compact }"
    :type="type ?? 'button'"
    :aria-label="label"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
.icon-btn {
  display: inline-grid;
  place-items: center;
  width: var(--tap);
  height: var(--tap);
  border-radius: var(--r-full);
  color: var(--muted);
  transition:
    background-color 150ms var(--ease),
    color 150ms var(--ease);
}

.icon-btn--compact {
  width: 36px;
  height: 36px;
}

.icon-btn:hover:not(:disabled) {
  background: var(--surface-strong);
  color: var(--text);
}

.icon-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.icon-btn :deep(svg) {
  width: 1.3em;
  height: 1.3em;
}

.icon-btn--compact :deep(svg) {
  width: 1.15em;
  height: 1.15em;
}
</style>
