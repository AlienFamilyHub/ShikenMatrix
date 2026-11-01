import { reactive } from "vue";

export type ToastTone = "success" | "error" | "info";

export interface Toast {
  id: number;
  tone: ToastTone;
  message: string;
}

export const toasts = reactive<Toast[]>([]);

let nextId = 1;

export function dismissToast(id: number): void {
  const index = toasts.findIndex(toast => toast.id === id);
  if (index >= 0)
    toasts.splice(index, 1);
}

export function notify(message: string, tone: ToastTone = "success", ttl = 3600): void {
  const id = nextId++;
  toasts.push({ id, tone, message });
  if (toasts.length > 3)
    toasts.shift();
  window.setTimeout(dismissToast, ttl, id);
}
