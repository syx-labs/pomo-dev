<script setup lang="ts">
import { computed, ref } from "vue";
import type { HourlyStats } from "../../lib/tauri";

const props = defineProps<{
  data: HourlyStats[];
}>();

const hoveredHour = ref<number | null>(null);

const hours = computed(() => {
  // Show hours 6-23
  const range: { hour: number; count: number }[] = [];
  for (let h = 6; h <= 23; h++) {
    const entry = props.data.find((d) => d.hour === h);
    range.push({ hour: h, count: entry?.count ?? 0 });
  }
  return range;
});

const maxCount = computed(() => Math.max(...hours.value.map((h) => h.count), 1));

const top3Hours = computed(() => {
  const sorted = [...hours.value].sort((a, b) => b.count - a.count);
  const top = sorted.slice(0, 3).filter((h) => h.count > 0);
  return new Set(top.map((h) => h.hour));
});

const hasData = computed(() => hours.value.some((h) => h.count > 0));

const chartWidth = 340;
const chartHeight = 120;
const barGap = 3;
const numBars = 18; // hours 6-23
const barWidth = (chartWidth - barGap * (numBars + 1)) / numBars;

function barHeight(count: number): number {
  if (maxCount.value === 0) return 3;
  return Math.max((count / maxCount.value) * (chartHeight - 20), 3);
}

function barX(index: number): number {
  return barGap + index * (barWidth + barGap);
}

function barY(count: number): number {
  return chartHeight - barHeight(count);
}

function barFill(hour: number): string {
  return top3Hours.value.has(hour) ? "var(--color-work)" : "var(--bg-hover)";
}

function formatHour(hour: number): string {
  if (hour === 0 || hour === 12) return "12";
  return String(hour > 12 ? hour - 12 : hour);
}
</script>

<template>
  <div class="time-of-day-chart">
    <div class="chart-header">
      <div class="chart-title">Time of Day</div>
      <div v-if="top3Hours.size > 0" class="golden-badge">Golden hours</div>
    </div>

    <div v-if="!hasData" class="empty-state">
      <span class="empty-text">No data yet</span>
    </div>

    <div v-else class="chart-container">
      <svg
        :viewBox="`0 0 ${chartWidth} ${chartHeight + 28}`"
        class="chart-svg"
        preserveAspectRatio="xMidYMid meet"
      >
        <!-- Bars -->
        <g v-for="(h, i) in hours" :key="h.hour">
          <rect
            :x="barX(i)"
            :y="barY(h.count)"
            :width="barWidth"
            :height="barHeight(h.count)"
            :rx="3"
            :fill="barFill(h.hour)"
            :opacity="hoveredHour === h.hour ? 1 : 0.85"
            class="bar"
            @mouseenter="hoveredHour = h.hour"
            @mouseleave="hoveredHour = null"
          />

          <!-- Tooltip -->
          <g v-if="hoveredHour === h.hour && h.count > 0">
            <rect
              :x="barX(i) + barWidth / 2 - 14"
              :y="barY(h.count) - 22"
              width="28"
              height="18"
              rx="4"
              fill="var(--bg-hover)"
            />
            <text
              :x="barX(i) + barWidth / 2"
              :y="barY(h.count) - 9"
              text-anchor="middle"
              class="tooltip-text"
            >
              {{ h.count }}
            </text>
          </g>

          <!-- Hour labels (show every 2) -->
          <text
            v-if="i % 2 === 0"
            :x="barX(i) + barWidth / 2"
            :y="chartHeight + 16"
            text-anchor="middle"
            class="hour-label"
          >
            {{ formatHour(h.hour) }}
          </text>
        </g>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.time-of-day-chart {
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  padding: 20px;
}

.chart-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.golden-badge {
  font-size: 11px;
  font-weight: 500;
  color: var(--color-work);
  padding: 2px 8px;
  background: color-mix(in srgb, var(--color-work) 12%, transparent);
  border-radius: var(--radius-sm);
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 0;
}

.empty-text {
  font-size: 13px;
  color: var(--text-muted);
  font-style: italic;
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

.hour-label {
  font-size: 10px;
  fill: var(--text-muted);
  font-family: var(--font-sans);
}

.tooltip-text {
  font-size: 10px;
  fill: var(--text-primary);
  font-family: var(--font-mono);
  font-weight: 600;
}
</style>
