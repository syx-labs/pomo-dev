import { defineStore } from "pinia";
import { ref, computed } from "vue";
import {
  listTasks,
  createTask,
  updateTask,
  deleteTask,
  reorderTasks,
  getSessionsForTask,
  type Task,
  type CreateTaskInput,
  type UpdateTaskInput,
} from "@/lib/tauri";
import { useToast } from "@/composables/useToast";

export const useTasksStore = defineStore("tasks", () => {
  const tasks = ref<Task[]>([]);
  const loading = ref(false);
  const filterStatus = ref<string | undefined>(undefined);
  const filterProject = ref<string | undefined>(undefined);
  const sessionCounts = ref<Record<string, number>>({});
  const savingTaskId = ref<string | null>(null);
  const deletingTaskId = ref<string | null>(null);

  const { showToast } = useToast();

  const filteredTasks = computed(() => {
    let result = [...tasks.value];
    if (filterStatus.value) {
      result = result.filter((t) => t.status === filterStatus.value);
    }
    if (filterProject.value) {
      result = result.filter((t) => t.project === filterProject.value);
    }
    return result.sort((a, b) => {
      if (a.sort_order !== b.sort_order) return a.sort_order - b.sort_order;
      return b.priority - a.priority;
    });
  });

  const groupedByStatus = computed(() => {
    const groups: Record<string, Task[]> = {
      todo: [],
      in_progress: [],
      done: [],
      archived: [],
    };
    for (const task of filteredTasks.value) {
      if (groups[task.status]) {
        groups[task.status].push(task);
      }
    }
    return groups;
  });

  const projects = computed(() => {
    const set = new Set<string>();
    for (const task of tasks.value) {
      if (task.project) set.add(task.project);
    }
    return Array.from(set).sort();
  });

  async function fetchTasks() {
    loading.value = true;
    try {
      tasks.value = await listTasks();
    } catch {
      showToast("Failed to load tasks", "error");
    } finally {
      loading.value = false;
    }
  }

  async function addTask(input: CreateTaskInput): Promise<Task | undefined> {
    try {
      const task = await createTask(input);
      tasks.value.push(task);
      return task;
    } catch {
      showToast("Failed to create task", "error");
      return undefined;
    }
  }

  async function editTask(input: UpdateTaskInput): Promise<Task | undefined> {
    savingTaskId.value = input.id;

    // Optimistic update: save previous state and apply immediately
    const idx = tasks.value.findIndex((t) => t.id === input.id);
    const previous = idx !== -1 ? { ...tasks.value[idx] } : null;

    if (idx !== -1) {
      tasks.value[idx] = { ...tasks.value[idx], ...input } as Task;
    }

    try {
      const updated = await updateTask(input);
      if (idx !== -1) {
        tasks.value[idx] = updated;
      }
      return updated;
    } catch {
      // Revert optimistic update
      if (idx !== -1 && previous) {
        tasks.value[idx] = previous;
      }
      showToast("Failed to save task", "error");
      return undefined;
    } finally {
      savingTaskId.value = null;
    }
  }

  async function removeTask(id: string) {
    deletingTaskId.value = id;

    // Optimistic: remove immediately, keep reference for revert
    const idx = tasks.value.findIndex((t) => t.id === id);
    const removed = idx !== -1 ? tasks.value[idx] : null;

    if (idx !== -1) {
      tasks.value.splice(idx, 1);
    }

    try {
      await deleteTask(id);
    } catch {
      // Revert: re-insert at original position
      if (removed && idx !== -1) {
        tasks.value.splice(idx, 0, removed);
      }
      showToast("Failed to delete task", "error");
    } finally {
      deletingTaskId.value = null;
    }
  }

  async function reorder(taskIds: string[]) {
    try {
      await reorderTasks(taskIds);
      await fetchTasks();
    } catch {
      showToast("Failed to reorder tasks", "error");
    }
  }

  async function fetchSessionCount(taskId: string): Promise<number> {
    try {
      const sessions = await getSessionsForTask(taskId);
      const count = sessions.filter((s) => s.completed && s.session_type === "work").length;
      sessionCounts.value[taskId] = count;
      return count;
    } catch {
      return sessionCounts.value[taskId] ?? 0;
    }
  }

  function setFilterStatus(status: string | undefined) {
    filterStatus.value = status;
  }

  function setFilterProject(project: string | undefined) {
    filterProject.value = project;
  }

  return {
    tasks,
    loading,
    filterStatus,
    filterProject,
    filteredTasks,
    groupedByStatus,
    projects,
    sessionCounts,
    savingTaskId,
    deletingTaskId,
    fetchTasks,
    addTask,
    editTask,
    removeTask,
    reorder,
    fetchSessionCount,
    setFilterStatus,
    setFilterProject,
  };
});
