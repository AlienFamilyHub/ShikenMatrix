<script setup lang="ts" vapor>
defineProps<{ eyebrow: string; title: string; description: string }>();
</script>

<template>
  <div class="view-head">
    <div class="view-head__text">
      <span class="eyebrow"><slot name="icon" />{{ eyebrow }}</span>
      <h1 class="view-title">
        {{ title }}
      </h1>
      <p class="view-sub">
        {{ description }}
      </p>
    </div>
    <div class="view-head__aside">
      <slot name="aside" />
    </div>
  </div>
</template>

<style scoped>
.view-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: 0.75rem 1rem;
  margin-bottom: 1.25rem;
}

.view-head__text {
  display: grid;
  gap: 0.2rem;
  min-width: 0;
}

.view-head__aside {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-left: auto;
}

.eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--faint);
  font-size: 12px;
  font-weight: 620;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.eyebrow :deep(svg) {
  width: 1.05em;
  height: 1.05em;
}

.view-title {
  font-size: clamp(1.55rem, 3.4vw, 2rem);
}

.view-sub {
  max-width: 62ch;
  color: var(--muted);
  font-size: 13.5px;
}
</style>
