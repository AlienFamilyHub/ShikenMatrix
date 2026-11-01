<script setup lang="ts" vapor>
import IconCheck from "~icons/mingcute/check-line";
import StatusBadge from "@/components/ui/status-badge.vue";

defineProps<{
  title: string;
  description: string;
  dirty?: boolean;
  saved?: boolean;
  footerHint?: string;
}>();
</script>

<template>
  <section class="section-card">
    <header class="section-card__head">
      <span class="section-card__icon"><slot name="icon" /></span>
      <span class="section-card__titles">
        <h2 class="section-card__title">{{ title }}</h2>
        <span class="section-card__desc">{{ description }}</span>
      </span>
      <span class="section-card__status">
        <StatusBadge v-if="dirty" tone="warning">
          未保存
        </StatusBadge>
        <StatusBadge v-else-if="saved" tone="success">
          <IconCheck />已保存
        </StatusBadge>
      </span>
    </header>

    <div class="section-card__body">
      <slot />
    </div>

    <footer v-if="$slots.footer" class="section-card__footer">
      <span v-if="footerHint" class="section-card__footer-hint">{{ footerHint }}</span>
      <slot name="footer" />
    </footer>
  </section>
</template>

<style scoped>
.section-card {
  display: grid;
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  background: var(--surface);
  box-shadow: var(--shadow-1);
  scroll-margin-top: 5.5rem;
}

.section-card__head {
  display: flex;
  align-items: flex-start;
  gap: 0.65rem;
  padding: 1rem 1rem 0;
}

.section-card__icon {
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  flex: none;
  border-radius: 10px;
  background: var(--accent-soft);
  color: var(--accent-strong);
}

.section-card__icon :deep(svg) {
  width: 1.2em;
  height: 1.2em;
}

.section-card__titles {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.section-card__title {
  font-size: 15px;
  font-weight: 620;
}

.section-card__desc {
  color: var(--muted);
  font-size: 12.5px;
}

.section-card__status {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.section-card__body {
  display: grid;
  gap: 0.9rem;
  padding: 0.9rem 1rem 1rem;
}

.section-card__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0.8rem 1rem 1rem;
  border-top: 1px dashed var(--border);
}

.section-card__footer-hint {
  margin-right: auto;
  color: var(--faint);
  font-size: 12px;
}
</style>
