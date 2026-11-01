<script setup lang="ts" vapor>
withDefaults(
  defineProps<{
    tone?: "accent" | "success" | "warning";
    dot?: boolean;
    pulse?: boolean;
    /** Reserve a stable width so ticking labels never resize the pill. */
    fixed?: boolean;
  }>(),
  {
    tone: undefined,
    dot: false,
    pulse: false,
    fixed: false,
  },
);
</script>

<template>
  <span
    class="badge"
    :class="[
      tone ? `badge--${tone}` : null,
      { 'badge--live': pulse, 'badge--fixed': fixed },
    ]"
  >
    <span v-if="dot" class="badge__dot" />
    <slot />
  </span>
</template>

<style scoped>
.badge {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.18rem 0.55rem;
  border-radius: var(--r-full);
  background: var(--surface-strong);
  color: var(--muted);
  font-size: 12px;
  font-weight: 570;
  letter-spacing: 0.01em;
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}

.badge--fixed {
  justify-content: center;
  min-width: 8.5em;
}

.badge--accent {
  background: var(--accent-soft);
  color: var(--accent-strong);
}

.badge--success {
  background: var(--success-soft);
  color: var(--success);
}

.badge--warning {
  background: var(--warning-soft);
  color: var(--warning);
}

.badge :deep(svg) {
  width: 1.1em;
  height: 1.1em;
}

.badge__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.badge--live .badge__dot {
  animation: badge-pulse 2s var(--ease) infinite;
}

@keyframes badge-pulse {
  0%,
  100% {
    opacity: 1;
    box-shadow: 0 0 0 0 color-mix(in oklab, currentColor 55%, transparent);
  }
  50% {
    opacity: 0.65;
    box-shadow: 0 0 0 4px transparent;
  }
}
</style>
