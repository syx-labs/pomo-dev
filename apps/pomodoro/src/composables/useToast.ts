import { reactive } from "vue";

export interface Toast {
  id: string;
  message: string;
  type: "error" | "success" | "warning";
  timeout: number;
}

const toasts = reactive<Toast[]>([]);

let counter = 0;

export function useToast() {
  function showToast(message: string, type: Toast["type"] = "success", durationMs?: number) {
    const duration = durationMs ?? (type === "error" ? 6000 : type === "warning" ? 5000 : 4000);
    const id = `toast-${++counter}`;
    const toast: Toast = { id, message, type, timeout: duration };
    toasts.push(toast);
    setTimeout(() => dismissToast(id), duration);
    return id;
  }

  function dismissToast(id: string) {
    const idx = toasts.findIndex((t) => t.id === id);
    if (idx !== -1) toasts.splice(idx, 1);
  }

  return { toasts, showToast, dismissToast };
}
