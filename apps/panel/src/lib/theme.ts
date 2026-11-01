import { ref } from "vue";

export type Theme = "light" | "dark";

const STORAGE_KEY = "shikenmatrix.theme";

function initialTheme(): Theme {
  const stored = document.documentElement.dataset.theme;
  return stored === "dark"
    ? "dark"
    : "light";
}

export const theme = ref<Theme>(initialTheme());

export function applyTheme(next: Theme): void {
  theme.value = next;
  document.documentElement.dataset.theme = next;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    /* ignore unavailable storage */
  }
}

export function toggleTheme(): void {
  applyTheme(theme.value === "dark"
    ? "light"
    : "dark");
}
