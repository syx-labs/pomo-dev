<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from "vue";
import type { Task, CreateTaskInput, UpdateTaskInput } from "@/lib/tauri";
import { useTasksStore } from "@/stores/tasks";

const props = defineProps<{
  task?: Task;
  visible: boolean;
}>();

const emit = defineEmits<{
  save: [input: CreateTaskInput | UpdateTaskInput];
  close: [];
}>();

const store = useTasksStore();

const title = ref("");
const description = ref("");
const priority = ref(0);
const project = ref("");
const dueDate = ref("");
const estimatedPomos = ref(0);
const showProjectSuggestions = ref(false);
const titleError = ref("");

const titleInput = ref<HTMLInputElement | null>(null);
const modalRef = ref<HTMLDivElement | null>(null);
let previouslyFocused: HTMLElement | null = null;

const isEditing = computed(() => !!props.task);
const formTitle = computed(() => (isEditing.value ? "Edit Task" : "New Task"));

const projectSuggestions = computed(() => {
  if (!project.value) return store.projects;
  const query = project.value.toLowerCase();
  return store.projects.filter((p) => p.toLowerCase().includes(query));
});

const priorityOptions = [
  { value: 0, label: "None" },
  { value: 1, label: "Low" },
  { value: 2, label: "Med" },
  { value: 3, label: "High" },
] as const;

function getPriorityIndicatorColor(value: number): string {
  switch (value) {
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

watch(
  () => props.visible,
  async (vis) => {
    if (vis) {
      previouslyFocused = document.activeElement as HTMLElement | null;
      titleError.value = "";
      if (props.task) {
        title.value = props.task.title;
        description.value = props.task.description ?? "";
        priority.value = props.task.priority;
        project.value = props.task.project ?? "";
        dueDate.value = props.task.due_date ?? "";
        estimatedPomos.value = props.task.estimated_pomos;
      } else {
        title.value = "";
        description.value = "";
        priority.value = 0;
        project.value = "";
        dueDate.value = "";
        estimatedPomos.value = 1;
      }
      await nextTick();
      titleInput.value?.focus();
    } else {
      previouslyFocused?.focus();
      previouslyFocused = null;
    }
  },
);

function handleSave() {
  if (!title.value.trim()) {
    titleError.value = "Title is required";
    return;
  }

  if (isEditing.value && props.task) {
    const input: UpdateTaskInput = {
      id: props.task.id,
      title: title.value.trim(),
      description: description.value.trim() || null,
      status: props.task.status,
      priority: priority.value,
      project: project.value.trim() || null,
      due_date: dueDate.value || null,
      estimated_pomos: estimatedPomos.value,
      sort_order: props.task.sort_order,
    };
    emit("save", input);
  } else {
    const input: CreateTaskInput = {
      title: title.value.trim(),
      description: description.value.trim() || undefined,
      priority: priority.value || undefined,
      project: project.value.trim() || undefined,
      due_date: dueDate.value || undefined,
      estimated_pomos: estimatedPomos.value || undefined,
    };
    emit("save", input);
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (!props.visible) return;

  if (e.key === "Escape") {
    emit("close");
    return;
  }
  if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
    handleSave();
    return;
  }

  // Focus trap
  if (e.key === "Tab" && modalRef.value) {
    const focusable = modalRef.value.querySelectorAll<HTMLElement>(
      'input:not([disabled]), textarea:not([disabled]), button:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])',
    );
    if (focusable.length === 0) return;

    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    if (e.shiftKey) {
      if (document.activeElement === first) {
        e.preventDefault();
        last.focus();
      }
    } else {
      if (document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }
}

function hideProjectSuggestions() {
  setTimeout(() => {
    showProjectSuggestions.value = false;
  }, 150);
}

function selectProject(p: string) {
  project.value = p;
  showProjectSuggestions.value = false;
}

function incrementPomos() {
  estimatedPomos.value++;
}

function decrementPomos() {
  if (estimatedPomos.value > 0) estimatedPomos.value--;
}

function onOverlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains("form-overlay")) {
    emit("close");
  }
}

onMounted(() => {
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="form-overlay" @mousedown="onOverlayClick">
        <div ref="modalRef" class="form-modal" role="dialog" :aria-label="formTitle">
          <h2 class="form-title">{{ formTitle }}</h2>

          <div class="form-field">
            <label class="form-label" for="task-title">Title</label>
            <input
              id="task-title"
              ref="titleInput"
              v-model="title"
              type="text"
              class="form-input"
              :class="{ 'form-input--error': titleError }"
              placeholder="What needs to be done?"
              required
              @keydown="titleError = ''"
            />
            <span v-if="titleError" class="form-error">{{ titleError }}</span>
          </div>

          <div class="form-field">
            <label class="form-label" for="task-desc">Description</label>
            <textarea
              id="task-desc"
              v-model="description"
              class="form-input form-textarea"
              placeholder="Optional details..."
              rows="3"
            />
          </div>

          <div class="form-field">
            <label class="form-label">Priority</label>
            <div class="priority-segmented">
              <button
                v-for="opt in priorityOptions"
                :key="opt.value"
                class="priority-option"
                :class="{ 'priority-option--active': priority === opt.value }"
                :style="
                  priority === opt.value
                    ? { borderColor: getPriorityIndicatorColor(opt.value) }
                    : {}
                "
                @click="priority = opt.value"
                type="button"
              >
                <span
                  class="priority-indicator"
                  :style="{ backgroundColor: getPriorityIndicatorColor(opt.value) }"
                />
                {{ opt.label }}
              </button>
            </div>
          </div>

          <div class="form-field">
            <label class="form-label" for="task-project">Project</label>
            <div class="project-input-wrap">
              <input
                id="task-project"
                v-model="project"
                type="text"
                class="form-input"
                placeholder="e.g. Work, Personal"
                @focus="showProjectSuggestions = true"
                @blur="hideProjectSuggestions"
              />
              <div
                v-if="showProjectSuggestions && projectSuggestions.length > 0"
                class="project-suggestions"
              >
                <button
                  v-for="p in projectSuggestions"
                  :key="p"
                  class="project-suggestion"
                  @mousedown.prevent="selectProject(p)"
                  type="button"
                >
                  {{ p }}
                </button>
              </div>
            </div>
          </div>

          <div class="form-row">
            <div class="form-field form-field--half">
              <label class="form-label" for="task-due">Due date</label>
              <input id="task-due" v-model="dueDate" type="date" class="form-input" />
            </div>

            <div class="form-field form-field--half">
              <label class="form-label">Est. pomodoros</label>
              <div class="pomo-stepper">
                <button
                  class="pomo-btn"
                  @click="decrementPomos"
                  type="button"
                  :disabled="estimatedPomos <= 0"
                >
                  -
                </button>
                <span class="pomo-value">{{ estimatedPomos }}</span>
                <button class="pomo-btn" @click="incrementPomos" type="button">+</button>
              </div>
            </div>
          </div>

          <div class="form-actions">
            <button class="btn btn--ghost" @click="emit('close')" type="button">Cancel</button>
            <button
              class="btn btn--primary"
              @click="handleSave"
              type="button"
              :disabled="!title.trim()"
            >
              {{ isEditing ? "Save Changes" : "Create Task" }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.form-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--overlay-bg);
  backdrop-filter: blur(4px);
}

.form-modal {
  width: 100%;
  max-width: 480px;
  margin: 16px;
  padding: 24px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
}

.form-title {
  font-family: var(--font-sans);
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 20px;
}

.form-field {
  margin-bottom: 16px;
}

.form-field--half {
  flex: 1;
  min-width: 0;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-label {
  display: block;
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.form-input {
  width: 100%;
  padding: 10px 12px;
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--text-primary);
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color 0.15s ease;
  box-sizing: border-box;
}

.form-input::placeholder {
  color: var(--text-muted);
}

.form-input:focus {
  border-color: var(--color-work);
}

.form-input--error {
  border-color: var(--color-priority-high);
}

.form-input--error:focus {
  border-color: var(--color-priority-high);
}

.form-error {
  display: block;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--color-priority-high);
  margin-top: 4px;
}

.form-textarea {
  resize: vertical;
  min-height: 60px;
}

/* Priority segmented control */
.priority-segmented {
  display: flex;
  gap: 6px;
}

.priority-option {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 4px;
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: var(--bg-card);
  border: 1.5px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.15s ease;
}

.priority-option:hover {
  background: var(--bg-hover);
}

.priority-option--active {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.priority-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* Project autocomplete */
.project-input-wrap {
  position: relative;
}

.project-suggestions {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  overflow: hidden;
  z-index: 10;
}

.project-suggestion {
  display: block;
  width: 100%;
  text-align: left;
  padding: 8px 12px;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition:
    background 0.1s ease,
    color 0.1s ease;
}

.project-suggestion:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Pomo stepper */
.pomo-stepper {
  display: flex;
  align-items: center;
  gap: 0;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--bg-card);
}

.pomo-btn {
  width: 36px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-mono);
  font-size: 16px;
  font-weight: 600;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition:
    background 0.1s ease,
    color 0.1s ease;
}

.pomo-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.pomo-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.pomo-value {
  flex: 1;
  text-align: center;
  font-family: var(--font-mono);
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

/* Date input styling */
input[type="date"] {
  color-scheme: var(--date-color-scheme);
}

/* Actions */
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 24px;
}

.btn {
  padding: 10px 20px;
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 600;
  border-radius: var(--radius-sm);
  border: none;
  cursor: pointer;
  transition:
    background 0.15s ease,
    opacity 0.15s ease;
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn--ghost {
  color: var(--text-secondary);
  background: transparent;
}

.btn--ghost:hover {
  color: var(--text-primary);
  background: var(--bg-card);
}

.btn--ghost:active {
  transform: scale(0.97);
}

.btn--primary {
  color: var(--bg-primary);
  background: var(--color-work);
}

.btn--primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn--primary:active:not(:disabled) {
  transform: scale(0.97);
}

/* Modal transition */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active .form-modal,
.modal-leave-active .form-modal {
  transition:
    transform 0.2s ease,
    opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .form-modal,
.modal-leave-to .form-modal {
  opacity: 0;
  transform: scale(0.95) translateY(8px);
}
</style>
