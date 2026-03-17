<script setup lang="ts">
import { useAiStore } from "@/stores/ai";

const ai = useAiStore();
</script>

<template>
  <div class="weekly-report">
    <div class="report-header">
      <div class="report-title">
        <svg class="sparkle-icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 0L9.5 5.5L16 8L9.5 10.5L8 16L6.5 10.5L0 8L6.5 5.5Z" />
        </svg>
        <span>Weekly AI Report</span>
      </div>
      <button class="generate-btn" :disabled="ai.reportLoading" @click="ai.fetchReport()">
        <template v-if="ai.reportLoading">
          <svg class="spinner" width="14" height="14" viewBox="0 0 14 14">
            <circle
              cx="7"
              cy="7"
              r="5.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-dasharray="20 14"
            />
          </svg>
          Generating...
        </template>
        <template v-else>
          {{ ai.report ? "Regenerate" : "Generate" }}
        </template>
      </button>
    </div>

    <div v-if="ai.report" class="report-body">
      <p class="report-summary">{{ ai.report.summary }}</p>

      <div v-if="ai.report.highlights.length > 0" class="report-section">
        <h4 class="report-section-title">Highlights</h4>
        <ul class="highlights-list">
          <li v-for="(item, i) in ai.report.highlights" :key="i" class="highlight-item">
            {{ item }}
          </li>
        </ul>
      </div>

      <div v-if="ai.report.suggestions.length > 0" class="report-section">
        <h4 class="report-section-title">Suggestions</h4>
        <div class="suggestions-list">
          <div v-for="(item, i) in ai.report.suggestions" :key="i" class="suggestion-card">
            {{ item }}
          </div>
        </div>
      </div>
    </div>

    <div v-else-if="!ai.reportLoading" class="report-empty">
      Click Generate to create your weekly AI report
    </div>
  </div>
</template>

<style scoped>
.weekly-report {
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.report-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border-subtle);
}

.report-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.sparkle-icon {
  color: var(--color-work);
  flex-shrink: 0;
}

.generate-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  background: var(--color-work);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: opacity 0.15s ease;
}

.generate-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.generate-btn:active:not(:disabled) {
  transform: scale(0.97);
}

.generate-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.report-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.report-summary {
  font-size: 14px;
  line-height: 1.5;
  color: var(--text-secondary);
  margin: 0;
}

.report-section-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  margin: 0 0 8px;
}

.highlights-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.highlight-item {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.4;
  padding-left: 16px;
  position: relative;
}

.highlight-item::before {
  content: "";
  position: absolute;
  left: 0;
  top: 7px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-work);
}

.suggestions-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.suggestion-card {
  font-size: 13px;
  line-height: 1.4;
  color: var(--text-secondary);
  padding: 10px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
}

.report-empty {
  padding: 24px 16px;
  text-align: center;
  font-size: 13px;
  color: var(--text-muted);
}
</style>
