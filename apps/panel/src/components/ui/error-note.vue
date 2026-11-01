<template>
  <p class="error-note" role="alert">
    <slot />
  </p>
</template>

<style scoped>
.error-note {
  padding: 0.65rem 0.8rem;
  border-radius: var(--r-sm);
  background: var(--danger-soft);
  color: var(--danger);
  font-size: 13px;
}
</style>
