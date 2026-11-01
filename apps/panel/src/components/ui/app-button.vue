<script setup lang="ts" vapor>
withDefaults(
  defineProps<{
    variant?: "primary" | "outline" | "ghost" | "danger";
    size?: "md" | "sm";
    block?: boolean;
    type?: "button" | "submit";
    disabled?: boolean;
  }>(),
  {
    variant: "outline",
    size: "md",
    block: false,
    type: "button",
    disabled: false,
  },
);
</script>

<template>
  <button
    class="btn"
    :class="[
      `btn--${variant}`,
      { 'btn--sm': size === 'sm', 'btn--block': block },
    ]"
    :type="type"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  min-height: var(--tap);
  padding: 0 1rem;
  border: 1px solid transparent;
  border-radius: var(--r-md);
  font-size: 14px;
  font-weight: 570;
  line-height: 1;
  white-space: nowrap;
  transition:
    background-color 150ms var(--ease),
    border-color 150ms var(--ease),
    color 150ms var(--ease),
    transform 120ms var(--ease);
}

.btn:active:not(:disabled) {
  transform: translateY(1px);
}

.btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.btn :deep(svg) {
  width: 1.15em;
  height: 1.15em;
}

.btn--primary {
  background: var(--accent);
  color: var(--on-accent);
}

.btn--primary:hover:not(:disabled) {
  background: var(--accent-strong);
}

.btn--outline {
  border-color: var(--border-strong);
  background: var(--surface);
  color: var(--text);
}

.btn--outline:hover:not(:disabled) {
  background: var(--surface-muted);
  border-color: var(--accent-weak);
}

.btn--ghost {
  color: var(--muted);
}

.btn--ghost:hover:not(:disabled) {
  background: var(--surface-strong);
  color: var(--text);
}

.btn--danger {
  border-color: color-mix(in oklab, var(--danger) 32%, var(--border));
  color: var(--danger);
}

.btn--danger:hover:not(:disabled) {
  background: var(--danger-soft);
  border-color: var(--danger);
}

.btn--sm {
  min-height: 34px;
  padding: 0 0.7rem;
  font-size: 13px;
  border-radius: var(--r-sm);
}

.btn--block {
  width: 100%;
}

@media (pointer: coarse) {
  .btn {
    min-height: 46px;
  }
}
</style>
