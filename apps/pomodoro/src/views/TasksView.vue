<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useTasksStore } from "@/stores/tasks";
import { useTasks } from "@/composables/useTasks";
import TaskFilters from "@/components/tasks/TaskFilters.vue";
import TaskList from "@/components/tasks/TaskList.vue";
import TaskForm from "@/components/tasks/TaskForm.vue";
import type { CreateTaskInput, UpdateTaskInput } from "@/lib/tauri";

const store = useTasksStore();
const { filteredTasks, loading } = useTasks();

const taskListRef = ref<InstanceType<typeof TaskList> | null>(null);
const fabFormVisible = ref(false);

const taskCount = computed(() => filteredTasks.value.length);
const totalCount = computed(() => store.tasks.length);

onMounted(async () => {
  await store.fetchTasks();
});

function openAddTask() {
  if (taskListRef.value) {
    taskListRef.value.openCreate();
  } else {
    fabFormVisible.value = true;
  }
}

async function handleFabSave(input: CreateTaskInput | UpdateTaskInput) {
  if (!("id" in input)) {
    await store.addTask(input as CreateTaskInput);
  }
  fabFormVisible.value = false;
}
</script>

<template>
  <div class="tasks-view">
    <header class="tasks-header">
      <div class="tasks-header-left">
        <h1 class="tasks-title">Tasks</h1>
        <span class="tasks-count">{{ taskCount }}</span>
      </div>
    </header>

    <TaskFilters class="tasks-filters" />

    <div v-if="loading" class="tasks-loading">
      <div class="loading-spinner" />
      <span class="loading-text">Loading tasks...</span>
    </div>

    <div v-else class="tasks-body">
      <TaskList ref="taskListRef" :tasks="filteredTasks" />
    </div>

    <button
      v-if="totalCount > 0 || !loading"
      class="fab"
      @click="openAddTask"
      aria-label="Add task"
      type="button"
    >
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
        <path
          d="M10 4V16M4 10H16"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
        />
      </svg>
    </button>

    <TaskForm :visible="fabFormVisible" @save="handleFabSave" @close="fabFormVisible = false" />
  </div>
</template>

<style scoped>
.tasks-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: calc(100vh - var(--titlebar-height));
  padding: 24px 20px 80px;
  position: relative;
  overflow: hidden;
}

.tasks-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  flex-shrink: 0;
}

.tasks-header-left {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.tasks-title {
  font-family: var(--font-sans);
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.tasks-count {
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
  background: var(--bg-card);
  padding: 2px 8px;
  border-radius: 999px;
}

.tasks-filters {
  flex-shrink: 0;
  margin-bottom: 16px;
}

.tasks-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  flex: 1;
}

.loading-text {
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--text-muted);
}

.tasks-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
}

.tasks-body::-webkit-scrollbar {
  width: 6px;
}

.tasks-body::-webkit-scrollbar-track {
  background: transparent;
}

.tasks-body::-webkit-scrollbar-thumb {
  background: var(--border-default);
  border-radius: 3px;
}

/* Floating action button */
.fab {
  position: absolute;
  bottom: 24px;
  right: 20px;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: none;
  background: var(--color-work);
  color: var(--bg-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: var(--shadow-fab);
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease;
  z-index: 10;
}

.fab:hover {
  transform: scale(1.08) rotate(90deg);
  box-shadow: var(--shadow-fab-hover);
}

.fab:active {
  transform: scale(0.96);
}
</style>
