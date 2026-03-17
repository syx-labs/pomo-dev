<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue";
import { useCommandPalette, type PaletteIconName } from "@/composables/useCommandPalette";

const {
  isOpen,
  searchQuery,
  selectedIndex,
  isAiMode,
  isAiLoading,
  aiResult,
  filteredCommands,
  groupedCommands,
  close,
  selectNext,
  selectPrev,
  executeSelected,
  executeCommand,
  globalIndex,
} = useCommandPalette();

const inputRef = ref<HTMLInputElement | null>(null);
const containerRef = ref<HTMLDivElement | null>(null);
let previouslyFocused: HTMLElement | null = null;

watch(isOpen, async (open) => {
  if (open) {
    previouslyFocused = document.activeElement as HTMLElement | null;
    await nextTick();
    inputRef.value?.focus();
  } else {
    previouslyFocused?.focus();
    previouslyFocused = null;
  }
});

watch(searchQuery, () => {
  selectedIndex.value = 0;
});

function handleKeydown(e: KeyboardEvent) {
  if (!isOpen.value) return;

  if (e.key === "Tab" && containerRef.value) {
    const focusable = containerRef.value.querySelectorAll<HTMLElement>(
      'input, button:not([disabled]), [tabindex]:not([tabindex="-1"])',
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

function scrollSelectedIntoView() {
  nextTick(() => {
    const el = containerRef.value?.querySelector(".result-item.selected");
    el?.scrollIntoView({ block: "nearest" });
  });
}

watch(selectedIndex, () => scrollSelectedIntoView());

onMounted(() => {
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="palette">
      <div v-if="isOpen" class="palette-overlay" @mousedown.self="close">
        <div ref="containerRef" class="palette-container">
          <!-- Search input -->
          <div class="palette-input-wrapper">
            <svg class="search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none">
              <circle cx="7" cy="7" r="4.5" stroke="currentColor" stroke-width="1.5" fill="none" />
              <path
                d="M10.5 10.5L14 14"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
            <input
              ref="inputRef"
              v-model="searchQuery"
              class="palette-input"
              placeholder="Type a command..."
              spellcheck="false"
              @keydown.escape="close"
              @keydown.enter.prevent="executeSelected"
              @keydown.arrow-up.prevent="selectPrev"
              @keydown.arrow-down.prevent="selectNext"
            />
            <kbd v-if="!searchQuery" class="shortcut-hint">ESC</kbd>
          </div>

          <!-- AI mode indicator -->
          <div v-if="isAiMode" class="ai-mode-banner">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path
                d="M8 1l1.5 4.5L14 7l-4.5 1.5L8 13l-1.5-4.5L2 7l4.5-1.5L8 1z"
                fill="currentColor"
              />
            </svg>
            <span>AI Mode — describe what you want to do</span>
          </div>

          <!-- AI loading state -->
          <div v-if="isAiLoading" class="ai-loading">
            <div class="spinner" />
            <span>Thinking...</span>
          </div>

          <!-- AI result -->
          <div v-else-if="aiResult" class="ai-result" :class="{ success: aiResult.success }">
            <p>{{ aiResult.message }}</p>
          </div>

          <!-- Command list -->
          <div v-else-if="!isAiMode" class="palette-results">
            <template v-for="(group, category) in groupedCommands" :key="category">
              <div class="result-group">
                <div class="group-label">{{ category }}</div>
                <button
                  v-for="(cmd, i) in group"
                  :key="cmd.id"
                  class="result-item"
                  :class="{ selected: globalIndex(String(category), i) === selectedIndex }"
                  @click="executeCommand(cmd)"
                  @mouseenter="selectedIndex = globalIndex(String(category), i)"
                >
                  <span class="cmd-icon">
                    <!-- play -->
                    <svg
                      v-if="cmd.icon === 'play'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <path d="M4 2.5v11l9-5.5L4 2.5z" fill="currentColor" />
                    </svg>
                    <!-- pause -->
                    <svg
                      v-else-if="cmd.icon === 'pause'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <rect x="3" y="2" width="3.5" height="12" rx="1" fill="currentColor" />
                      <rect x="9.5" y="2" width="3.5" height="12" rx="1" fill="currentColor" />
                    </svg>
                    <!-- skip -->
                    <svg
                      v-else-if="cmd.icon === 'skip'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <path d="M2 2.5v11l7-5.5L2 2.5z" fill="currentColor" />
                      <rect x="11" y="2" width="3" height="12" rx="1" fill="currentColor" />
                    </svg>
                    <!-- reset -->
                    <svg
                      v-else-if="cmd.icon === 'reset'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <path
                        d="M2.5 8a5.5 5.5 0 1 1 1.3 3.56"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        fill="none"
                      />
                      <path
                        d="M2 4.5V7h2.5"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        fill="none"
                      />
                    </svg>
                    <!-- check -->
                    <svg
                      v-else-if="cmd.icon === 'check'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <path
                        d="M3 8.5l3.5 3.5L13 4"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        fill="none"
                      />
                    </svg>
                    <!-- plus -->
                    <svg
                      v-else-if="cmd.icon === 'plus'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <path
                        d="M8 3v10M3 8h10"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                      />
                    </svg>
                    <!-- search -->
                    <svg
                      v-else-if="cmd.icon === 'search'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <circle
                        cx="7"
                        cy="7"
                        r="4.5"
                        stroke="currentColor"
                        stroke-width="1.5"
                        fill="none"
                      />
                      <path
                        d="M10.5 10.5L14 14"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                      />
                    </svg>
                    <!-- timer -->
                    <svg
                      v-else-if="cmd.icon === 'timer'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <circle
                        cx="8"
                        cy="8.5"
                        r="5.5"
                        stroke="currentColor"
                        stroke-width="1.5"
                        fill="none"
                      />
                      <path
                        d="M8 5v4l2.5 1.5"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      />
                    </svg>
                    <!-- list -->
                    <svg
                      v-else-if="cmd.icon === 'list'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <path
                        d="M3 4h10M3 8h10M3 12h7"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                      />
                    </svg>
                    <!-- chart -->
                    <svg
                      v-else-if="cmd.icon === 'chart'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <rect x="2" y="9" width="3" height="5" rx="0.5" fill="currentColor" />
                      <rect x="6.5" y="5" width="3" height="9" rx="0.5" fill="currentColor" />
                      <rect x="11" y="2" width="3" height="12" rx="0.5" fill="currentColor" />
                    </svg>
                    <!-- gear -->
                    <svg
                      v-else-if="cmd.icon === 'gear'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <circle
                        cx="8"
                        cy="8"
                        r="2"
                        stroke="currentColor"
                        stroke-width="1.5"
                        fill="none"
                      />
                      <path
                        d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M3.4 12.6l1.4-1.4M11.2 4.8l1.4-1.4"
                        stroke="currentColor"
                        stroke-width="1.2"
                        stroke-linecap="round"
                      />
                    </svg>
                    <!-- sparkle -->
                    <svg
                      v-else-if="cmd.icon === 'sparkle'"
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                    >
                      <path
                        d="M8 1l1.5 4.5L14 7l-4.5 1.5L8 13l-1.5-4.5L2 7l4.5-1.5L8 1z"
                        fill="currentColor"
                      />
                    </svg>
                  </span>
                  <span class="cmd-label">{{ cmd.label }}</span>
                  <kbd v-if="cmd.shortcut" class="cmd-shortcut">{{ cmd.shortcut }}</kbd>
                </button>
              </div>
            </template>
            <div v-if="filteredCommands.length === 0" class="empty-state">No commands found</div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.palette-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  background: var(--overlay-bg-light);
  backdrop-filter: blur(4px);
}

.palette-container {
  width: 100%;
  max-width: 560px;
  margin-top: 20vh;
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-palette);
}

.palette-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--border-subtle);
}

.search-icon {
  position: absolute;
  left: 16px;
  color: var(--text-muted);
  pointer-events: none;
}

.palette-input {
  width: 100%;
  padding: 16px 16px 16px 44px;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-primary);
  font-family: var(--font-sans);
  font-size: 16px;
}

.palette-input::placeholder {
  color: var(--text-muted);
}

.shortcut-hint {
  position: absolute;
  right: 16px;
  background: var(--bg-card);
  color: var(--text-muted);
  padding: 2px 8px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 11px;
  pointer-events: none;
}

.ai-mode-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: var(--color-work-alpha-10);
  border-bottom: 1px solid var(--border-subtle);
  color: var(--color-work);
  font-family: var(--font-sans);
  font-size: 13px;
}

.ai-loading {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 24px 16px;
  color: var(--text-secondary);
  font-family: var(--font-sans);
  font-size: 14px;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-default);
  border-top-color: var(--color-work);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.ai-result {
  padding: 16px;
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.ai-result.success {
  color: var(--color-work);
}

.ai-result p {
  margin: 0;
}

.palette-results {
  max-height: 360px;
  overflow-y: auto;
  padding: 8px 0;
}

.result-group {
  /* no extra spacing needed */
}

.group-label {
  padding: 8px 16px 4px;
  font-family: var(--font-sans);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 16px;
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  font-family: var(--font-sans);
  font-size: 14px;
  text-align: left;
  transition: background 0.1s ease;
}

.result-item:hover,
.result-item.selected {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.cmd-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  color: var(--text-muted);
}

.result-item.selected .cmd-icon,
.result-item:hover .cmd-icon {
  color: var(--text-secondary);
}

.cmd-label {
  flex: 1;
  min-width: 0;
}

.cmd-shortcut {
  margin-left: auto;
  background: var(--bg-card);
  color: var(--text-muted);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 11px;
  flex-shrink: 0;
}

.empty-state {
  padding: 24px 16px;
  text-align: center;
  color: var(--text-muted);
  font-family: var(--font-sans);
  font-size: 14px;
}

/* Transitions */
.palette-enter-active,
.palette-leave-active {
  transition: opacity 0.15s ease-out;
}

.palette-enter-active .palette-container,
.palette-leave-active .palette-container {
  transition:
    transform 0.15s ease-out,
    opacity 0.15s ease-out;
}

.palette-enter-from,
.palette-leave-to {
  opacity: 0;
}

.palette-enter-from .palette-container,
.palette-leave-to .palette-container {
  opacity: 0;
  transform: scale(0.98) translateY(-8px);
}
</style>
