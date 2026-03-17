<script setup lang="ts">
import type { DailyStats } from "../../lib/tauri";
import { formatSecs } from "../../composables/useStats";

defineProps<{
  stats: DailyStats;
}>();
</script>

<template>
  <div class="daily-cards">
    <div class="card">
      <div class="card-accent" />
      <div class="card-value">{{ stats.total_sessions }}</div>
      <div class="card-label">Pomodoros</div>
    </div>
    <div class="card">
      <div class="card-accent" />
      <div class="card-value">{{ formatSecs(stats.total_work_secs) }}</div>
      <div class="card-label">Focus Time</div>
    </div>
    <div class="card">
      <div class="card-accent" />
      <div class="card-value">{{ formatSecs(stats.total_break_secs) }}</div>
      <div class="card-label">Break Time</div>
    </div>
  </div>
</template>

<style scoped>
.daily-cards {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.card {
  position: relative;
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  padding: 20px 16px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease;
}

.card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
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

.card-value {
  font-family: var(--font-mono);
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
}

.card-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
</style>
