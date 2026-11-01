<script setup lang="ts" vapor>
defineProps<{ title: string; hint?: string }>();
</script>

<template>
  <div class="empty">
    <slot />
    <span class="empty__title">{{ title }}</span>
    <span v-if="hint" class="empty__hint">{{ hint }}</span>
  </div>
</template>

<style scoped>
.empty {
  display: grid;
  justify-items: center;
  gap: 0.35rem;
  padding: 2.5rem 1rem;
  color: var(--faint);
  text-align: center;
}

.empty :deep(svg) {
  width: 2rem;
  height: 2rem;
  opacity: 0.55;
}

.empty__title {
  color: var(--muted);
  font-size: 14px;
  font-weight: 560;
}

.empty__hint {
  font-size: 13px;
  max-width: 34ch;
}
</style>
