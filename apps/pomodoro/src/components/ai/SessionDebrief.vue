<script setup lang="ts">
import { watch, ref, onUnmounted } from "vue";
import { useAiStore } from "@/stores/ai";

const ai = useAiStore();
const visible = ref(false);
let dismissTimer: ReturnType<typeof setTimeout> | null = null;

function clearTimer() {
  if (dismissTimer) {
    clearTimeout(dismissTimer);
    dismissTimer = null;
  }
}

function dismiss() {
  visible.value = false;
  clearTimer();
  setTimeout(() => ai.dismissDebrief(), 300);
}

watch(
  () => ai.debriefVisible,
  (show) => {
    if (show) {
      visible.value = true;
      clearTimer();
      dismissTimer = setTimeout(dismiss, 8000);
    } else {
      visible.value = false;
      clearTimer();
    }
  },
);

onUnmounted(clearTimer);
</script>

<template>
  <Teleport to="body">
    <Transition name="debrief">
      <div v-if="visible && ai.debrief" class="session-debrief" role="status">
        <div class="debrief-content">
          <!-- Brain icon -->
          <svg
            class="brain-icon"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M12 2a7 7 0 0 1 7 7c0 2.38-1.19 4.47-3 5.74V17a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2v-2.26C6.19 13.47 5 11.38 5 9a7 7 0 0 1 7-7z"
            />
            <path d="M9 21h6" />
            <path d="M10 17v4" />
            <path d="M14 17v4" />
            <path d="M12 2v5" />
            <path d="M8 9h8" />
          </svg>
          <span class="debrief-message">{{ ai.debrief.message }}</span>
        </div>
        <button class="debrief-dismiss" @click="dismiss" aria-label="Dismiss" type="button">
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
    </Transition>
  </Teleport>
</template>

<style scoped>
.session-debrief {
  position: fixed;
  bottom: 80px;
  right: 16px;
  z-index: 10000;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  max-width: 360px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-left: 3px solid var(--color-work);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-toast);
}

.debrief-content {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  flex: 1;
}

.brain-icon {
  color: var(--color-work);
  flex-shrink: 0;
  margin-top: 1px;
}

.debrief-message {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  line-height: 1.4;
}

.debrief-dismiss {
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

.debrief-dismiss:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Transitions */
.debrief-enter-active {
  transition: all 0.3s ease;
}

.debrief-leave-active {
  transition: all 0.2s ease;
}

.debrief-enter-from {
  opacity: 0;
  transform: translateX(24px);
}

.debrief-leave-to {
  opacity: 0;
  transform: translateX(24px);
}
</style>
