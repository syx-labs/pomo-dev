<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue";

const props = withDefaults(
  defineProps<{
    visible: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    confirmVariant?: "danger" | "default";
  }>(),
  {
    confirmLabel: "Confirm",
    confirmVariant: "default",
  },
);

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

const confirmBtn = ref<HTMLButtonElement | null>(null);
const dialogRef = ref<HTMLDivElement | null>(null);
let previouslyFocused: HTMLElement | null = null;

function handleKeydown(e: KeyboardEvent) {
  if (!props.visible) return;

  if (e.key === "Escape") {
    emit("cancel");
    return;
  }

  if (e.key === "Enter") {
    emit("confirm");
    return;
  }

  // Focus trap
  if (e.key === "Tab" && dialogRef.value) {
    const focusable = dialogRef.value.querySelectorAll<HTMLElement>(
      'button:not([disabled]), [tabindex]:not([tabindex="-1"])',
    );
    if (focusable.length === 0) return;

    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    if (e.shiftKey) {
      if (document.activeElement === first) {
        e.preventDefault();
        last.focus();
      }
    } else {
      if (document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }
}

watch(
  () => props.visible,
  async (vis) => {
    if (vis) {
      previouslyFocused = document.activeElement as HTMLElement | null;
      await nextTick();
      confirmBtn.value?.focus();
    } else {
      previouslyFocused?.focus();
      previouslyFocused = null;
    }
  },
);

function onOverlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains("confirm-overlay")) {
    emit("cancel");
  }
}

onMounted(() => {
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="confirm-modal">
      <div v-if="visible" class="confirm-overlay" @mousedown="onOverlayClick">
        <div ref="dialogRef" class="confirm-dialog" role="alertdialog" :aria-label="title">
          <h3 class="confirm-title">{{ title }}</h3>
          <p class="confirm-message">{{ message }}</p>
          <div class="confirm-actions">
            <button class="confirm-btn confirm-btn--ghost" @click="emit('cancel')" type="button">
              Cancel
            </button>
            <button
              ref="confirmBtn"
              class="confirm-btn"
              :class="confirmVariant === 'danger' ? 'confirm-btn--danger' : 'confirm-btn--primary'"
              @click="emit('confirm')"
              type="button"
            >
              {{ confirmLabel }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--overlay-bg);
  backdrop-filter: blur(4px);
}

.confirm-dialog {
  width: 100%;
  max-width: 380px;
  margin: 16px;
  padding: 24px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
}

.confirm-title {
  font-family: var(--font-sans);
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 8px;
}

.confirm-message {
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0 0 20px;
  line-height: 1.5;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.confirm-btn {
  padding: 8px 18px;
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 600;
  border-radius: var(--radius-sm);
  border: none;
  cursor: pointer;
  transition:
    background 0.15s ease,
    opacity 0.15s ease;
}

.confirm-btn--ghost {
  color: var(--text-secondary);
  background: transparent;
}

.confirm-btn--ghost:hover {
  color: var(--text-primary);
  background: var(--bg-card);
}

.confirm-btn--ghost:active {
  transform: scale(0.97);
}

.confirm-btn--primary {
  color: var(--bg-primary);
  background: var(--color-work);
}

.confirm-btn--primary:hover {
  opacity: 0.9;
}

.confirm-btn--primary:active {
  transform: scale(0.97);
}

.confirm-btn--danger {
  color: var(--text-primary);
  background: var(--color-priority-high);
}

.confirm-btn--danger:hover {
  opacity: 0.9;
}

.confirm-btn--danger:active {
  transform: scale(0.97);
}

/* Transition */
.confirm-modal-enter-active,
.confirm-modal-leave-active {
  transition: opacity 0.2s ease;
}

.confirm-modal-enter-active .confirm-dialog,
.confirm-modal-leave-active .confirm-dialog {
  transition:
    transform 0.2s ease,
    opacity 0.2s ease;
}

.confirm-modal-enter-from,
.confirm-modal-leave-to {
  opacity: 0;
}

.confirm-modal-enter-from .confirm-dialog,
.confirm-modal-leave-to .confirm-dialog {
  opacity: 0;
  transform: scale(0.95) translateY(8px);
}
</style>
