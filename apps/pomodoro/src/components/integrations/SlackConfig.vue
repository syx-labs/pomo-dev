<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue";
import type { IntegrationConfig } from "@/lib/tauri";

const ALL_EVENTS = [
  "session.started",
  "session.completed",
  "session.paused",
  "break.started",
  "task.created",
  "task.completed",
] as const;

const DEFAULT_EVENTS = ["session.started", "session.completed", "break.started"];

const props = defineProps<{
  integration?: IntegrationConfig;
  visible: boolean;
}>();

const emit = defineEmits<{
  save: [config: { name: string; botToken: string; statusEmoji: string; events: string[] }];
  cancel: [];
}>();

const name = ref("Slack Status");
const botToken = ref("");
const statusEmoji = ref("\u{1F345}");
const selectedEvents = ref<string[]>([...DEFAULT_EVENTS]);
const nameError = ref("");
const tokenError = ref("");

const modalRef = ref<HTMLDivElement | null>(null);
const nameInput = ref<HTMLInputElement | null>(null);
let previouslyFocused: HTMLElement | null = null;

watch(
  () => props.visible,
  async (vis) => {
    if (vis) {
      previouslyFocused = document.activeElement as HTMLElement | null;
      nameError.value = "";
      tokenError.value = "";
      if (props.integration) {
        const cfg = JSON.parse(props.integration.config) as {
          botToken?: string;
          statusEmoji?: string;
        };
        name.value = props.integration.name;
        botToken.value = cfg.botToken ?? "";
        statusEmoji.value = cfg.statusEmoji ?? "\u{1F345}";
        selectedEvents.value = JSON.parse(props.integration.events) as string[];
      } else {
        name.value = "Slack Status";
        botToken.value = "";
        statusEmoji.value = "\u{1F345}";
        selectedEvents.value = [...DEFAULT_EVENTS];
      }
      await nextTick();
      nameInput.value?.focus();
    } else {
      previouslyFocused?.focus();
      previouslyFocused = null;
    }
  },
);

function toggleEvent(evt: string) {
  const idx = selectedEvents.value.indexOf(evt);
  if (idx === -1) {
    selectedEvents.value.push(evt);
  } else {
    selectedEvents.value.splice(idx, 1);
  }
}

function handleSave() {
  nameError.value = "";
  tokenError.value = "";

  if (!name.value.trim()) {
    nameError.value = "Name is required";
    return;
  }
  if (!botToken.value.trim()) {
    tokenError.value = "Bot token is required";
    return;
  }

  emit("save", {
    name: name.value.trim(),
    botToken: botToken.value.trim(),
    statusEmoji: statusEmoji.value,
    events: selectedEvents.value,
  });
}

function handleKeydown(e: KeyboardEvent) {
  if (!props.visible) return;
  if (e.key === "Escape") {
    emit("cancel");
    return;
  }
  if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
    handleSave();
    return;
  }
}

function onOverlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains("config-overlay")) {
    emit("cancel");
  }
}

onMounted(() => document.addEventListener("keydown", handleKeydown));
onUnmounted(() => document.removeEventListener("keydown", handleKeydown));
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="config-overlay" @mousedown="onOverlayClick">
        <div ref="modalRef" class="config-modal" role="dialog" aria-label="Slack Configuration">
          <h2 class="config-title">{{ integration ? "Edit Slack" : "New Slack Integration" }}</h2>

          <div class="form-field">
            <label class="form-label">Name</label>
            <input
              ref="nameInput"
              v-model="name"
              type="text"
              class="form-input"
              :class="{ 'form-input--error': nameError }"
              placeholder="Slack Status"
              @keydown="nameError = ''"
            />
            <span v-if="nameError" class="form-error">{{ nameError }}</span>
          </div>

          <div class="form-field">
            <label class="form-label">Bot Token</label>
            <input
              v-model="botToken"
              type="password"
              class="form-input"
              :class="{ 'form-input--error': tokenError }"
              placeholder="xoxb-..."
              @keydown="tokenError = ''"
            />
            <span v-if="tokenError" class="form-error">{{ tokenError }}</span>
          </div>

          <div class="form-field">
            <label class="form-label">Status Emoji</label>
            <input v-model="statusEmoji" type="text" class="form-input emoji-input" />
          </div>

          <div class="form-field">
            <label class="form-label">Events</label>
            <div class="events-grid">
              <label v-for="evt in ALL_EVENTS" :key="evt" class="event-check">
                <input
                  type="checkbox"
                  :checked="selectedEvents.includes(evt)"
                  @change="toggleEvent(evt)"
                />
                <span>{{ evt }}</span>
              </label>
            </div>
          </div>

          <div class="form-actions">
            <button class="btn btn--ghost" type="button" @click="emit('cancel')">Cancel</button>
            <button class="btn btn--primary" type="button" @click="handleSave">Save</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.config-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--overlay-bg);
  backdrop-filter: blur(4px);
}

.config-modal {
  width: 100%;
  max-width: 480px;
  max-height: 80vh;
  overflow-y: auto;
  margin: 16px;
  padding: 24px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
}

.config-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 20px;
}

.form-field {
  margin-bottom: 16px;
}

.form-label {
  display: block;
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
  font-size: 14px;
  color: var(--text-primary);
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color 0.15s;
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

.form-error {
  display: block;
  font-size: 12px;
  color: var(--color-priority-high);
  margin-top: 4px;
}

.emoji-input {
  width: 80px;
}

.events-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.event-check {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
}

.event-check input[type="checkbox"] {
  accent-color: var(--color-work);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 24px;
}

.btn {
  padding: 10px 20px;
  font-size: 13px;
  font-weight: 600;
  border-radius: var(--radius-sm);
  border: none;
  cursor: pointer;
  transition:
    background 0.15s,
    opacity 0.15s;
}

.btn--ghost {
  color: var(--text-secondary);
  background: transparent;
}

.btn--ghost:hover {
  color: var(--text-primary);
  background: var(--bg-card);
}

.btn--primary {
  color: var(--bg-primary);
  background: var(--color-work);
}

.btn--primary:hover {
  opacity: 0.9;
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-active .config-modal,
.modal-leave-active .config-modal {
  transition:
    transform 0.2s ease,
    opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from .config-modal,
.modal-leave-to .config-modal {
  opacity: 0;
  transform: scale(0.95) translateY(8px);
}
</style>
