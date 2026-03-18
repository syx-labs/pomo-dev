import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// IPC Error class
export class IpcError extends Error {
  code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = "IpcError";
    this.code = code;
  }
}

// Read-only commands that are safe to retry
const RETRYABLE_COMMANDS = new Set([
  "list_tasks",
  "get_timer_state",
  "get_all_settings",
  "get_daily_stats",
  "get_weekly_stats",
  "get_sessions_for_task",
  "get_heatmap_data",
  "get_streak_data",
  "get_time_of_day_stats",
  "get_project_stats",
  "get_goals",
  "get_audio_state",
  "get_available_sounds",
  "list_sound_presets",
  "list_integrations",
  "get_event_log",
  "get_daily_briefing",
  "get_session_debrief",
  "get_weekly_report",
  "get_settings_advice",
  "ollama_check_health",
  "ollama_list_local_models",
  "ollama_get_curated_models",
]);

const RETRY_DELAY_MS = 500;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function ipc<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    // Retry once for read-only commands
    if (RETRYABLE_COMMANDS.has(command)) {
      await sleep(RETRY_DELAY_MS);
      try {
        return await invoke<T>(command, args);
      } catch (retryErr) {
        throw toIpcError(command, retryErr);
      }
    }
    throw toIpcError(command, err);
  }
}

function toIpcError(command: string, err: unknown): IpcError {
  if (err instanceof IpcError) return err;
  const message = err instanceof Error ? err.message : typeof err === "string" ? err : String(err);
  return new IpcError(command, message);
}

// Timer commands
export const startTimer = () => ipc<TimerState>("start_timer");
export const pauseTimer = () => ipc<TimerState>("pause_timer");
export const resumeTimer = () => ipc<TimerState>("resume_timer");
export const skipTimer = () => ipc<TimerState>("skip_timer");
export const resetTimer = () => ipc<TimerState>("reset_timer");
export const getTimerState = () => ipc<TimerState>("get_timer_state");

// Task commands
export const createTask = (input: CreateTaskInput) => ipc<Task>("create_task", { input });
export const updateTask = (input: UpdateTaskInput) => ipc<Task>("update_task", { input });
export const deleteTask = (id: string) => ipc<void>("delete_task", { id });
export const listTasks = (status?: string, project?: string) =>
  ipc<Task[]>("list_tasks", { status, project });
export const reorderTasks = (taskIds: string[]) => ipc<void>("reorder_tasks", { taskIds });

// Pomodoro commands
export const startPomodoroForTask = (taskId: string) =>
  ipc<TimerState>("start_pomodoro_for_task", { taskId });
export const completePomodoro = () => ipc<TimerState>("complete_pomodoro");
export const getSessionsForTask = (taskId: string) =>
  ipc<PomodoroSession[]>("get_sessions_for_task", { taskId });

// Settings commands
export const getSetting = (key: string) => ipc<string | null>("get_setting", { key });
export const setSetting = (key: string, value: string) => ipc<void>("set_setting", { key, value });
export const getAllSettings = () => ipc<Record<string, string>>("get_all_settings");

// Stats commands
export const getDailyStats = (date: string) => ipc<DailyStats>("get_daily_stats", { date });
export const getWeeklyStats = (startDate: string) =>
  ipc<WeeklyStats>("get_weekly_stats", { startDate });

// Analytics commands
export const getHeatmapData = (year: number) => ipc<HeatmapEntry[]>("get_heatmap_data", { year });
export const getStreakData = () => ipc<StreakData>("get_streak_data");
export const getTimeOfDayStats = (startDate: string, endDate: string) =>
  ipc<HourlyStats[]>("get_time_of_day_stats", { startDate, endDate });
export const getProjectStats = (startDate: string, endDate: string) =>
  ipc<ProjectStats[]>("get_project_stats", { startDate, endDate });
export const setGoal = (goalType: string, target: number) =>
  ipc<Goal>("set_goal", { goalType, target });
export const getGoals = () => ipc<Goal[]>("get_goals");

// Event listeners
export const onTimerTick = (callback: (payload: TimerState) => void) =>
  listen<TimerState>("timer:tick", (event) => callback(event.payload));

export const onTimerComplete = (callback: (payload: { sessionType: string }) => void) =>
  listen<{ sessionType: string }>("timer:complete", (event) => callback(event.payload));

// Types
export interface Task {
  id: string;
  title: string;
  description: string | null;
  status: "todo" | "in_progress" | "done" | "archived";
  priority: number;
  project: string | null;
  due_date: string | null;
  estimated_pomos: number;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface CreateTaskInput {
  title: string;
  description?: string;
  priority?: number;
  project?: string;
  due_date?: string;
  estimated_pomos?: number;
}

export interface UpdateTaskInput {
  id: string;
  title: string;
  description: string | null;
  status: string;
  priority: number;
  project: string | null;
  due_date: string | null;
  estimated_pomos: number;
  sort_order: number;
}

export interface TaskFilters {
  status?: string;
  project?: string;
  priority?: number;
}

export interface PomodoroSession {
  id: string;
  task_id: string | null;
  session_type: string;
  duration_secs: number;
  started_at: string;
  ended_at: string | null;
  completed: boolean;
  notes: string | null;
}

export interface TimerState {
  status: "idle" | "running" | "paused";
  session_type: string;
  remaining_secs: number;
  current_cycle: number;
  total_cycles: number;
}

export interface DailyStats {
  date: string;
  total_sessions: number;
  total_work_secs: number;
  total_break_secs: number;
}

export interface WeeklyStats {
  days: DailyStats[];
  total_sessions: number;
  total_work_secs: number;
}

export interface HeatmapEntry {
  date: string;
  count: number;
}

export interface StreakData {
  current: number;
  best: number;
}

export interface HourlyStats {
  hour: number;
  count: number;
}

export interface ProjectStats {
  project: string;
  total_sessions: number;
  total_work_secs: number;
}

export interface Goal {
  id: string;
  goal_type: string;
  target: number;
  created_at: string;
}

export interface AiCommandResult {
  action: string;
  params: Record<string, string>;
  message: string;
  success: boolean;
}

export const invokeAiCommand = (prompt: string) =>
  ipc<AiCommandResult>("invoke_ai_command", { prompt });

// AI Coach types
export interface AiBriefing {
  message: string;
  stats_summary: string;
}

export interface AiDebrief {
  message: string;
}

export interface AiReport {
  highlights: string[];
  suggestions: string[];
  summary: string;
}

export interface SettingAdvice {
  setting: string;
  current: number;
  suggested: number;
  reason: string;
}

// AI Coach commands
export const getDailyBriefing = () => ipc<AiBriefing>("get_daily_briefing");
export const getSessionDebrief = (sessionType: string, consecutiveCount: number) =>
  ipc<AiDebrief>("get_session_debrief", { sessionType, consecutiveCount });
export const getWeeklyReport = () => ipc<AiReport>("get_weekly_report");
export const getSettingsAdvice = () => ipc<SettingAdvice[]>("get_settings_advice");

// Audio types
export interface LayerState {
  name: string;
  volume: number;
  is_playing: boolean;
}

export interface AudioState {
  layers: LayerState[];
  master_volume: number;
}

export interface SoundPreset {
  id: string;
  name: string;
  layers: string; // JSON string of PresetLayer[]
  is_default: boolean;
  created_at: string;
}

export interface PresetLayer {
  sound: string;
  volume: number;
}

// Audio commands
export const playSound = (name: string, volume: number) =>
  ipc<void>("play_sound", { name, volume });
export const stopSound = (name: string) => ipc<void>("stop_sound", { name });
export const setSoundVolume = (name: string, volume: number) =>
  ipc<void>("set_sound_volume", { name, volume });
export const setMasterVolume = (volume: number) => ipc<void>("set_master_volume", { volume });
export const fadeSounds = (direction: string) => ipc<void>("fade_sounds", { direction });
export const stopAllSounds = () => ipc<void>("stop_all_sounds");
export const getAudioState = () => ipc<AudioState>("get_audio_state");
export const getAvailableSounds = () => ipc<string[]>("get_available_sounds");

// Sound preset commands
export const saveSoundPreset = (name: string, layers: string) =>
  ipc<SoundPreset>("save_sound_preset", { name, layers });
export const listSoundPresets = () => ipc<SoundPreset[]>("list_sound_presets");
export const loadSoundPreset = (id: string) => ipc<void>("load_sound_preset", { id });
export const deleteSoundPreset = (id: string) => ipc<void>("delete_sound_preset", { id });

// Integration types
export interface IntegrationConfig {
  id: string;
  integration_type: string;
  name: string;
  config: string;
  enabled: boolean;
  events: string;
  created_at: string;
  updated_at: string;
}

export interface EventLogEntry {
  id: string;
  integration_id: string;
  event_type: string;
  payload: string;
  status: string;
  error_message: string | null;
  created_at: string;
}

// Integration commands
export const createIntegration = (
  integrationType: string,
  name: string,
  config: string,
  events: string,
) => ipc<IntegrationConfig>("create_integration", { integrationType, name, config, events });

export const updateIntegration = (
  id: string,
  name: string,
  config: string,
  events: string,
  enabled: boolean,
) => ipc<void>("update_integration", { id, name, config, events, enabled });

export const deleteIntegration = (id: string) => ipc<void>("delete_integration", { id });

export const listIntegrations = () => ipc<IntegrationConfig[]>("list_integrations");

export const testIntegration = (id: string) => ipc<string>("test_integration", { id });

export const getEventLog = (integrationId?: string, limit?: number) =>
  ipc<EventLogEntry[]>("get_event_log", { integrationId, limit });

// Ollama model management types
export interface OllamaModel {
  name: string;
  size: number;
  modified_at: string;
}

export type ModelCategory = "fast" | "balanced" | "capable";

export interface CuratedModel {
  name: string;
  display_name: string;
  description: string;
  size_mb: number;
  category: ModelCategory;
}

export interface PullProgress {
  model: string;
  status: string;
  total: number;
  completed: number;
  percent: number;
}

export interface PullComplete {
  model: string;
  success: boolean;
  error: string | null;
}

// Ollama commands
export const ollamaCheckHealth = () => ipc<boolean>("ollama_check_health");
export const ollamaListLocalModels = () => ipc<OllamaModel[]>("ollama_list_local_models");
export const ollamaGetCuratedModels = () => ipc<CuratedModel[]>("ollama_get_curated_models");
export const ollamaPullModel = (name: string) => ipc<void>("ollama_pull_model", { name });
export const ollamaCancelPull = () => ipc<void>("ollama_cancel_pull");
export const ollamaDeleteModel = (name: string) => ipc<void>("ollama_delete_model", { name });

// Ollama event listeners
export const onOllamaPullProgress = (callback: (payload: PullProgress) => void) =>
  listen<PullProgress>("ollama:pull-progress", (event) => callback(event.payload));

export const onOllamaPullComplete = (callback: (payload: PullComplete) => void) =>
  listen<PullComplete>("ollama:pull-complete", (event) => callback(event.payload));
