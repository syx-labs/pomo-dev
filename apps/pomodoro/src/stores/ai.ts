import { defineStore } from "pinia";
import { ref, computed } from "vue";
import {
  getDailyBriefing,
  getSessionDebrief,
  getWeeklyReport,
  getSettingsAdvice,
  type AiBriefing,
  type AiDebrief,
  type AiReport,
  type SettingAdvice,
} from "@/lib/tauri";

const CACHE_DURATION_MS = 60 * 60 * 1000; // 1 hour

export const useAiStore = defineStore("ai", () => {
  // Briefing
  const briefing = ref<AiBriefing | null>(null);
  const briefingLoading = ref(false);
  const briefingLastFetched = ref<number | null>(null);

  // Debrief
  const debrief = ref<AiDebrief | null>(null);
  const debriefVisible = ref(false);

  // Report
  const report = ref<AiReport | null>(null);
  const reportLoading = ref(false);

  // Advice
  const advice = ref<SettingAdvice[] | null>(null);
  const adviceLoading = ref(false);

  // Computed
  const hasBriefing = computed(() => briefing.value !== null);
  const isBriefingStale = computed(() => {
    if (!briefingLastFetched.value) return true;
    return Date.now() - briefingLastFetched.value > CACHE_DURATION_MS;
  });

  async function fetchBriefing() {
    if (!isBriefingStale.value && briefing.value) return;
    briefingLoading.value = true;
    try {
      briefing.value = await getDailyBriefing();
      briefingLastFetched.value = Date.now();
    } catch {
      // Silently fail — briefing is non-critical
    } finally {
      briefingLoading.value = false;
    }
  }

  async function refreshBriefing() {
    briefingLastFetched.value = null;
    await fetchBriefing();
  }

  async function fetchDebrief(sessionType: string) {
    try {
      debrief.value = await getSessionDebrief(sessionType);
      debriefVisible.value = true;
    } catch {
      // Silently fail — debrief is non-critical
    }
  }

  function dismissDebrief() {
    debriefVisible.value = false;
  }

  async function fetchReport() {
    reportLoading.value = true;
    try {
      report.value = await getWeeklyReport();
    } catch {
      // Silently fail
    } finally {
      reportLoading.value = false;
    }
  }

  async function fetchAdvice() {
    adviceLoading.value = true;
    try {
      advice.value = await getSettingsAdvice();
    } catch {
      // Silently fail
    } finally {
      adviceLoading.value = false;
    }
  }

  return {
    briefing,
    briefingLoading,
    briefingLastFetched,
    debrief,
    debriefVisible,
    report,
    reportLoading,
    advice,
    adviceLoading,
    hasBriefing,
    isBriefingStale,
    fetchBriefing,
    refreshBriefing,
    fetchDebrief,
    dismissDebrief,
    fetchReport,
    fetchAdvice,
  };
});
