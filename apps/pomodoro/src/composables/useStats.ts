import { ref, computed } from "vue";
import {
  getDailyStats,
  getWeeklyStats,
  getHeatmapData,
  getStreakData,
  getTimeOfDayStats,
  getProjectStats,
  getGoals,
  type DailyStats,
  type WeeklyStats,
  type HeatmapEntry,
  type StreakData,
  type HourlyStats,
  type ProjectStats,
  type Goal,
} from "../lib/tauri";

function formatDate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function getMondayOfWeek(d: Date): Date {
  const date = new Date(d);
  const day = date.getDay();
  // getDay(): 0=Sun, 1=Mon ... 6=Sat  ->  offset to Monday
  const diff = day === 0 ? -6 : 1 - day;
  date.setDate(date.getDate() + diff);
  return date;
}

export function formatSecs(totalSecs: number): string {
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export type SelectedPeriod = "week" | "month" | "3months" | "year";

function getPeriodRange(period: SelectedPeriod): { start: string; end: string } {
  const end = new Date();
  const start = new Date();
  switch (period) {
    case "week":
      start.setDate(end.getDate() - 7);
      break;
    case "month":
      start.setMonth(end.getMonth() - 1);
      break;
    case "3months":
      start.setMonth(end.getMonth() - 3);
      break;
    case "year":
      start.setFullYear(end.getFullYear() - 1);
      break;
  }
  return { start: formatDate(start), end: formatDate(end) };
}

export function useStats() {
  const daily = ref<DailyStats | null>(null);
  const weekly = ref<WeeklyStats | null>(null);
  const heatmapData = ref<HeatmapEntry[]>([]);
  const streakData = ref<StreakData | null>(null);
  const hourlyStats = ref<HourlyStats[]>([]);
  const projectStats = ref<ProjectStats[]>([]);
  const goals = ref<Goal[]>([]);
  const selectedPeriod = ref<SelectedPeriod>("month");
  const loading = ref(false);
  const currentDate = ref(new Date());

  const dateString = computed(() => formatDate(currentDate.value));

  const mondayString = computed(() => formatDate(getMondayOfWeek(currentDate.value)));

  const isToday = computed(() => {
    const now = new Date();
    return formatDate(now) === dateString.value;
  });

  const displayDate = computed(() => {
    if (isToday.value) return "Today";
    return currentDate.value.toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
  });

  async function today() {
    const result = await getDailyStats(dateString.value);
    daily.value = result;
    return result;
  }

  async function thisWeek() {
    const result = await getWeeklyStats(mondayString.value);
    weekly.value = result;
    return result;
  }

  async function fetchHeatmap(year: number) {
    heatmapData.value = await getHeatmapData(year);
  }

  async function fetchStreak() {
    streakData.value = await getStreakData();
  }

  async function fetchHourly(period: SelectedPeriod) {
    const { start, end } = getPeriodRange(period);
    hourlyStats.value = await getTimeOfDayStats(start, end);
  }

  async function fetchProjects(period: SelectedPeriod) {
    const { start, end } = getPeriodRange(period);
    projectStats.value = await getProjectStats(start, end);
  }

  async function fetchGoals() {
    goals.value = await getGoals();
  }

  async function fetchAll() {
    loading.value = true;
    try {
      await Promise.all([today(), thisWeek()]);
    } finally {
      loading.value = false;
    }
  }

  async function fetchAnalytics() {
    loading.value = true;
    try {
      await Promise.all([
        today(),
        thisWeek(),
        fetchHeatmap(new Date().getFullYear()),
        fetchStreak(),
        fetchHourly(selectedPeriod.value),
        fetchProjects(selectedPeriod.value),
        fetchGoals(),
      ]);
    } finally {
      loading.value = false;
    }
  }

  function goToPreviousDay() {
    const d = new Date(currentDate.value);
    d.setDate(d.getDate() - 1);
    currentDate.value = d;
  }

  function goToNextDay() {
    const d = new Date(currentDate.value);
    d.setDate(d.getDate() + 1);
    currentDate.value = d;
  }

  function goToToday() {
    currentDate.value = new Date();
  }

  return {
    daily,
    weekly,
    heatmapData,
    streakData,
    hourlyStats,
    projectStats,
    goals,
    selectedPeriod,
    loading,
    currentDate,
    dateString,
    mondayString,
    isToday,
    displayDate,
    today,
    thisWeek,
    fetchAll,
    fetchAnalytics,
    fetchHeatmap,
    fetchStreak,
    fetchHourly,
    fetchProjects,
    fetchGoals,
    goToPreviousDay,
    goToNextDay,
    goToToday,
    formatSecs,
  };
}
