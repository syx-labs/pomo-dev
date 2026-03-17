<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useIntegrationsStore } from "@/stores/integrations";

const props = withDefaults(
  defineProps<{
    integrationId?: string;
    limit?: number;
  }>(),
  {
    limit: 10,
  },
);

const store = useIntegrationsStore();
const expandedIds = ref<Set<string>>(new Set());

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

function toggleExpand(id: string) {
  if (expandedIds.value.has(id)) {
    expandedIds.value.delete(id);
  } else {
    expandedIds.value.add(id);
  }
}

function formatPayload(payload: string): string {
  try {
    return JSON.stringify(JSON.parse(payload), null, 2);
  } catch {
    return payload;
  }
}

function getIntegrationName(integrationId: string): string {
  const item = store.integrations.find((i) => i.id === integrationId);
  return item?.name ?? "Unknown";
}

function refresh() {
  store.fetchEventLog(props.integrationId, props.limit);
}

onMounted(() => refresh());
</script>

<template>
  <div class="event-log">
    <div class="log-header">
      <span class="log-title">Event Log</span>
      <button class="refresh-btn" type="button" @click="refresh">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          width="14"
          height="14"
        >
          <path d="M23 4v6h-6" />
          <path d="M1 20v-6h6" />
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
        </svg>
        Refresh
      </button>
    </div>

    <div v-if="store.eventLog.length === 0" class="log-empty">No events dispatched yet.</div>

    <div v-else class="log-entries">
      <div
        v-for="entry in store.eventLog"
        :key="entry.id"
        class="log-entry"
        :class="{ expanded: expandedIds.has(entry.id) }"
        @click="toggleExpand(entry.id)"
      >
        <div class="entry-row">
          <span
            class="status-dot"
            :class="{
              success: entry.status === 'success',
              error: entry.status === 'error',
            }"
          />
          <span class="entry-event">{{ entry.event_type }}</span>
          <span class="entry-arrow">&rarr;</span>
          <span class="entry-target">{{ getIntegrationName(entry.integration_id) }}</span>
          <span class="entry-time">{{ relativeTime(entry.created_at) }}</span>
          <span
            class="entry-status-label"
            :class="{
              'label-success': entry.status === 'success',
              'label-error': entry.status === 'error',
            }"
            >{{ entry.status }}</span
          >
        </div>
        <div v-if="entry.error_message" class="entry-error">
          {{ entry.error_message }}
        </div>
        <div v-if="expandedIds.has(entry.id)" class="entry-payload">
          <pre>{{ formatPayload(entry.payload) }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.event-log {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.log-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.refresh-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  font-size: 11px;
  color: var(--text-muted);
  background: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    color 0.15s,
    border-color 0.15s;
}

.refresh-btn:hover {
  color: var(--text-primary);
  border-color: var(--text-muted);
}

.log-empty {
  font-size: 13px;
  color: var(--text-muted);
  padding: 12px 0;
}

.log-entries {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.log-entry {
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.1s;
}

.log-entry:hover {
  background: var(--bg-card);
}

.entry-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
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

.entry-event {
  color: var(--text-primary);
  font-weight: 500;
}

.entry-arrow {
  color: var(--text-muted);
}

.entry-target {
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entry-time {
  color: var(--text-muted);
  margin-left: auto;
  flex-shrink: 0;
}

.entry-status-label {
  font-size: 10px;
  font-weight: 500;
  padding: 1px 5px;
  border-radius: 3px;
  flex-shrink: 0;
}

.label-success {
  color: var(--color-success);
  background: var(--color-work-alpha-10);
}

.label-error {
  color: var(--color-error);
  background: var(--color-error-alpha-10);
}

.entry-error {
  font-size: 11px;
  color: var(--color-priority-high);
  margin-top: 4px;
  padding-left: 14px;
}

.entry-payload {
  margin-top: 6px;
  padding-left: 14px;
}

.entry-payload pre {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-primary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 8px;
  margin: 0;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
