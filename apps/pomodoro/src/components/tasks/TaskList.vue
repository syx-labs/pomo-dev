<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import type { Task, CreateTaskInput, UpdateTaskInput } from "@/lib/tauri";
import { useTasksStore } from "@/stores/tasks";
import { useTimerStore } from "@/stores/timer";
import { useToast } from "@/composables/useToast";
import TaskCard from "./TaskCard.vue";
import TaskForm from "./TaskForm.vue";
import ConfirmDialog from "./ConfirmDialog.vue";

defineProps<{
  tasks: Task[];
}>();

const store = useTasksStore();
const timerStore = useTimerStore();
const router = useRouter();
const { showToast } = useToast();

const formVisible = ref(false);
const editingTask = ref<Task | undefined>(undefined);

// Confirm dialog state
const confirmVisible = ref(false);
const pendingDeleteTask = ref<Task | null>(null);

function openCreate() {
  editingTask.value = undefined;
  formVisible.value = true;
}

function openEdit(task: Task) {
  editingTask.value = task;
  formVisible.value = true;
}

function closeForm() {
  formVisible.value = false;
  editingTask.value = undefined;
}

async function handleSave(input: CreateTaskInput | UpdateTaskInput) {
  if ("id" in input) {
    await store.editTask(input as UpdateTaskInput);
  } else {
    await store.addTask(input as CreateTaskInput);
  }
  closeForm();
}

function handleDelete(task: Task) {
  pendingDeleteTask.value = task;
  confirmVisible.value = true;
}

async function confirmDelete() {
  if (pendingDeleteTask.value) {
    await store.removeTask(pendingDeleteTask.value.id);
  }
  confirmVisible.value = false;
  pendingDeleteTask.value = null;
}

function cancelDelete() {
  confirmVisible.value = false;
  pendingDeleteTask.value = null;
}

async function handleStartPomodoro(task: Task) {
  await timerStore.startForTask(task.id, task.title);
  if (timerStore.isRunning) {
    showToast(`Pomodoro started: ${task.title}`, "success");
    router.push("/");
  }
}

defineExpose({ openCreate });
</script>

<template>
  <div class="task-list-container" aria-live="polite">
    <TransitionGroup v-if="tasks.length > 0" name="task-item" tag="div" class="task-list">
      <TaskCard
        v-for="task in tasks"
        :key="task.id"
        :task="task"
        @edit="openEdit"
        @delete="handleDelete"
        @start-pomodoro="handleStartPomodoro"
      />
    </TransitionGroup>

    <div v-else class="empty-state">
      <div class="empty-icon">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
          <rect x="8" y="10" width="32" height="28" rx="4" stroke="currentColor" stroke-width="2" />
          <path
            d="M16 22H32M16 30H26"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          />
        </svg>
      </div>
      <p class="empty-text">No tasks yet</p>
      <button class="empty-add-btn" @click="openCreate" type="button">Add your first task</button>
    </div>

    <TaskForm :visible="formVisible" :task="editingTask" @save="handleSave" @close="closeForm" />

    <ConfirmDialog
      :visible="confirmVisible"
      title="Delete Task"
      :message="`Are you sure you want to delete &quot;${pendingDeleteTask?.title ?? ''}&quot;?`"
      confirm-label="Delete"
      confirm-variant="danger"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
    />
  </div>
</template>

<style scoped>
.task-list-container {
  flex: 1;
  min-height: 0;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

/* Empty state */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  text-align: center;
}

.empty-icon {
  color: var(--text-muted);
  opacity: 0.5;
  margin-bottom: 16px;
}

.empty-text {
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 500;
  color: var(--text-muted);
  margin: 0 0 16px;
}

.empty-add-btn {
  padding: 10px 20px;
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 600;
  color: var(--bg-primary);
  background: var(--color-work);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}

.empty-add-btn:hover {
  opacity: 0.9;
  transform: translateY(-1px);
}

.empty-add-btn:active {
  transform: scale(0.97);
}

/* List transitions */
.task-item-enter-active {
  transition: all 0.25s ease;
}

.task-item-leave-active {
  transition: all 0.2s ease;
}

.task-item-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}

.task-item-leave-to {
  opacity: 0;
  transform: translateX(16px);
}

.task-item-move {
  transition: transform 0.25s ease;
}
</style>
