<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import type { Task } from "@/lib/tauri";
import { useTasks } from "@/composables/useTasks";
import { useTasksStore } from "@/stores/tasks";

const props = defineProps<{
  task: Task;
}>();

const emit = defineEmits<{
  edit: [task: Task];
  delete: [task: Task];
  "start-pomodoro": [task: Task];
}>();

const { getPriorityLabel, getPriorityColor, cycleTaskStatus } = useTasks();
const store = useTasksStore();

const completedPomos = ref(0);

const isDone = computed(() => props.task.status === "done");
const showPriority = computed(() => props.task.priority > 0);
const isSaving = computed(() => store.savingTaskId === props.task.id);
const isDeleting = computed(() => store.deletingTaskId === props.task.id);
const isBusy = computed(() => isSaving.value || isDeleting.value);

onMounted(async () => {
  try {
    completedPomos.value = await store.fetchSessionCount(props.task.id);
  } catch {
    completedPomos.value = 0;
  }
});

async function onCheckboxClick() {
  await cycleTaskStatus(props.task);
}
</script>

<template>
  <div
    class="task-card"
    :class="{ 'task-card--done': isDone, 'task-card--busy': isBusy }"
    @dblclick="!isBusy && emit('edit', task)"
  >
    <button
      class="task-checkbox"
      :class="{ 'task-checkbox--checked': isDone }"
      @click="onCheckboxClick"
      :disabled="isBusy"
      aria-label="Toggle task status"
    >
      <svg v-if="isDone" width="14" height="14" viewBox="0 0 14 14" fill="none">
        <path
          d="M3 7.5L5.5 10L11 4"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>

    <div class="task-content">
      <span class="task-title">{{ task.title }}</span>
      <div class="task-meta">
        <span v-if="showPriority" class="task-badge task-badge--priority">
          <span
            class="priority-dot"
            :style="{ backgroundColor: getPriorityColor(task.priority) }"
          />
          {{ getPriorityLabel(task.priority) }}
        </span>
        <span v-if="task.project" class="task-badge task-badge--project">
          {{ task.project }}
        </span>
        <span v-if="task.estimated_pomos > 0" class="task-badge task-badge--pomos">
          {{ completedPomos }}/{{ task.estimated_pomos }}
        </span>
      </div>
    </div>

    <div class="task-actions">
      <button
        class="task-action-btn task-action-btn--play"
        @click.stop="emit('start-pomodoro', task)"
        aria-label="Start pomodoro"
        :disabled="isDone || isBusy"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M5 3L12 8L5 13V3Z" fill="currentColor" />
        </svg>
      </button>
      <button
        class="task-action-btn task-action-btn--delete"
        @click.stop="emit('delete', task)"
        :disabled="isBusy"
        aria-label="Delete task"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path
            d="M4.5 5V13H11.5V5M2.5 5H13.5M6 5V3.5C6 3.224 6.224 3 6.5 3H9.5C9.776 3 10 3.224 10 3.5V5M8 7.5V10.5M6 7.5V10.5M10 7.5V10.5"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.task-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 14px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  cursor: default;
  transition:
    background 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease;
}

.task-card:hover {
  background: var(--bg-card);
  border-color: var(--border-default);
  box-shadow: var(--card-hover-shadow), var(--card-hover-inset);
}

.task-card--busy {
  opacity: 0.5;
  pointer-events: none;
}

.task-card--done .task-title {
  text-decoration: line-through;
  color: var(--text-muted);
}

.task-checkbox {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 2px solid var(--border-default);
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--text-primary);
  padding: 0;
  margin-top: 1px;
  transition:
    border-color 0.15s ease,
    background 0.15s ease,
    transform 0.15s ease,
    box-shadow 0.15s ease;
}

.task-checkbox:hover {
  border-color: var(--color-work);
}

.task-checkbox:active {
  transform: scale(0.85);
}

.task-checkbox--checked {
  background: var(--color-work);
  border-color: var(--color-work);
  animation: checkbox-pop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes checkbox-pop {
  0% {
    transform: scale(0.8);
  }
  50% {
    transform: scale(1.15);
  }
  100% {
    transform: scale(1);
  }
}

.task-content {
  flex: 1;
  min-width: 0;
}

.task-title {
  display: block;
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 22px;
}

.task-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 6px;
}

.task-meta:empty {
  display: none;
}

.task-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 999px;
  font-family: var(--font-sans);
  font-size: 11px;
  font-weight: 500;
  line-height: 1;
  background: var(--bg-secondary);
  color: var(--text-secondary);
}

.priority-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.task-badge--pomos {
  font-family: var(--font-mono);
  letter-spacing: 0.02em;
}

.task-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.task-card:hover .task-actions {
  opacity: 1;
}

.task-action-btn {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  border: none;
  background: transparent;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
  transition:
    background 0.15s ease,
    color 0.15s ease,
    transform 0.1s ease;
}

.task-action-btn:hover {
  transform: scale(1.1);
}

.task-action-btn--play {
  color: var(--text-secondary);
}

.task-action-btn--play:hover {
  background: var(--color-work-alpha-15);
  color: var(--color-work);
}

.task-action-btn--delete:hover {
  background: var(--color-error-alpha-12);
  color: var(--color-priority-high);
}

.task-action-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
  transform: none;
}
</style>
