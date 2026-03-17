<script setup lang="ts">
import { computed } from "vue";
import { useTimerStore } from "@/stores/timer";

const timer = useTimerStore();

const accentVar = computed(() => `var(--color-${timer.accentColor})`);

const cycles = computed(() => {
  const items = [];
  for (let i = 1; i <= timer.state.total_cycles; i++) {
    items.push({
      index: i,
      completed: i < timer.state.current_cycle,
      current: i === timer.state.current_cycle,
    });
  }
  return items;
});
</script>

<template>
  <div class="cycle-indicator">
    <div class="cycle-dots">
      <span
        v-for="cycle in cycles"
        :key="cycle.index"
        class="cycle-dot"
        :class="{
          completed: cycle.completed,
          current: cycle.current,
        }"
        :style="cycle.completed || cycle.current ? { '--dot-color': accentVar } : {}"
      />
    </div>
    <span class="cycle-text">
      Pomodoro {{ timer.state.current_cycle }} of {{ timer.state.total_cycles }}
    </span>
  </div>
</template>

<style scoped>
.cycle-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.cycle-dots {
  display: flex;
  align-items: center;
  gap: 8px;
}

.cycle-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--bg-card);
  border: 1.5px solid var(--border-default);
  transition: all 0.3s ease;
}

.cycle-dot.completed {
  background: var(--dot-color);
  border-color: var(--dot-color);
}

.cycle-dot.current {
  width: 10px;
  height: 10px;
  border-color: var(--dot-color);
  background: transparent;
  box-shadow: 0 0 8px color-mix(in srgb, var(--dot-color) 50%, transparent);
}

.cycle-text {
  font-size: 12px;
  color: var(--text-muted);
  letter-spacing: 0.5px;
}
</style>
