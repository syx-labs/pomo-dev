<script setup lang="ts">
import { onMounted, onUnmounted, onErrorCaptured } from "vue";
import Titlebar from "./components/layout/Titlebar.vue";
import Sidebar from "./components/layout/Sidebar.vue";
import ToastContainer from "./components/layout/ToastContainer.vue";
import CommandPalette from "./components/CommandPalette.vue";
import { useTimerStore } from "@/stores/timer";
import { useSoundStore } from "@/stores/sound";
import { useAiStore } from "@/stores/ai";
import { useToast } from "@/composables/useToast";
import { useCommandPalette } from "@/composables/useCommandPalette";
import SessionDebrief from "@/components/ai/SessionDebrief.vue";

const timer = useTimerStore();
const soundStore = useSoundStore();
const aiStore = useAiStore();
const { showToast } = useToast();
const { toggle: togglePalette } = useCommandPalette();

function handleGlobalKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === "k") {
    e.preventDefault();
    togglePalette();
  }
}

onMounted(async () => {
  await timer.init();
  await timer.setupListeners();
  await soundStore.init();
  aiStore.fetchBriefing();
  document.addEventListener("keydown", handleGlobalKeydown);
});

onUnmounted(() => {
  timer.cleanup();
  document.removeEventListener("keydown", handleGlobalKeydown);
});

onErrorCaptured((err) => {
  console.error("Uncaught error:", err);
  showToast("An unexpected error occurred", "error");
  return false;
});
</script>

<template>
  <div class="app-shell">
    <Titlebar />
    <Sidebar />
    <main class="app-content">
      <RouterView v-slot="{ Component }">
        <Transition name="page" mode="out-in">
          <component :is="Component" />
        </Transition>
      </RouterView>
    </main>
    <SessionDebrief />
    <ToastContainer />
    <CommandPalette />
  </div>
</template>

<style scoped>
.app-shell {
  width: 100vw;
  height: 100vh;
  background: var(--bg-primary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-content {
  margin-top: var(--titlebar-height);
  margin-left: var(--sidebar-width);
  flex: 1;
  overflow-y: auto;
  height: calc(100vh - var(--titlebar-height));
}

/* Route transition */
.page-enter-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.page-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
