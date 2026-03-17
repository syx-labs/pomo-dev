<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useStats } from "../composables/useStats";
import DailyCards from "../components/stats/DailyCards.vue";
import WeeklyChart from "../components/stats/WeeklyChart.vue";
import ProjectBreakdown from "../components/stats/ProjectBreakdown.vue";

const {
  daily,
  weekly,
  loading,
  displayDate,
  isToday,
  dateString,
  fetchAll,
  goToPreviousDay,
  goToNextDay,
  goToToday,
} = useStats();

onMounted(fetchAll);

watch(dateString, () => {
  fetchAll();
});
</script>

<template>
  <div class="stats-view">
    <header class="stats-header">
      <h1 class="stats-title">Statistics</h1>
      <div class="date-nav">
        <button class="nav-btn" @click="goToPreviousDay" aria-label="Previous day">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path
              d="M10 12L6 8l4-4"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button class="date-label" :class="{ 'date-label--today': isToday }" @click="goToToday">
          {{ displayDate }}
        </button>
        <button class="nav-btn" @click="goToNextDay" aria-label="Next day">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path
              d="M6 4l4 4-4 4"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
      </div>
    </header>

    <div v-if="loading && !daily" class="loading-state">
      <div class="loading-spinner" />
      <span class="loading-text">Loading stats...</span>
    </div>

    <div v-else class="stats-content">
      <DailyCards v-if="daily" :stats="daily" />

      <WeeklyChart v-if="weekly" :stats="weekly" />

      <ProjectBreakdown :data="[]" />
    </div>
  </div>
</template>

<style scoped>
.stats-view {
  height: 100%;
  min-height: calc(100vh - var(--titlebar-height));
  overflow-y: auto;
  padding: 24px 20px 40px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.stats-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.stats-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.date-nav {
  display: flex;
  align-items: center;
  gap: 4px;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

.nav-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-btn:active {
  transform: scale(0.9);
}

.date-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  padding: 4px 10px;
  border: none;
  background: transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-family: var(--font-sans);
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

.date-label:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.date-label--today {
  color: var(--color-work);
}

.loading-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.loading-text {
  font-size: 14px;
  color: var(--text-muted);
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
</style>
