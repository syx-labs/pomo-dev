import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useTimerStore } from "@/stores/timer";
import { useTasksStore } from "@/stores/tasks";
import { useToast } from "@/composables/useToast";
import { invokeAiCommand, type AiCommandResult } from "@/lib/tauri";

export type PaletteIconName =
  | "play"
  | "pause"
  | "skip"
  | "reset"
  | "check"
  | "plus"
  | "search"
  | "timer"
  | "list"
  | "chart"
  | "gear"
  | "sparkle";

export interface Command {
  id: string;
  label: string;
  category: "timer" | "tasks" | "navigation" | "settings" | "ai";
  icon: PaletteIconName;
  shortcut?: string;
  action: () => void | Promise<void>;
  when?: () => boolean;
}

function fuzzyScore(query: string, target: string): number {
  const q = query.toLowerCase();
  const t = target.toLowerCase();
  if (t === q) return 100;
  if (t.startsWith(q)) return 80;
  if (t.includes(q)) return 50;
  let qi = 0;
  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) qi++;
  }
  return qi === q.length ? 30 : 0;
}

// Module-level singleton state
const isOpen = ref(false);
const searchQuery = ref("");
const selectedIndex = ref(0);
const isAiLoading = ref(false);
const aiResult = ref<AiCommandResult | null>(null);
const customCommands = ref<Command[]>([]);

export function useCommandPalette() {
  const router = useRouter();
  const timer = useTimerStore();
  const tasks = useTasksStore();
  const { showToast } = useToast();

  const isAiMode = computed(() => searchQuery.value.startsWith("/ai "));

  const defaultCommands = computed<Command[]>(() => [
    // Timer commands
    {
      id: "timer:start",
      label: "Start Timer",
      category: "timer",
      icon: "play",
      shortcut: "Space",
      action: () => timer.start(),
      when: () => timer.isIdle,
    },
    {
      id: "timer:pause",
      label: "Pause Timer",
      category: "timer",
      icon: "pause",
      shortcut: "Space",
      action: () => timer.pause(),
      when: () => timer.isRunning,
    },
    {
      id: "timer:resume",
      label: "Resume Timer",
      category: "timer",
      icon: "play",
      shortcut: "Space",
      action: () => timer.resume(),
      when: () => timer.isPaused,
    },
    {
      id: "timer:skip",
      label: "Skip Session",
      category: "timer",
      icon: "skip",
      action: () => timer.skip(),
      when: () => !timer.isIdle,
    },
    {
      id: "timer:reset",
      label: "Reset Timer",
      category: "timer",
      icon: "reset",
      action: () => timer.reset(),
      when: () => !timer.isIdle,
    },
    {
      id: "timer:finish",
      label: "Finish Pomodoro",
      category: "timer",
      icon: "check",
      action: () => timer.finish(),
      when: () => timer.isRunning,
    },
    // Task commands
    {
      id: "tasks:create",
      label: "Create New Task",
      category: "tasks",
      icon: "plus",
      shortcut: "N",
      action: () => {
        void router.push("/tasks");
      },
    },
    {
      id: "tasks:search",
      label: "Search Tasks...",
      category: "tasks",
      icon: "search",
      action: () => {
        void router.push("/tasks");
      },
    },
    // Navigation commands
    {
      id: "nav:timer",
      label: "Go to Timer",
      category: "navigation",
      icon: "timer",
      action: () => {
        void router.push("/");
      },
    },
    {
      id: "nav:tasks",
      label: "Go to Tasks",
      category: "navigation",
      icon: "list",
      action: () => {
        void router.push("/tasks");
      },
    },
    {
      id: "nav:analytics",
      label: "Go to Analytics",
      category: "navigation",
      icon: "chart",
      action: () => {
        void router.push("/stats");
      },
    },
    {
      id: "nav:settings",
      label: "Go to Settings",
      category: "navigation",
      icon: "gear",
      action: () => {
        void router.push("/settings");
      },
    },
  ]);

  const commands = computed(() => [...defaultCommands.value, ...customCommands.value]);

  const filteredCommands = computed(() => {
    if (isAiMode.value) return [];
    const query = searchQuery.value.trim();
    if (!query) {
      return commands.value.filter((cmd) => !cmd.when || cmd.when());
    }
    return commands.value
      .filter((cmd) => !cmd.when || cmd.when())
      .map((cmd) => ({ cmd, score: fuzzyScore(query, cmd.label) }))
      .filter(({ score }) => score > 0)
      .sort((a, b) => b.score - a.score)
      .map(({ cmd }) => cmd);
  });

  const groupedCommands = computed(() => {
    const groups: Record<string, Command[]> = {};
    for (const cmd of filteredCommands.value) {
      if (!groups[cmd.category]) groups[cmd.category] = [];
      groups[cmd.category].push(cmd);
    }
    return groups;
  });

  function open() {
    isOpen.value = true;
    searchQuery.value = "";
    selectedIndex.value = 0;
    aiResult.value = null;
    isAiLoading.value = false;
  }

  function close() {
    if (aiResult.value) {
      aiResult.value = null;
      return;
    }
    isOpen.value = false;
  }

  function toggle() {
    if (isOpen.value) {
      isOpen.value = false;
    } else {
      open();
    }
  }

  function selectNext() {
    const count = filteredCommands.value.length;
    if (count === 0) return;
    selectedIndex.value = (selectedIndex.value + 1) % count;
  }

  function selectPrev() {
    const count = filteredCommands.value.length;
    if (count === 0) return;
    selectedIndex.value = (selectedIndex.value - 1 + count) % count;
  }

  async function executeCommand(cmd: Command) {
    isOpen.value = false;
    await cmd.action();
  }

  async function executeSelected() {
    if (isAiMode.value) {
      const prompt = searchQuery.value.slice(4).trim();
      if (!prompt) return;
      await executeAi(prompt);
      return;
    }
    const cmd = filteredCommands.value[selectedIndex.value];
    if (cmd) await executeCommand(cmd);
  }

  async function executeAi(prompt: string) {
    isAiLoading.value = true;
    aiResult.value = null;
    try {
      const result = await invokeAiCommand(prompt);
      aiResult.value = result;
      if (result.success) {
        await handleAiAction(result);
      }
    } catch {
      showToast("Configure an AI provider in Settings → AI", "warning");
      isOpen.value = false;
    } finally {
      isAiLoading.value = false;
    }
  }

  async function handleAiAction(result: AiCommandResult) {
    switch (result.action) {
      case "create_task":
        await tasks.addTask({
          title: result.params.title ?? "New Task",
          description: result.params.description,
          priority: result.params.priority ? Number(result.params.priority) : undefined,
          project: result.params.project,
        });
        showToast(result.message, "success");
        isOpen.value = false;
        break;
      case "start_timer":
        await timer.start();
        showToast(result.message, "success");
        isOpen.value = false;
        break;
      case "pause_timer":
        await timer.pause();
        showToast(result.message, "success");
        isOpen.value = false;
        break;
      case "navigate": {
        const path = result.params.path;
        if (path) void router.push(path);
        isOpen.value = false;
        break;
      }
      default:
        showToast(result.message, "success");
        isOpen.value = false;
    }
  }

  function registerCommand(cmd: Command) {
    customCommands.value.push(cmd);
  }

  function unregisterCommand(id: string) {
    const idx = customCommands.value.findIndex((c) => c.id === id);
    if (idx !== -1) customCommands.value.splice(idx, 1);
  }

  function globalIndex(category: string, localIdx: number): number {
    const categories = Object.keys(groupedCommands.value);
    let offset = 0;
    for (const cat of categories) {
      if (cat === category) return offset + localIdx;
      offset += groupedCommands.value[cat].length;
    }
    return 0;
  }

  return {
    isOpen,
    searchQuery,
    selectedIndex,
    isAiMode,
    isAiLoading,
    aiResult,
    commands,
    filteredCommands,
    groupedCommands,
    open,
    close,
    toggle,
    selectNext,
    selectPrev,
    executeSelected,
    executeCommand,
    executeAi,
    registerCommand,
    unregisterCommand,
    globalIndex,
  };
}
