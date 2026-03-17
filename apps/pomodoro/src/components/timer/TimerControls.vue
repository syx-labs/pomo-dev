<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { useTimerStore } from "@/stores/timer";

const timer = useTimerStore();

const accentVar = computed(() => `var(--color-${timer.accentColor})`);
const accentDimVar = computed(() => `var(--color-${timer.accentColor}-dim)`);

async function handlePrimary() {
  if (timer.isIdle) {
    await timer.start();
  } else if (timer.isRunning) {
    await timer.pause();
  } else if (timer.isPaused) {
    await timer.resume();
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
  if (e.code === "Space") {
    e.preventDefault();
    handlePrimary();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <div class="timer-controls">
    <div class="controls-row">
      <!-- Reset -->
      <button
        class="control-btn secondary"
        :class="{ dimmed: timer.isIdle }"
        :disabled="timer.isIdle"
        title="Reset"
        @click="timer.reset()"
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
        </svg>
      </button>

      <!-- Primary: Play / Pause -->
      <button
        class="control-btn primary"
        :style="{
          '--btn-accent': accentVar,
          '--btn-accent-dim': accentDimVar,
        }"
        @click="handlePrimary"
      >
        <!-- Pause icon -->
        <svg v-if="timer.isRunning" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="4" width="4" height="16" rx="1" />
          <rect x="14" y="4" width="4" height="16" rx="1" />
        </svg>
        <!-- Play icon -->
        <svg v-else width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M8 5.14v13.72a1 1 0 0 0 1.5.86l11.04-6.86a1 1 0 0 0 0-1.72L9.5 4.28A1 1 0 0 0 8 5.14z"
          />
        </svg>
      </button>

      <!-- Skip -->
      <button
        class="control-btn secondary"
        :class="{ dimmed: timer.isIdle }"
        :disabled="timer.isIdle"
        title="Skip"
        @click="timer.skip()"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M5 4.5a1 1 0 0 1 1.54-.84l9.2 6.5a1 1 0 0 1 0 1.68l-9.2 6.5A1 1 0 0 1 5 17.5V4.5z"
          />
          <rect x="17" y="4" width="3" height="16" rx="1" />
        </svg>
      </button>
    </div>

    <!-- Finish early button -->
    <Transition name="fade-in">
      <button v-if="!timer.isIdle" class="finish-btn" @click="timer.finish()">
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
        Done
      </button>
    </Transition>

    <span v-if="timer.isIdle" class="hint-text"> <kbd>Space</kbd> to start </span>
    <span v-else class="hint-text">
      <kbd>Space</kbd> to {{ timer.isRunning ? "pause" : "resume" }}
    </span>
  </div>
</template>

<style scoped>
.timer-controls {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.controls-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
  outline: none;
}

.control-btn:focus-visible {
  box-shadow:
    0 0 0 2px var(--bg-primary),
    0 0 0 4px var(--text-muted);
}

.control-btn.primary {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--btn-accent);
  color: var(--bg-primary);
  font-size: 0;
}

.control-btn.primary:hover {
  filter: brightness(1.15);
  transform: scale(1.05);
}

.control-btn.primary:active {
  transform: scale(0.97);
}

.control-btn.secondary {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--bg-card);
  color: var(--text-secondary);
}

.control-btn.secondary:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.control-btn.secondary:active:not(:disabled) {
  transform: scale(0.95);
}

.control-btn.secondary.dimmed {
  opacity: 0.35;
  cursor: default;
}

.hint-text {
  font-size: 12px;
  color: var(--text-muted);
  user-select: none;
}

.finish-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px;
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.15s ease;
}

.finish-btn:hover {
  color: var(--color-work);
  border-color: var(--color-work);
  background: var(--color-work-alpha-10);
}

.finish-btn:active {
  transform: scale(0.97);
}

.hint-text kbd {
  display: inline-block;
  padding: 1px 5px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: 4px;
}

/* Fade-in transition for finish button */
.fade-in-enter-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.fade-in-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}

.fade-in-enter-from {
  opacity: 0;
  transform: scale(0.9);
}

.fade-in-leave-to {
  opacity: 0;
  transform: scale(0.9);
}
</style>
