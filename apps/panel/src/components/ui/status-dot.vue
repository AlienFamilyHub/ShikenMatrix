<script setup lang="ts" vapor>
withDefaults(defineProps<{ online?: boolean; large?: boolean }>(), {
  online: false,
  large: false,
});
</script>

<template>
  <span
    class="status-dot"
    :class="{ 'status-dot--online': online, 'status-dot--large': large }"
  />
</template>

<style scoped>
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--faint);
  flex: none;
}

.status-dot--large {
  width: 10px;
  height: 10px;
}

.status-dot--online {
  background: var(--success);
  box-shadow: 0 0 0 3px var(--success-soft);
}
</style>
