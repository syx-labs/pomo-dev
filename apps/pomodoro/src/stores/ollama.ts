import { defineStore } from "pinia";
import { ref, computed } from "vue";
import {
  ollamaCheckHealth,
  ollamaListLocalModels,
  ollamaGetCuratedModels,
  ollamaPullModel,
  ollamaCancelPull,
  ollamaDeleteModel,
  onOllamaPullProgress,
  onOllamaPullComplete,
  getSetting,
  setSetting,
  type OllamaModel,
  type CuratedModel,
  type PullProgress,
} from "@/lib/tauri";

export const useOllamaStore = defineStore("ollama", () => {
  const isRunning = ref<boolean | null>(null);
  const checking = ref(false);
  const localModels = ref<OllamaModel[]>([]);
  const curatedModels = ref<CuratedModel[]>([]);
  const activeModel = ref("");
  const pullingModel = ref<string | null>(null);
  const pullProgress = ref<PullProgress | null>(null);
  const deleting = ref<string | null>(null);

  const localModelNames = computed(() => new Set(localModels.value.map((m) => m.name)));

  const availableModels = computed(() =>
    curatedModels.value.filter((m) => !localModelNames.value.has(m.name)),
  );

  const downloadedCurated = computed(() =>
    curatedModels.value.filter((m) => localModelNames.value.has(m.name)),
  );

  async function checkHealth() {
    checking.value = true;
    const wasRunning = isRunning.value;
    try {
      isRunning.value = await ollamaCheckHealth();
    } catch {
      isRunning.value = false;
    } finally {
      checking.value = false;
    }
    // If Ollama just became available, load models and active model
    if (isRunning.value && !wasRunning) {
      await Promise.all([fetchLocalModels(), loadActiveModel()]);
    }
  }

  async function fetchLocalModels() {
    try {
      localModels.value = await ollamaListLocalModels();
    } catch {
      localModels.value = [];
    }
  }

  async function fetchCuratedModels() {
    try {
      curatedModels.value = await ollamaGetCuratedModels();
    } catch {
      curatedModels.value = [];
    }
  }

  async function loadActiveModel() {
    try {
      activeModel.value = (await getSetting("ai_model")) ?? "";
    } catch {
      activeModel.value = "";
    }
  }

  async function setActiveModel(name: string) {
    try {
      await setSetting("ai_model", name);
      activeModel.value = name;
    } catch {
      // State not updated if persist fails — keeps local and DB in sync
    }
  }

  async function pullModel(name: string) {
    pullingModel.value = name;
    pullProgress.value = null;
    try {
      await ollamaPullModel(name);
    } catch {
      pullingModel.value = null;
    }
  }

  async function cancelPull() {
    try {
      await ollamaCancelPull();
    } catch {
      // ignore — cancel is fire-and-forget
    }
  }

  async function deleteModel(name: string) {
    deleting.value = name;
    try {
      await ollamaDeleteModel(name);
      await fetchLocalModels();
      if (activeModel.value === name) {
        await setSetting("ai_model", "");
        activeModel.value = "";
      }
    } finally {
      deleting.value = null;
    }
  }

  const unlistenProgress = ref<(() => void) | null>(null);
  const unlistenComplete = ref<(() => void) | null>(null);

  async function setupEventListeners() {
    unlistenProgress.value?.();
    unlistenComplete.value?.();

    unlistenProgress.value = await onOllamaPullProgress((payload) => {
      pullProgress.value = payload;
    });

    unlistenComplete.value = await onOllamaPullComplete(async (payload) => {
      pullingModel.value = null;
      pullProgress.value = null;
      if (payload.success) {
        await fetchLocalModels();
      }
    });
  }

  async function init() {
    await setupEventListeners();
    // Fetch curated models (local) and check health (network) in parallel
    await Promise.all([fetchCuratedModels(), checkHealth()]);
    if (isRunning.value) {
      await Promise.all([fetchLocalModels(), loadActiveModel()]);
    }
  }

  return {
    // State (read-only from components)
    isRunning,
    checking,
    activeModel,
    pullingModel,
    pullProgress,
    deleting,
    // Computed
    availableModels,
    downloadedCurated,
    // Actions
    checkHealth,
    setActiveModel,
    pullModel,
    cancelPull,
    deleteModel,
    init,
  };
});
