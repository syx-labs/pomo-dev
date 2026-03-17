<script setup lang="ts">
import { computed } from "vue";
import { useTimerStore } from "@/stores/timer";

const timer = useTimerStore();

const SIZE = 240;
const STROKE = 6;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

const dashOffset = computed(() => {
  return CIRCUMFERENCE * (1 - timer.progress);
});

const accentVar = computed(() => `var(--color-${timer.accentColor})`);
const accentDimVar = computed(() => `var(--color-${timer.accentColor}-dim)`);
</script>

<template>
  <div class="timer-ring-wrapper" :class="{ 'is-running': timer.isRunning }">
    <!-- Glow backdrop when running -->
    <div
      class="timer-glow"
      :class="{ active: timer.isRunning }"
      :style="{ '--glow-color': accentVar }"
    />

    <svg class="timer-svg" :width="SIZE" :height="SIZE" :viewBox="`0 0 ${SIZE} ${SIZE}`">
      <!-- Background ring -->
      <circle
        class="ring-bg"
        :cx="SIZE / 2"
        :cy="SIZE / 2"
        :r="RADIUS"
        fill="none"
        :stroke-width="STROKE"
      />

      <!-- Progress ring -->
      <circle
        class="ring-progress"
        :cx="SIZE / 2"
        :cy="SIZE / 2"
        :r="RADIUS"
        fill="none"
        :stroke-width="STROKE"
        stroke-linecap="round"
        :stroke-dasharray="CIRCUMFERENCE"
        :stroke-dashoffset="dashOffset"
        :style="{ stroke: accentVar }"
      />
    </svg>

    <!-- Center content -->
    <div class="timer-center" aria-live="polite">
      <span class="timer-digits">{{ timer.displayTime }}</span>
      <span class="timer-label" :style="{ color: accentVar }">{{ timer.sessionLabel }}</span>
    </div>
  </div>
</template>

<style scoped>
.timer-ring-wrapper {
  position: relative;
  width: 240px;
  height: 240px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.timer-glow {
  position: absolute;
  inset: -24px;
  border-radius: 50%;
  background: radial-gradient(
    circle,
    color-mix(in srgb, var(--glow-color) 15%, transparent) 0%,
    transparent 70%
  );
  opacity: 0;
  transition: opacity 0.6s ease;
  pointer-events: none;
}

.timer-glow.active {
  opacity: 1;
}

.timer-svg {
  position: absolute;
  transform: rotate(-90deg);
}

.ring-bg {
  stroke: var(--bg-card);
}

.ring-progress {
  transition:
    stroke-dashoffset 0.4s cubic-bezier(0.4, 0, 0.2, 1),
    stroke 0.3s ease;
  filter: drop-shadow(0 0 6px color-mix(in srgb, currentColor 40%, transparent));
}

.timer-center {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.timer-digits {
  font-family: var(--font-mono);
  font-size: 52px;
  font-weight: 600;
  letter-spacing: -1px;
  color: var(--text-primary);
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.timer-label {
  font-size: 13px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  transition: color 0.3s ease;
}

/* Subtle breathing effect on digits when timer is running */
.is-running .timer-digits {
  animation: digits-breathe 2s ease-in-out infinite;
}

@keyframes digits-breathe {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.82;
  }
}
</style>
