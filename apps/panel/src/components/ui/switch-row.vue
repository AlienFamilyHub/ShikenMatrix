<script setup lang="ts" vapor>
defineProps<{
  label: string;
  hint?: string;
  checked: boolean;
}>();

const emit = defineEmits<{ toggle: [checked: boolean] }>();
</script>

<template>
  <div class="switch-row">
    <span class="switch-row__text">
      <span class="switch-row__label">{{ label }}</span>
      <span v-if="hint" class="switch-row__hint">{{ hint }}</span>
    </span>
    <button
      class="switch"
      type="button"
      role="switch"
      :aria-checked="checked ? 'true' : 'false'"
      :aria-label="label"
      @click="emit('toggle', !checked)"
    />
  </div>
</template>

<style scoped>
.switch-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem 0;
}

.switch-row + .switch-row {
  border-top: 1px dashed var(--border);
}

.switch-row__text {
  display: grid;
  gap: 0.1rem;
}

.switch-row__label {
  font-size: 14px;
  font-weight: 540;
}

.switch-row__hint {
  color: var(--faint);
  font-size: 12.5px;
}

.switch {
  position: relative;
  flex: none;
  width: 46px;
  height: 28px;
  margin-top: 0.1rem;
  border-radius: var(--r-full);
  background: var(--border-strong);
  transition: background-color 180ms var(--ease);
}

.switch::after {
  content: "";
  position: absolute;
  top: 3px;
  left: 3px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgb(0 0 0 / 0.3);
  transition: translate 180ms var(--ease);
}

.switch[aria-checked="true"] {
  background: var(--accent);
}

.switch[aria-checked="true"]::after {
  translate: 18px 0;
}
</style>
