<script setup lang="ts">
import { computed, ref } from "vue";
import type { Goal } from "../../lib/tauri";
import { setGoal } from "../../lib/tauri";

const props = defineProps<{
  goals: Goal[];
  currentWeekSessions: number;
}>();

const emit = defineEmits<{
  updated: [];
}>();

const editing = ref(false);
const editTarget = ref(0);

const weeklyGoal = computed(() => props.goals.find((g) => g.goal_type === "weekly_sessions"));

const target = computed(() => weeklyGoal.value?.target ?? 20);
const progress = computed(() => Math.min(props.currentWeekSessions / target.value, 1));
const percentage = computed(() => Math.round(progress.value * 100));

const SIZE = 120;
const STROKE = 8;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
const dashOffset = computed(() => CIRCUMFERENCE * (1 - progress.value));

const ringColor = computed(() => {
  if (progress.value >= 1) return "var(--color-work)";
  if (progress.value >= 0.5) return "var(--color-priority-medium)";
  return "var(--color-priority-high)";
});

function startEdit() {
  editTarget.value = target.value;
  editing.value = true;
}

async function saveTarget() {
  if (editTarget.value > 0) {
    await setGoal("weekly_sessions", editTarget.value);
    emit("updated");
  }
  editing.value = false;
}
</script>

<template>
  <div class="goals-card">
    <div class="card-accent" />
    <div class="goals-title">Weekly Goal</div>

    <div class="goals-content">
      <div class="ring-container">
        <svg :width="SIZE" :height="SIZE" :viewBox="`0 0 ${SIZE} ${SIZE}`">
          <circle
            :cx="SIZE / 2"
            :cy="SIZE / 2"
            :r="RADIUS"
            fill="none"
            stroke="var(--bg-hover)"
            :stroke-width="STROKE"
          />
          <circle
            :cx="SIZE / 2"
            :cy="SIZE / 2"
            :r="RADIUS"
            fill="none"
            :stroke="ringColor"
            :stroke-width="STROKE"
            stroke-linecap="round"
            :stroke-dasharray="CIRCUMFERENCE"
            :stroke-dashoffset="dashOffset"
            class="progress-ring"
            transform="rotate(-90 60 60)"
          />
        </svg>
        <div class="ring-center">
          <span class="ring-pct">{{ percentage }}%</span>
        </div>
      </div>

      <div class="goals-info">
        <div class="sessions-count">
          <span class="count-current">{{ currentWeekSessions }}</span>
          <span class="count-sep">/</span>
          <span v-if="!editing" class="count-target" @click="startEdit">
            {{ target }}
          </span>
          <input
            v-else
            v-model.number="editTarget"
            type="number"
            min="1"
            max="999"
            class="target-input"
            @blur="saveTarget"
            @keydown.enter="saveTarget"
          />
        </div>
        <div class="sessions-label">sessions this week</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.goals-card {
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

.goals-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 16px;
}

.goals-content {
  display: flex;
  align-items: center;
  gap: 20px;
}

.ring-container {
  position: relative;
  flex-shrink: 0;
  width: 120px;
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ring-container svg {
  position: absolute;
}

.progress-ring {
  transition:
    stroke-dashoffset 0.4s cubic-bezier(0.4, 0, 0.2, 1),
    stroke 0.3s ease;
}

.ring-center {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.ring-pct {
  font-family: var(--font-mono);
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
}

.goals-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sessions-count {
  display: flex;
  align-items: baseline;
  gap: 2px;
}

.count-current {
  font-family: var(--font-mono);
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
}

.count-sep {
  font-family: var(--font-mono);
  font-size: 20px;
  color: var(--text-muted);
  margin: 0 2px;
}

.count-target {
  font-family: var(--font-mono);
  font-size: 20px;
  font-weight: 600;
  color: var(--text-secondary);
  cursor: pointer;
  border-bottom: 1px dashed var(--border-default);
  transition: color 0.15s ease;
}

.count-target:hover {
  color: var(--text-primary);
}

.target-input {
  width: 48px;
  font-family: var(--font-mono);
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: 0 4px;
  outline: none;
}

.target-input:focus {
  border-color: var(--color-work);
}

.sessions-label {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
