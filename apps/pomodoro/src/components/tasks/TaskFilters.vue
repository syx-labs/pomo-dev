<script setup lang="ts">
import { computed } from "vue";
import { useTasksStore } from "@/stores/tasks";

const emit = defineEmits<{
  "filter-change": [filters: { status?: string; project?: string }];
}>();

const store = useTasksStore();

const activeStatus = computed(() => store.filterStatus);
const activeProject = computed(() => store.filterProject);
const projects = computed(() => store.projects);

const statusOptions = [
  { value: undefined, label: "All" },
  { value: "todo", label: "Todo" },
  { value: "in_progress", label: "In Progress" },
  { value: "done", label: "Done" },
] as const;

function setStatus(value: string | undefined) {
  store.setFilterStatus(value);
  emit("filter-change", { status: value, project: activeProject.value });
}

function setProject(value: string | undefined) {
  store.setFilterProject(value);
  emit("filter-change", { status: activeStatus.value, project: value });
}
</script>

<template>
  <div class="filters-bar">
    <div class="filter-group">
      <button
        v-for="opt in statusOptions"
        :key="opt.label"
        class="filter-pill"
        :class="{ 'filter-pill--active': activeStatus === opt.value }"
        @click="setStatus(opt.value)"
        type="button"
      >
        {{ opt.label }}
      </button>
    </div>

    <div v-if="projects.length > 0" class="filter-group">
      <span class="filter-divider" />
      <button
        class="filter-pill"
        :class="{ 'filter-pill--active': !activeProject }"
        @click="setProject(undefined)"
        type="button"
      >
        All Projects
      </button>
      <button
        v-for="p in projects"
        :key="p"
        class="filter-pill"
        :class="{ 'filter-pill--active': activeProject === p }"
        @click="setProject(p)"
        type="button"
      >
        {{ p }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.filters-bar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  padding: 0 0 4px;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.filter-divider {
  width: 1px;
  height: 16px;
  background: var(--border-default);
  margin: 0 6px;
}

.filter-pill {
  padding: 5px 12px;
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: var(--bg-card);
  border: 1px solid transparent;
  border-radius: 999px;
  cursor: pointer;
  white-space: nowrap;
  transition:
    all 0.15s ease,
    transform 0.1s ease;
}

.filter-pill:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.filter-pill:active {
  transform: scale(0.95);
}

.filter-pill--active {
  color: var(--bg-primary);
  background: var(--color-work);
  border-color: var(--color-work);
}

.filter-pill--active:hover {
  color: var(--bg-primary);
  opacity: 0.9;
}
</style>
