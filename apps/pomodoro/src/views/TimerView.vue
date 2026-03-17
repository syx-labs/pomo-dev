<script setup lang="ts">
import { computed } from "vue";
import { useTimerStore } from "@/stores/timer";
import { useSoundStore } from "@/stores/sound";
import CycleIndicator from "@/components/timer/CycleIndicator.vue";
import TimerRing from "@/components/timer/TimerRing.vue";
import TimerControls from "@/components/timer/TimerControls.vue";
import DailyBriefing from "@/components/ai/DailyBriefing.vue";

const timer = useTimerStore();
const sound = useSoundStore();

const accentVar = computed(() => `var(--color-${timer.accentColor})`);
const accentDimVar = computed(() => `var(--color-${timer.accentColor}-dim)`);

const soundIndicatorLabel = computed(() => {
  if (!sound.isAnySoundPlaying) return "";
  const preset = sound.presets.find((p) => p.id === sound.activePresetId);
  if (preset) return preset.name;
  const names = sound.activeLayers.map((l) => l.name);
  return names.length === 1 ? names[0].charAt(0).toUpperCase() + names[0].slice(1) : "Custom mix";
});
</script>

<template>
  <div
    class="timer-view"
    :style="{
      '--accent': accentVar,
      '--accent-dim': accentDimVar,
    }"
  >
    <!-- Subtle background glow -->
    <div class="bg-glow" :class="{ active: timer.isRunning }" />

    <div class="timer-content">
      <CycleIndicator />

      <TimerRing />

      <TimerControls />

      <!-- Sound indicator -->
      <Transition name="fade-slide">
        <div v-if="sound.isAnySoundPlaying" class="sound-indicator">
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
            <path d="M3 18v-6a9 9 0 0 1 18 0v6" />
            <path
              d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"
            />
          </svg>
          <span>{{ soundIndicatorLabel }}</span>
        </div>
      </Transition>

      <DailyBriefing />

      <!-- Linked task -->
      <Transition name="fade-slide">
        <div v-if="timer.linkedTaskTitle" class="linked-task">
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
            <circle cx="12" cy="12" r="10" />
            <path d="m9 12 2 2 4-4" />
          </svg>
          <span>{{ timer.linkedTaskTitle }}</span>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.timer-view {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: calc(100vh - var(--titlebar-height));
  overflow: hidden;
}

.bg-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -55%);
  width: 400px;
  height: 400px;
  border-radius: 50%;
  background: radial-gradient(
    circle,
    color-mix(in srgb, var(--accent-dim) 20%, transparent) 0%,
    transparent 70%
  );
  opacity: 0;
  transition: opacity 0.8s ease;
  pointer-events: none;
}

.bg-glow.active {
  opacity: 1;
}

.timer-content {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 32px;
}

.sound-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
}

.sound-indicator svg {
  color: var(--accent);
  flex-shrink: 0;
}

.linked-task {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--text-secondary);
  max-width: 280px;
  overflow: hidden;
}

.linked-task svg {
  flex-shrink: 0;
  color: var(--accent);
}

.linked-task span {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Fade-slide transition for conditional elements */
.fade-slide-enter-active {
  transition:
    opacity 0.25s ease,
    transform 0.25s ease;
}

.fade-slide-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
