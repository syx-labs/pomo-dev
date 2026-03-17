<script setup lang="ts">
defineProps<{
  current: number;
  best: number;
}>();
</script>

<template>
  <div class="streak-card">
    <div class="card-accent" />
    <div class="streak-title">Streak</div>
    <div v-if="current === 0 && best === 0" class="streak-empty">
      Complete your first pomodoro to start a streak!
    </div>
    <div v-else class="streak-row">
      <div class="streak-item">
        <div class="streak-icon-row">
          <!-- Fire icon -->
          <svg
            v-if="current >= 3"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            class="fire-icon"
          >
            <path d="M12 2c0 4-4 6-4 10a4 4 0 0 0 8 0c0-4-4-6-4-10Z" fill="#f59e0b" opacity="0.8" />
            <path d="M12 8c0 2-2 3-2 5a2 2 0 0 0 4 0c0-2-2-3-2-5Z" fill="#fbbf24" />
          </svg>
        </div>
        <div class="streak-value" :class="{ pulse: current >= 7 }">{{ current }}</div>
        <div class="streak-label">Current</div>
      </div>
      <div class="streak-divider" />
      <div class="streak-item">
        <div class="streak-value streak-value--best">{{ best }}</div>
        <div class="streak-label">Best</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.streak-card {
  position: relative;
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  padding: 20px;
  overflow: hidden;
}

.card-accent {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: var(--color-work);
  border-radius: var(--radius-lg) var(--radius-lg) 0 0;
}

.streak-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 16px;
}

.streak-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 24px;
}

.streak-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.streak-icon-row {
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.fire-icon {
  animation: flicker 1.5s ease-in-out infinite alternate;
}

@keyframes flicker {
  0% {
    opacity: 0.8;
    transform: scale(1);
  }
  100% {
    opacity: 1;
    transform: scale(1.1);
  }
}

.streak-value {
  font-family: var(--font-mono);
  font-size: 36px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
}

.streak-value--best {
  color: var(--text-secondary);
}

.streak-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.streak-empty {
  font-size: 13px;
  color: var(--text-muted);
  text-align: center;
  padding: 8px 0;
}

.streak-divider {
  width: 1px;
  height: 48px;
  background: var(--border-default);
}

.pulse {
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}
</style>
