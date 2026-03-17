<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAiStore } from "@/stores/ai";

const ai = useAiStore();

const STORAGE_KEY = "ai-briefing-collapsed";
const collapsed = ref(localStorage.getItem(STORAGE_KEY) === "true");

function toggleCollapsed() {
  collapsed.value = !collapsed.value;
  localStorage.setItem(STORAGE_KEY, String(collapsed.value));
}

onMounted(() => {
  ai.fetchBriefing();
});
</script>

<template>
  <div class="daily-briefing">
    <div class="briefing-header" @click="toggleCollapsed">
      <div class="briefing-title">
        <!-- Sparkle icon: 4-point star -->
        <svg class="sparkle-icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 0L9.5 5.5L16 8L9.5 10.5L8 16L6.5 10.5L0 8L6.5 5.5Z" />
        </svg>
        <span>Daily Briefing</span>
      </div>
      <div class="briefing-actions">
        <button
          v-if="!collapsed"
          class="icon-btn"
          title="Refresh"
          @click.stop="ai.refreshBriefing()"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M21.5 2v6h-6" />
            <path d="M2.5 22v-6h6" />
            <path d="M3.34 16A9 9 0 0 0 21.5 8" />
            <path d="M20.66 8A9 9 0 0 0 2.5 16" />
          </svg>
        </button>
        <button class="icon-btn chevron-btn" :class="{ rotated: collapsed }">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path
              d="M4 6l4 4 4-4"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
      </div>
    </div>

    <Transition name="collapse">
      <div v-if="!collapsed" class="briefing-body">
        <!-- Loading skeleton -->
        <div v-if="ai.briefingLoading" class="skeleton-wrap">
          <div class="skeleton-bar skeleton-bar--wide" />
          <div class="skeleton-bar skeleton-bar--narrow" />
        </div>

        <!-- Content -->
        <template v-else-if="ai.briefing">
          <p class="briefing-message">{{ ai.briefing.message }}</p>
          <p class="briefing-stats">{{ ai.briefing.stats_summary }}</p>
        </template>

        <!-- Empty state -->
        <p v-else class="briefing-empty">No briefing available yet.</p>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.daily-briefing {
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  overflow: hidden;
  max-width: 360px;
  width: 100%;
}

.briefing-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  cursor: pointer;
  user-select: none;
  transition: background 0.15s ease;
}

.briefing-header:hover {
  background: var(--bg-hover);
}

.briefing-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.sparkle-icon {
  color: var(--color-work);
  flex-shrink: 0;
}

.briefing-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.chevron-btn {
  transition:
    transform 0.2s ease,
    background 0.15s ease,
    color 0.15s ease;
}

.chevron-btn.rotated {
  transform: rotate(-90deg);
}

.briefing-body {
  padding: 0 14px 12px;
}

.briefing-message {
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
  margin: 0 0 8px;
}

.briefing-stats {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
}

.briefing-empty {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0;
}

/* Skeleton loading */
.skeleton-wrap {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-bar {
  height: 14px;
  background: var(--bg-hover);
  border-radius: 4px;
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}

.skeleton-bar--wide {
  width: 80%;
}

.skeleton-bar--narrow {
  width: 60%;
}

@keyframes skeleton-pulse {
  0%,
  100% {
    opacity: 0.5;
  }
  50% {
    opacity: 0.3;
  }
}

/* Collapse transition */
.collapse-enter-active {
  transition:
    opacity 0.2s ease,
    max-height 0.2s ease;
  overflow: hidden;
}

.collapse-leave-active {
  transition:
    opacity 0.15s ease,
    max-height 0.15s ease;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  max-height: 200px;
}
</style>
