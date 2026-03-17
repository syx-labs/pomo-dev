import { computed } from "vue";
import { useSettingsStore } from "@/stores/settings";

export function useSettings() {
  const store = useSettingsStore();

  const workMinutes = computed(() => store.settings.work_duration);
  const shortBreakMinutes = computed(() => store.settings.short_break_duration);
  const longBreakMinutes = computed(() => store.settings.long_break_duration);
  const totalCycles = computed(() => store.settings.cycles_before_long_break);

  const timerSummary = computed(
    () =>
      `${workMinutes.value}m work / ${shortBreakMinutes.value}m break / ${totalCycles.value} cycles`,
  );

  return {
    store,
    workMinutes,
    shortBreakMinutes,
    longBreakMinutes,
    totalCycles,
    timerSummary,
  };
}
