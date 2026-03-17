import { defineStore } from "pinia";
import { ref, computed, watch } from "vue";

export type ThemePreference = "light" | "dark" | "system";

const STORAGE_KEY = "theme-preference";

function getSystemTheme(): "light" | "dark" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export const useThemeStore = defineStore("theme", () => {
  const preference = ref<ThemePreference>(
    (localStorage.getItem(STORAGE_KEY) as ThemePreference) || "system",
  );

  const resolvedTheme = computed<"light" | "dark">(() => {
    if (preference.value === "system") {
      return getSystemTheme();
    }
    return preference.value;
  });

  function applyTheme() {
    document.documentElement.setAttribute("data-theme", resolvedTheme.value);
  }

  function setPreference(pref: ThemePreference) {
    preference.value = pref;
    localStorage.setItem(STORAGE_KEY, pref);
    applyTheme();
  }

  function cycleTheme() {
    const order: ThemePreference[] = ["system", "light", "dark"];
    const currentIndex = order.indexOf(preference.value);
    const nextIndex = (currentIndex + 1) % order.length;
    setPreference(order[nextIndex]);
  }

  // Watch for system theme changes
  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQuery.addEventListener("change", () => {
    if (preference.value === "system") {
      applyTheme();
    }
  });

  // Watch preference changes
  watch(preference, () => {
    applyTheme();
  });

  // Apply theme immediately on store creation
  applyTheme();

  return {
    preference,
    resolvedTheme,
    setPreference,
    cycleTheme,
  };
});
