<script setup lang="ts">
import { computed, ref } from "vue";
import type { WeeklyStats } from "../../lib/tauri";
import { formatSecs } from "../../composables/useStats";

const props = defineProps<{
  stats: WeeklyStats;
}>();

const dayLabels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const hoveredIndex = ref<number | null>(null);

const todayDayIndex = computed(() => {
  const day = new Date().getDay();
  // Convert: 0=Sun->6, 1=Mon->0, ... 6=Sat->5
  return day === 0 ? 6 : day - 1;
});

const maxSessions = computed(() => {
  if (!props.stats.days.length) return 0;
  return Math.max(...props.stats.days.map((d) => d.total_sessions), 1);
});

const hasData = computed(() => props.stats.days.some((d) => d.total_sessions > 0));

const chartWidth = 320;
const chartHeight = 140;
const barGap = 10;
const barWidth = computed(() => (chartWidth - barGap * (dayLabels.length + 1)) / dayLabels.length);

function barHeight(sessions: number): number {
  if (maxSessions.value === 0) return 4;
  return Math.max((sessions / maxSessions.value) * (chartHeight - 20), 4);
}

function barX(index: number): number {
  return barGap + index * (barWidth.value + barGap);
}

function barY(sessions: number): number {
  return chartHeight - barHeight(sessions);
}

function barOpacity(index: number): number {
  if (!hasData.value) return 0.2;
  if (index === todayDayIndex.value) return 1;
  return 0.55;
}

function sessionsForDay(index: number): number {
  return props.stats.days[index]?.total_sessions ?? 0;
}
</script>

<template>
  <div class="weekly-chart">
    <div class="chart-title">This Week</div>
    <div class="chart-container">
      <svg
        :viewBox="`0 0 ${chartWidth} ${chartHeight + 28}`"
        class="chart-svg"
        preserveAspectRatio="xMidYMid meet"
      >
        <!-- Empty state text -->
        <text
          v-if="!hasData"
          :x="chartWidth / 2"
          :y="chartHeight / 2"
          text-anchor="middle"
          class="empty-text"
        >
          No data yet
        </text>

        <!-- Bars -->
        <g v-for="(_, i) in dayLabels" :key="i">
          <rect
            :x="barX(i)"
            :y="barY(sessionsForDay(i))"
            :width="barWidth"
            :height="barHeight(sessionsForDay(i))"
            :rx="4"
            fill="var(--color-work)"
            :opacity="hoveredIndex === i ? 1 : barOpacity(i)"
            class="bar"
            @mouseenter="hoveredIndex = i"
            @mouseleave="hoveredIndex = null"
          />

          <!-- Tooltip on hover -->
          <g v-if="hoveredIndex === i">
            <rect
              :x="barX(i) + barWidth / 2 - 18"
              :y="barY(sessionsForDay(i)) - 26"
              width="36"
              height="20"
              rx="4"
              fill="var(--bg-hover)"
            />
            <text
              :x="barX(i) + barWidth / 2"
              :y="barY(sessionsForDay(i)) - 12"
              text-anchor="middle"
              class="tooltip-text"
            >
              {{ sessionsForDay(i) }}
            </text>
          </g>

          <!-- Day labels -->
          <text
            :x="barX(i) + barWidth / 2"
            :y="chartHeight + 18"
            text-anchor="middle"
            class="day-label"
            :class="{ 'day-label--today': i === todayDayIndex }"
          >
            {{ dayLabels[i] }}
          </text>
        </g>
      </svg>
    </div>

    <div class="chart-summary">
      <div class="summary-item">
        <span class="summary-value">{{ stats.total_sessions }}</span>
        <span class="summary-label">sessions</span>
      </div>
      <div class="summary-item">
        <span class="summary-value">{{ formatSecs(stats.total_work_secs) }}</span>
        <span class="summary-label">focus time</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.weekly-chart {
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  padding: 20px;
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 16px;
}

.chart-container {
  width: 100%;
}

.chart-svg {
  width: 100%;
  height: auto;
}

.bar {
  cursor: pointer;
  transition: opacity 0.15s ease;
}

.day-label {
  font-size: 11px;
  fill: var(--text-muted);
  font-family: var(--font-sans);
}

.day-label--today {
  fill: var(--text-primary);
  font-weight: 600;
}

.tooltip-text {
  font-size: 11px;
  fill: var(--text-primary);
  font-family: var(--font-mono);
  font-weight: 600;
}

.empty-text {
  font-size: 14px;
  fill: var(--text-muted);
  font-family: var(--font-sans);
}

.chart-summary {
  display: flex;
  gap: 24px;
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid var(--border-default);
}

.summary-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.summary-value {
  font-family: var(--font-mono);
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
}

.summary-label {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
