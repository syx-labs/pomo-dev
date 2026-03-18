<script setup lang="ts">
import { computed } from "vue";
import type { ProjectStats } from "../../lib/tauri";
import { formatSecs } from "../../composables/useStats";

const props = defineProps<{
  data: ProjectStats[];
}>();

const SIZE = 160;
const STROKE = 24;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

function hashColor(name: string): string {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = Math.abs(hash) % 360;
  return `hsl(${hue}, 55%, 55%)`;
}

const totalSecs = computed(() => props.data.reduce((sum, p) => sum + p.total_secs, 0));

const segments = computed(() => {
  if (totalSecs.value === 0) return [];
  let offset = 0;
  return props.data.map((p) => {
    const ratio = p.total_secs / totalSecs.value;
    const dashArray = CIRCUMFERENCE * ratio;
    const dashOffset = -offset;
    offset += dashArray;
    return {
      project: p.project,
      sessions: p.sessions,
      secs: p.total_secs,
      percentage: Math.round(ratio * 100),
      color: hashColor(p.project),
      dashArray: `${dashArray} ${CIRCUMFERENCE - dashArray}`,
      dashOffset,
    };
  });
});

const hasData = computed(() => props.data.length > 0);
</script>

<template>
  <div class="project-breakdown">
    <div class="chart-title">Projects</div>

    <div v-if="!hasData" class="empty-state">
      <span class="empty-text">No project data yet</span>
    </div>

    <div v-else class="breakdown-content">
      <div class="donut-container">
        <svg :width="SIZE" :height="SIZE" :viewBox="`0 0 ${SIZE} ${SIZE}`">
          <circle
            v-for="(seg, i) in segments"
            :key="i"
            :cx="SIZE / 2"
            :cy="SIZE / 2"
            :r="RADIUS"
            fill="none"
            :stroke="seg.color"
            :stroke-width="STROKE"
            :stroke-dasharray="seg.dashArray"
            :stroke-dashoffset="seg.dashOffset"
            class="donut-segment"
            transform="rotate(-90 80 80)"
          />
        </svg>
      </div>

      <div class="legend">
        <div v-for="seg in segments" :key="seg.project" class="legend-item">
          <span class="legend-dot" :style="{ background: seg.color }" />
          <span class="legend-name">{{ seg.project || "No project" }}</span>
          <span class="legend-pct">{{ seg.percentage }}%</span>
          <span class="legend-time">{{ formatSecs(seg.secs) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.project-breakdown {
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

.breakdown-content {
  display: flex;
  align-items: center;
  gap: 20px;
}

.donut-container {
  flex-shrink: 0;
}

.donut-segment {
  transition: opacity 0.2s ease;
}

.donut-segment:hover {
  opacity: 0.75;
}

.legend {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.legend-name {
  color: var(--text-primary);
  font-weight: 500;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.legend-pct {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-weight: 600;
  flex-shrink: 0;
}

.legend-time {
  color: var(--text-muted);
  font-family: var(--font-mono);
  flex-shrink: 0;
}
</style>
