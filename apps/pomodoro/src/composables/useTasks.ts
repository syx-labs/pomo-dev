import { computed } from "vue";
import { useTasksStore } from "@/stores/tasks";
import type { Task } from "@/lib/tauri";

export function useTasks() {
  const store = useTasksStore();

  const filteredTasks = computed(() => store.filteredTasks);
  const projects = computed(() => store.projects);
  const loading = computed(() => store.loading);

  function toggleStatus(task: Task): "todo" | "in_progress" | "done" {
    const cycle: Record<string, "todo" | "in_progress" | "done"> = {
      todo: "in_progress",
      in_progress: "done",
      done: "todo",
    };
    return cycle[task.status] ?? "todo";
  }

  function getPriorityLabel(priority: number): string {
    switch (priority) {
      case 3:
        return "High";
      case 2:
        return "Medium";
      case 1:
        return "Low";
      default:
        return "None";
    }
  }

  function getPriorityColor(priority: number): string {
    switch (priority) {
      case 3:
        return "var(--color-priority-high)";
      case 2:
        return "var(--color-priority-medium)";
      case 1:
        return "var(--color-priority-low)";
      default:
        return "var(--text-muted)";
    }
  }

  async function cycleTaskStatus(task: Task) {
    // Read current task from store to avoid stale data
    const current = store.tasks.find((t) => t.id === task.id);
    if (!current) return; // Task was deleted, bail

    const newStatus = toggleStatus(current);
    await store.editTask({
      id: current.id,
      title: current.title,
      description: current.description,
      status: newStatus,
      priority: current.priority,
      project: current.project,
      due_date: current.due_date,
      estimated_pomos: current.estimated_pomos,
      sort_order: current.sort_order,
    });
  }

  return {
    filteredTasks,
    projects,
    loading,
    toggleStatus,
    getPriorityLabel,
    getPriorityColor,
    cycleTaskStatus,
  };
}
