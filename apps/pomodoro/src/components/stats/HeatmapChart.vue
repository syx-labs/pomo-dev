<script setup lang="ts">
import { computed } from "vue";
import type { HeatmapEntry } from "../../lib/tauri";

const props = defineProps<{
  data: HeatmapEntry[];
  year: number;
}>();

const cellSize = 13;
const gap = 2;
const step = cellSize + gap;
const labelOffsetX = 30;
const labelOffsetY = 18;

const dayLabels = ["Mon", "", "Wed", "", "Fri", "", ""];
const monthLabels = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

const dataMap = computed(() => {
  const map = new Map<string, number>();
  for (const entry of props.data) {
    map.set(entry.date, entry.count);
  }
  return map;
});

interface CellData {
  date: string;
  count: number;
  col: number;
  row: number;
}

const cells = computed<CellData[]>(() => {
  const result: CellData[] = [];
  const startDate = new Date(props.year, 0, 1);
  const endDate = new Date(props.year, 11, 31);

  // Find the Monday of the week containing Jan 1
  const startDay = startDate.getDay();
  const mondayOffset = startDay === 0 ? -6 : 1 - startDay;
  const gridStart = new Date(startDate);
  gridStart.setDate(gridStart.getDate() + mondayOffset);

  const current = new Date(gridStart);
  let col = 0;

  while (current <= endDate || col < 53) {
    for (let row = 0; row < 7; row++) {
      const d = new Date(current);
      d.setDate(d.getDate() + row);
      if (d.getFullYear() === props.year) {
        const dateStr = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
        result.push({
          date: dateStr,
          count: dataMap.value.get(dateStr) ?? 0,
          col,
          row,
        });
      }
    }
    current.setDate(current.getDate() + 7);
    col++;
    if (col >= 53) break;
  }

  return result;
});

const monthPositions = computed(() => {
  const positions: { label: string; x: number }[] = [];
  let lastMonth = -1;
  for (const cell of cells.value) {
    const month = Number.parseInt(cell.date.split("-")[1]) - 1;
    if (month !== lastMonth && cell.row === 0) {
      positions.push({ label: monthLabels[month], x: cell.col });
      lastMonth = month;
    }
  }
  return positions;
});

const svgWidth = computed(() => labelOffsetX + 53 * step);
const svgHeight = labelOffsetY + 7 * step;

function cellColor(count: number): string {
  if (count === 0) return "var(--heatmap-empty)";
  if (count <= 2) return "var(--heatmap-low)";
  if (count <= 5) return "var(--heatmap-mid)";
  return "var(--heatmap-high)";
}
</script>

<template>
  <div class="heatmap-chart">
    <div class="chart-title">{{ year }} Activity</div>
    <div class="chart-container">
      <svg
        :viewBox="`0 0 ${svgWidth} ${svgHeight}`"
        class="heatmap-svg"
        preserveAspectRatio="xMidYMid meet"
      >
        <!-- Month labels -->
        <text
          v-for="mp in monthPositions"
          :key="mp.label + mp.x"
          :x="labelOffsetX + mp.x * step"
          :y="12"
          class="month-label"
        >
          {{ mp.label }}
        </text>

        <!-- Day labels -->
        <text
          v-for="(label, i) in dayLabels"
          :key="'day-' + i"
          :x="labelOffsetX - 6"
          :y="labelOffsetY + i * step + cellSize - 2"
          text-anchor="end"
          class="day-label"
        >
          {{ label }}
        </text>

        <!-- Cells -->
        <rect
          v-for="cell in cells"
          :key="cell.date"
          :x="labelOffsetX + cell.col * step"
          :y="labelOffsetY + cell.row * step"
          :width="cellSize"
          :height="cellSize"
          :rx="2"
          :fill="cellColor(cell.count)"
          stroke="var(--border-subtle)"
          stroke-width="1"
          class="cell"
        >
          <title>{{ cell.date }}: {{ cell.count }} sessions</title>
        </rect>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.heatmap-chart {
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  padding: 20px;
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.chart-container {
  width: 100%;
  overflow-x: auto;
}

.heatmap-svg {
  width: 100%;
  height: auto;
  display: block;
}

.month-label {
  font-size: 10px;
  fill: var(--text-muted);
  font-family: var(--font-sans);
}

.day-label {
  font-size: 9px;
  fill: var(--text-muted);
  font-family: var(--font-sans);
}

.cell {
  transition: opacity 0.15s ease;
}

.cell:hover {
  opacity: 0.8;
  cursor: pointer;
}
</style>
