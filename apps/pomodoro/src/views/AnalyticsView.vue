<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useStats, type SelectedPeriod } from "../composables/useStats";
import HeatmapChart from "../components/stats/HeatmapChart.vue";
import StreakCard from "../components/stats/StreakCard.vue";
import GoalsCard from "../components/stats/GoalsCard.vue";
import ProjectBreakdown from "../components/stats/ProjectBreakdown.vue";
import TimeOfDayChart from "../components/stats/TimeOfDayChart.vue";
import DailyCards from "../components/stats/DailyCards.vue";
import WeeklyChart from "../components/stats/WeeklyChart.vue";
import WeeklyReport from "../components/ai/WeeklyReport.vue";

const {
  daily,
  weekly,
  heatmapData,
  streakData,
  hourlyStats,
  projectStats,
  goals,
  selectedPeriod,
  loading,
  displayDate,
  isToday,
  dateString,
  fetchAnalytics,
  fetchHourly,
  fetchProjects,
  fetchGoals,
  goToPreviousDay,
  goToNextDay,
  goToToday,
} = useStats();

const currentYear = new Date().getFullYear();

const periods: { label: string; value: SelectedPeriod }[] = [
  { label: "Last Week", value: "week" },
  { label: "Month", value: "month" },
  { label: "3 Months", value: "3months" },
  { label: "Year", value: "year" },
];

function selectPeriod(period: SelectedPeriod) {
  selectedPeriod.value = period;
}

async function onGoalUpdated() {
  await fetchGoals();
}

onMounted(fetchAnalytics);

watch(dateString, () => {
  fetchAnalytics();
});

watch(selectedPeriod, (period) => {
  fetchHourly(period);
  fetchProjects(period);
});
</script>

<template>
  <div class="analytics-view">
    <header class="analytics-header">
      <h1 class="analytics-title">Analytics</h1>
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

    <!-- Loading skeleton -->
    <div v-if="loading && !daily" class="loading-state">
      <div class="skeleton skeleton-heatmap" />
      <div class="skeleton-row">
        <div class="skeleton skeleton-card" />
        <div class="skeleton skeleton-card" />
      </div>
    </div>

    <div v-else class="analytics-content">
      <!-- Row 1: Heatmap -->
      <HeatmapChart :data="heatmapData" :year="currentYear" />

      <!-- Row 2: Streak + Goals -->
      <div class="row-2">
        <StreakCard :current="streakData?.current ?? 0" :best="streakData?.best ?? 0" />
        <GoalsCard
          :goals="goals"
          :current-week-sessions="weekly?.total_sessions ?? 0"
          @updated="onGoalUpdated"
        />
      </div>

      <!-- AI Weekly Report -->
      <WeeklyReport />

      <!-- Period selector for breakdowns -->
      <div class="period-selector">
        <button
          v-for="p in periods"
          :key="p.value"
          class="period-btn"
          :class="{ 'period-btn--active': selectedPeriod === p.value }"
          @click="selectPeriod(p.value)"
        >
          {{ p.label }}
        </button>
      </div>

      <!-- Row 3: Projects + Time of Day -->
      <div class="row-3">
        <ProjectBreakdown :data="projectStats" />
        <TimeOfDayChart :data="hourlyStats" />
      </div>

      <!-- Row 4: Daily cards -->
      <DailyCards
        :stats="daily ?? { date: '', total_sessions: 0, total_work_secs: 0, total_break_secs: 0 }"
      />

      <!-- Row 5: Weekly chart -->
      <WeeklyChart v-if="weekly" :stats="weekly" />
    </div>
  </div>
</template>

<style scoped>
.analytics-view {
  height: 100%;
  min-height: calc(100vh - var(--titlebar-height));
  overflow-y: auto;
  padding: 24px 20px 40px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.analytics-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.analytics-title {
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

/* Loading skeleton */
.loading-state {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.skeleton {
  background: var(--bg-hover);
  border-radius: var(--radius-lg);
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}

.skeleton-heatmap {
  height: 160px;
}

.skeleton-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.skeleton-card {
  height: 140px;
}

@keyframes skeleton-pulse {
  0%,
  100% {
    opacity: 0.5;
  }
  50% {
    opacity: 0.3;
  }
}

/* Content layout */
.analytics-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.row-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.row-3 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

/* Period selector */
.period-selector {
  display: flex;
  gap: 6px;
}

.period-btn {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  padding: 4px 12px;
  border: 1px solid var(--border-default);
  background: transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-family: var(--font-sans);
  transition: all 0.15s ease;
}

.period-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}

.period-btn:active {
  transform: scale(0.95);
}

.period-btn--active {
  color: var(--text-primary);
  background: var(--bg-hover);
  border-color: var(--color-work);
}
</style>
