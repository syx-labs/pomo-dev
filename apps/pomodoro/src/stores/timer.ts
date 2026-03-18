import { defineStore } from "pinia";
import { ref, computed } from "vue";
import {
  startTimer,
  pauseTimer,
  resumeTimer,
  skipTimer,
  resetTimer,
  getTimerState,
  onTimerTick,
  onTimerComplete,
  startPomodoroForTask,
  completePomodoro,
  type TimerState,
} from "@/lib/tauri";
import { useToast } from "@/composables/useToast";
import { useSoundStore } from "@/stores/sound";
import { useAiStore } from "@/stores/ai";

export const useTimerStore = defineStore("timer", () => {
  const state = ref<TimerState>({
    status: "idle",
    session_type: "work",
    remaining_secs: 25 * 60,
    current_cycle: 1,
    total_cycles: 4,
  });

  const linkedTaskId = ref<string | null>(null);
  const linkedTaskTitle = ref<string | null>(null);

  const { showToast } = useToast();

  const isRunning = computed(() => state.value.status === "running");
  const isPaused = computed(() => state.value.status === "paused");
  const isIdle = computed(() => state.value.status === "idle");

  const minutes = computed(() => Math.floor(state.value.remaining_secs / 60));
  const seconds = computed(() => state.value.remaining_secs % 60);
  const displayTime = computed(
    () => `${String(minutes.value).padStart(2, "0")}:${String(seconds.value).padStart(2, "0")}`,
  );

  const progress = computed(() => {
    const total = getTotalDuration(state.value.session_type);
    if (total === 0) return 0;
    return 1 - state.value.remaining_secs / total;
  });

  const accentColor = computed(() => {
    switch (state.value.session_type) {
      case "short_break":
        return "short-break";
      case "long_break":
        return "long-break";
      default:
        return "work";
    }
  });

  const sessionLabel = computed(() => {
    switch (state.value.session_type) {
      case "short_break":
        return "Short Break";
      case "long_break":
        return "Long Break";
      default:
        return "Focus";
    }
  });

  // Total duration in seconds, derived from the last known timer state.
  // When remaining_secs equals the full duration (idle, just reset) we
  // can infer it.  While running/paused we fall back to a cached value
  // that is updated whenever we fetch fresh state from the backend.
  const totalDurations = ref<Record<string, number>>({
    work: 25 * 60,
    short_break: 5 * 60,
    long_break: 15 * 60,
  });

  function getTotalDuration(type: string): number {
    return totalDurations.value[type] ?? 25 * 60;
  }

  /** Re-read timer state from the backend (e.g. after settings change). */
  async function refreshState() {
    try {
      state.value = (await getTimerState()) as TimerState;
      // When idle the remaining_secs IS the total duration
      if (state.value.status === "idle") {
        totalDurations.value[state.value.session_type] = state.value.remaining_secs;
      }
    } catch {
      // non-fatal
    }
  }

  async function start() {
    try {
      state.value = (await startTimer()) as TimerState;
    } catch {
      showToast("Failed to start timer", "error");
    }
  }

  async function pause() {
    try {
      state.value = (await pauseTimer()) as TimerState;
    } catch {
      showToast("Failed to pause timer", "error");
    }
  }

  async function resume() {
    try {
      state.value = (await resumeTimer()) as TimerState;
    } catch {
      showToast("Failed to resume timer", "error");
    }
  }

  async function skip() {
    try {
      state.value = (await skipTimer()) as TimerState;
    } catch {
      showToast("Failed to skip session", "error");
    }
  }

  async function reset() {
    try {
      state.value = (await resetTimer()) as TimerState;
      linkedTaskId.value = null;
      linkedTaskTitle.value = null;
    } catch {
      showToast("Failed to reset timer", "error");
    }
  }

  async function finish() {
    try {
      state.value = (await completePomodoro()) as TimerState;
      linkedTaskId.value = null;
      linkedTaskTitle.value = null;
    } catch {
      showToast("Failed to complete pomodoro", "error");
    }
  }

  async function startForTask(taskId: string, taskTitle?: string) {
    try {
      state.value = (await startPomodoroForTask(taskId)) as TimerState;
      linkedTaskId.value = taskId;
      linkedTaskTitle.value = taskTitle ?? null;
    } catch {
      showToast("Failed to start pomodoro for task", "error");
    }
  }

  async function init() {
    try {
      state.value = (await getTimerState()) as TimerState;
      // Cache the total duration for the current session type
      if (state.value.status === "idle") {
        totalDurations.value[state.value.session_type] = state.value.remaining_secs;
      }
    } catch {
      showToast("Failed to load timer state", "warning");
    }
  }

  const unlistenTick = ref<(() => void) | null>(null);
  const unlistenComplete = ref<(() => void) | null>(null);

  async function setupListeners() {
    // Clean up any existing listeners before registering new ones
    cleanup();

    unlistenTick.value = await onTimerTick((payload) => {
      const prevStatus = state.value.status;
      state.value = payload;

      // Sync sound fade with timer state transitions
      if (prevStatus !== payload.status) {
        const soundStore = useSoundStore();
        if (!soundStore.isAnySoundPlaying) return;

        if (payload.status === "running") {
          void soundStore.fadeIn();
        } else if (payload.status === "paused" || payload.status === "idle") {
          void soundStore.fadeOut();
        }
      }
    });
    unlistenComplete.value = await onTimerComplete((payload) => {
      if (payload.sessionType === "work") {
        const aiStore = useAiStore();
        void aiStore.fetchDebrief(payload.sessionType);
      }
    });
  }

  function cleanup() {
    unlistenTick.value?.();
    unlistenComplete.value?.();
    unlistenTick.value = null;
    unlistenComplete.value = null;
  }

  return {
    state,
    isRunning,
    isPaused,
    isIdle,
    minutes,
    seconds,
    displayTime,
    progress,
    accentColor,
    sessionLabel,
    linkedTaskId,
    linkedTaskTitle,
    start,
    pause,
    resume,
    skip,
    reset,
    finish,
    startForTask,
    init,
    refreshState,
    setupListeners,
    cleanup,
  };
});
