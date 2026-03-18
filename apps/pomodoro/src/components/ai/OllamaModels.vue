<script setup lang="ts">
import { onMounted } from "vue";
import { useOllamaStore } from "@/stores/ollama";

const store = useOllamaStore();

function formatSize(mb: number): string {
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${mb} MB`;
}

function formatBytes(bytes: number): string {
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return `${gb.toFixed(1)} GB`;
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(0)} MB`;
}

function openOllamaDownload() {
  window.open("https://ollama.com/download", "_blank");
}

onMounted(() => {
  store.init();
});
</script>

<template>
  <div class="ollama-models">
    <!-- Connection Status -->
    <div
      class="status-banner"
      :class="{ connected: store.isRunning, disconnected: store.isRunning === false }"
    >
      <div class="status-left">
        <span class="status-dot" />
        <span v-if="store.isRunning">Ollama running</span>
        <span v-else-if="store.isRunning === false">Ollama not detected</span>
        <span v-else>Checking...</span>
      </div>
      <div class="status-actions">
        <button
          v-if="store.isRunning === false"
          class="action-btn install-btn"
          @click="openOllamaDownload"
        >
          Install Ollama
        </button>
        <button class="action-btn" :disabled="store.checking" @click="store.checkHealth()">
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            :class="{ spinning: store.checking }"
          >
            <polyline points="23 4 23 10 17 10" />
            <polyline points="1 20 1 14 7 14" />
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
        </button>
      </div>
    </div>

    <template v-if="store.isRunning">
      <!-- Active Model -->
      <div v-if="store.activeModel" class="active-model">
        Active: <strong>{{ store.activeModel }}</strong>
      </div>

      <!-- Downloaded Models -->
      <div v-if="store.downloadedCurated.length > 0" class="section">
        <h4 class="section-title">Downloaded Models</h4>
        <div class="model-list">
          <div
            v-for="model in store.downloadedCurated"
            :key="model.name"
            class="model-card"
            :class="{ active: store.activeModel === model.name }"
          >
            <div class="model-info">
              <div class="model-header">
                <span class="model-name">{{ model.display_name }}</span>
                <span v-if="store.activeModel === model.name" class="badge badge-active"
                  >Active</span
                >
                <span class="badge" :class="`badge-${model.category}`">{{ model.category }}</span>
              </div>
              <p class="model-desc">{{ model.description }}</p>
            </div>
            <div class="model-actions">
              <button
                v-if="store.activeModel !== model.name"
                class="action-btn"
                @click="store.setActiveModel(model.name)"
              >
                Select
              </button>
              <button
                class="action-btn danger"
                :disabled="store.deleting === model.name"
                @click="store.deleteModel(model.name)"
              >
                {{ store.deleting === model.name ? "Deleting..." : "Delete" }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Available to Download -->
      <div v-if="store.availableModels.length > 0" class="section">
        <h4 class="section-title">Available to Download</h4>
        <div class="model-list">
          <div v-for="model in store.availableModels" :key="model.name" class="model-card">
            <div class="model-info">
              <div class="model-header">
                <span class="model-name">{{ model.display_name }}</span>
                <span class="badge" :class="`badge-${model.category}`">{{ model.category }}</span>
              </div>
              <p class="model-desc">{{ model.description }}</p>
            </div>

            <!-- Download progress -->
            <div v-if="store.pullingModel === model.name" class="progress-section">
              <div class="progress-bar-track">
                <div
                  class="progress-bar-fill"
                  :style="{ width: `${store.pullProgress?.percent ?? 0}%` }"
                />
              </div>
              <div class="progress-info">
                <span class="progress-status">{{
                  store.pullProgress?.status ?? "Starting..."
                }}</span>
                <span
                  v-if="store.pullProgress && store.pullProgress.total > 0"
                  class="progress-pct"
                >
                  {{ formatBytes(store.pullProgress.completed) }} /
                  {{ formatBytes(store.pullProgress.total) }} ({{
                    Math.round(store.pullProgress.percent)
                  }}%)
                </span>
              </div>
              <button class="action-btn danger" @click="store.cancelPull()">Cancel</button>
            </div>

            <!-- Download button -->
            <div v-else class="model-actions">
              <span class="model-size">{{ formatSize(model.size_mb) }}</span>
              <button
                class="action-btn download-btn"
                :disabled="store.pullingModel !== null"
                @click="store.pullModel(model.name)"
              >
                Download
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- No models at all -->
      <div
        v-if="store.downloadedCurated.length === 0 && store.availableModels.length === 0"
        class="empty-state"
      >
        Loading models...
      </div>
    </template>
  </div>
</template>

<style scoped>
.ollama-models {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 16px;
}

.status-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--bg-card);
  font-size: 13px;
}

.status-banner.connected {
  border-color: var(--color-work);
}

.status-banner.disconnected {
  border-color: #f59e0b;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted);
}

.connected .status-dot {
  background: var(--color-work);
}

.disconnected .status-dot {
  background: #f59e0b;
}

.status-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  margin: 0;
}

.model-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.model-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px 14px;
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
}

.model-card.active {
  border-color: var(--color-work);
}

.model-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.model-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.model-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.model-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0;
}

.model-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.model-size {
  font-size: 12px;
  color: var(--text-muted);
  margin-right: auto;
}

.badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 9999px;
  text-transform: capitalize;
}

.badge-active {
  background: var(--color-work);
  color: #000;
}

.badge-fast {
  background: rgba(16, 185, 129, 0.15);
  color: var(--color-work);
}

.badge-balanced {
  background: rgba(56, 189, 248, 0.15);
  color: var(--color-short-break);
}

.badge-capable {
  background: rgba(139, 92, 246, 0.15);
  color: var(--color-long-break);
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  background: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    color 0.15s,
    border-color 0.15s;
}

.action-btn:hover:not(:disabled) {
  color: var(--text-primary);
  border-color: var(--text-muted);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.danger {
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

.action-btn.danger:hover:not(:disabled) {
  border-color: #ef4444;
}

.action-btn.download-btn {
  color: var(--color-work);
  border-color: rgba(16, 185, 129, 0.3);
}

.action-btn.download-btn:hover:not(:disabled) {
  border-color: var(--color-work);
}

.install-btn {
  color: #f59e0b;
  border-color: rgba(245, 158, 11, 0.3);
}

.install-btn:hover {
  border-color: #f59e0b;
}

.active-model {
  font-size: 13px;
  color: var(--text-secondary);
  padding: 8px 0;
}

.active-model strong {
  color: var(--text-primary);
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-bar-track {
  height: 6px;
  background: var(--bg-hover);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: var(--color-work);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-muted);
}

.progress-status {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 60%;
}

.empty-state {
  text-align: center;
  padding: 20px;
  color: var(--text-muted);
  font-size: 13px;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
