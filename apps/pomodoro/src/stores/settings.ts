import { defineStore } from "pinia";
import { reactive, ref } from "vue";
import { getAllSettings, setSetting } from "@/lib/tauri";
import { useToast } from "@/composables/useToast";
import { useTimerStore } from "@/stores/timer";

export interface AppSettings {
  work_duration: number;
  short_break_duration: number;
  long_break_duration: number;
  cycles_before_long_break: number;
  auto_start_work: boolean;
  auto_start_break: boolean;
  sound_enabled: boolean;
  sound_volume: number;
  dnd_enabled: boolean;
  auto_launch: boolean;
  accent_color: string;
}

const defaults: AppSettings = {
  work_duration: 25,
  short_break_duration: 5,
  long_break_duration: 15,
  cycles_before_long_break: 4,
  auto_start_work: false,
  auto_start_break: false,
  sound_enabled: true,
  sound_volume: 80,
  dnd_enabled: false,
  auto_launch: false,
  accent_color: "emerald",
};

// Validation ranges for numeric settings (values are in minutes for durations)
const CLAMP_RANGES: Partial<Record<keyof AppSettings, [number, number]>> = {
  work_duration: [1, 120], // 1-120 minutes (60-7200 seconds)
  short_break_duration: [1, 30], // 1-30 minutes (60-1800 seconds)
  long_break_duration: [1, 60], // 1-60 minutes (60-3600 seconds)
  cycles_before_long_break: [1, 10],
  sound_volume: [0, 100],
};

function clampValue<K extends keyof AppSettings>(key: K, value: AppSettings[K]): AppSettings[K] {
  const range = CLAMP_RANGES[key];
  if (range && typeof value === "number") {
    return Math.min(Math.max(value, range[0]), range[1]) as AppSettings[K];
  }
  return value;
}

export const useSettingsStore = defineStore("settings", () => {
  const settings = reactive<AppSettings>({ ...defaults });
  const loaded = ref(false);

  const { showToast } = useToast();

  async function load() {
    try {
      const raw = await getAllSettings();
      for (const [key, value] of Object.entries(raw)) {
        if (key in settings) {
          const typedKey = key as keyof AppSettings;
          const defaultVal = defaults[typedKey];
          if (typeof defaultVal === "boolean") {
            (settings as Record<string, unknown>)[key] = value === "true";
          } else if (typeof defaultVal === "number") {
            const num = Number(value);
            (settings as Record<string, unknown>)[key] = clampValue(
              typedKey,
              (Number.isNaN(num) ? defaultVal : num) as AppSettings[typeof typedKey],
            );
          } else {
            (settings as Record<string, unknown>)[key] = value;
          }
        }
      }
    } catch {
      showToast("Failed to load settings", "warning");
    }
    loaded.value = true;
  }

  const TIMER_KEYS: readonly string[] = [
    "work_duration",
    "short_break_duration",
    "long_break_duration",
    "cycles_before_long_break",
  ];

  async function update<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    const clamped = clampValue(key, value);
    const previous = settings[key];
    settings[key] = clamped;
    try {
      await setSetting(key, String(clamped));
      // Sync frontend timer state when timer-related settings change
      if (TIMER_KEYS.includes(key)) {
        const timerStore = useTimerStore();
        await timerStore.refreshState();
      }
    } catch {
      // Revert on failure
      settings[key] = previous;
      showToast("Failed to save setting", "error");
    }
  }

  async function resetToDefaults() {
    const previousSettings = { ...settings };
    const persistedKeys: string[] = [];
    try {
      for (const [key, value] of Object.entries(defaults)) {
        (settings as Record<string, unknown>)[key] = value;
        await setSetting(key, String(value));
        persistedKeys.push(key);
      }
      const timerStore = useTimerStore();
      await timerStore.refreshState();
    } catch {
      // Revert in-memory state
      Object.assign(settings, previousSettings);
      // Best-effort revert of persisted keys
      for (const key of persistedKeys.reverse()) {
        try {
          await setSetting(
            key,
            String((previousSettings as Record<string, unknown>)[key]),
          );
        } catch {
          // ignore rollback failure
        }
      }
      showToast("Failed to reset settings", "error");
    }
  }

  return { settings, loaded, load, update, resetToDefaults };
});
