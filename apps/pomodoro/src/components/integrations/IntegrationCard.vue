<script setup lang="ts">
import { computed } from "vue";
import type { IntegrationConfig, EventLogEntry } from "@/lib/tauri";

const props = defineProps<{
  integration: IntegrationConfig;
  testing: boolean;
  lastEvent?: EventLogEntry;
}>();

const emit = defineEmits<{
  edit: [];
  delete: [];
  test: [];
  toggle: [];
}>();

const events = computed<string[]>(() => {
  try {
    return JSON.parse(props.integration.events) as string[];
  } catch {
    return [];
  }
});

const typeLabel = computed(() => {
  switch (props.integration.integration_type) {
    case "webhook":
      return "Webhook";
    case "slack":
      return "Slack";
    case "discord":
      return "Discord";
    default:
      return props.integration.integration_type;
  }
});

function relativeTime(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime();
  const secs = Math.floor(diff / 1000);
  if (secs < 60) return "just now";
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}min ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  const days = Math.floor(hrs / 24);
  return `${days}d ago`;
}
</script>

<template>
  <div class="card" :class="{ disabled: !integration.enabled }">
    <div class="card-header">
      <div class="card-title-row">
        <!-- Icon -->
        <svg
          v-if="integration.integration_type === 'webhook'"
          class="card-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
          <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
        </svg>
        <svg
          v-else-if="integration.integration_type === 'slack'"
          class="card-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
        </svg>
        <svg
          v-else
          class="card-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="12" cy="12" r="10" />
          <path d="M8 14s1.5 2 4 2 4-2 4-2" />
          <line x1="9" y1="9" x2="9.01" y2="9" />
          <line x1="15" y1="9" x2="15.01" y2="9" />
        </svg>

        <div class="card-info">
          <span class="card-name">{{ integration.name }}</span>
          <span class="card-type">{{ typeLabel }}</span>
        </div>

        <button
          class="toggle-switch"
          :class="{ active: integration.enabled }"
          @click="emit('toggle')"
          type="button"
          :aria-label="integration.enabled ? 'Disable' : 'Enable'"
        >
          <span class="toggle-knob" />
        </button>
      </div>
    </div>

    <div class="card-events">
      <span v-for="evt in events" :key="evt" class="event-pill">{{ evt }}</span>
    </div>

    <div v-if="lastEvent" class="card-last">
      <span
        class="status-dot"
        :class="{
          success: lastEvent.status === 'success',
          error: lastEvent.status === 'error',
        }"
      />
      <span class="last-status">{{ lastEvent.status }}</span>
      <span class="last-time">{{ relativeTime(lastEvent.created_at) }}</span>
      <span v-if="lastEvent.error_message" class="last-error">{{ lastEvent.error_message }}</span>
    </div>

    <div class="card-actions">
      <button class="action-btn" :disabled="testing" @click="emit('test')" type="button">
        <svg
          v-if="testing"
          class="spinner"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="12" cy="12" r="10" opacity="0.25" />
          <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round" />
        </svg>
        <template v-else>Test</template>
      </button>
      <button class="action-btn" @click="emit('edit')" type="button">Edit</button>
      <button class="action-btn action-btn--danger" @click="emit('delete')" type="button">
        Delete
      </button>
    </div>
  </div>
</template>

<style scoped>
.card {
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: opacity 0.2s;
}

.card.disabled {
  opacity: 0.5;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.card-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
}

.card-icon {
  width: 18px;
  height: 18px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.card-info {
  flex: 1;
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.card-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-type {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  flex-shrink: 0;
}

.toggle-switch {
  position: relative;
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: var(--bg-hover);
  border: none;
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  transition: background 0.2s;
}

.toggle-switch.active {
  background: var(--color-work);
}

.toggle-switch .toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--toggle-knob-bg);
  transition: transform 0.2s;
}

.toggle-switch.active .toggle-knob {
  transform: translateX(16px);
}

.card-events {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.event-pill {
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-hover);
  border-radius: 4px;
  padding: 2px 6px;
  white-space: nowrap;
}

.card-last {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-status-neutral);
  flex-shrink: 0;
}

.status-dot.success {
  background: var(--color-success);
}

.status-dot.error {
  background: var(--color-error);
}

.last-status {
  font-weight: 500;
}

.last-time {
  color: var(--text-muted);
}

.last-error {
  color: var(--color-priority-high);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-actions {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}

.action-btn {
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: transparent;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    color 0.15s,
    background 0.15s,
    border-color 0.15s;
  display: flex;
  align-items: center;
  gap: 4px;
}

.action-btn:hover:not(:disabled) {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.action-btn:active:not(:disabled) {
  transform: scale(0.95);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn--danger:hover:not(:disabled) {
  color: var(--color-priority-high);
  border-color: var(--color-priority-high);
}

.spinner {
  width: 14px;
  height: 14px;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
