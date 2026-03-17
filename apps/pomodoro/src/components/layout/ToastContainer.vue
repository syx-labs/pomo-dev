<script setup lang="ts">
import { useToast } from "@/composables/useToast";

const { toasts, dismissToast } = useToast();
</script>

<template>
  <Teleport to="body">
    <div class="toast-container" aria-live="assertive">
      <TransitionGroup name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="toast"
          :class="`toast--${toast.type}`"
          role="alert"
        >
          <span class="toast-message">{{ toast.message }}</span>
          <button
            class="toast-dismiss"
            @click="dismissToast(toast.id)"
            aria-label="Dismiss"
            type="button"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path
                d="M3.5 3.5L10.5 10.5M10.5 3.5L3.5 10.5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 16px;
  right: 16px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 360px;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-toast);
  pointer-events: auto;
}

.toast--error {
  border-left: 3px solid var(--color-priority-high);
}

.toast--success {
  border-left: 3px solid var(--color-work);
}

.toast--warning {
  border-left: 3px solid var(--color-priority-medium);
}

.toast-message {
  flex: 1;
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  line-height: 1.4;
}

.toast-dismiss {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 50%;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

.toast-dismiss:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Transitions */
.toast-enter-active {
  transition: all 0.3s ease;
}

.toast-leave-active {
  transition: all 0.2s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(24px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(24px);
}

.toast-move {
  transition: transform 0.25s ease;
}
</style>
