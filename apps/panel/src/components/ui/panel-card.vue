<script setup lang="ts" vapor>
defineProps<{ title: string }>();
</script>

<template>
  <section class="panel">
    <header class="panel__head">
      <slot name="icon" />
      <h2 class="panel__title">
        {{ title }}
      </h2>
      <span class="panel__trailing"><slot name="trailing" /></span>
    </header>

    <div class="panel__body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.panel {
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  background: var(--surface);
  box-shadow: var(--shadow-1);
}

.panel__head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.9rem 1rem 0.15rem;
}

.panel__head :deep(svg) {
  width: 1.1em;
  height: 1.1em;
  color: var(--accent);
}

.panel__title {
  font-size: 13px;
  font-weight: 620;
  color: var(--muted);
  letter-spacing: 0.01em;
}

.panel__trailing {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.panel__body {
  padding: 0.75rem 1rem 1rem;
}
</style>
