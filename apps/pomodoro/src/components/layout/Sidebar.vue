<script setup lang="ts">
import { useRoute } from "vue-router";
import { useThemeStore } from "@/stores/theme";

const route = useRoute();
const themeStore = useThemeStore();

const navItems = [
  { path: "/", label: "Timer", icon: "timer" },
  { path: "/tasks", label: "Tasks", icon: "tasks" },
  { path: "/stats", label: "Stats", icon: "stats" },
] as const;

function isActive(path: string): boolean {
  return route.path === path;
}
</script>

<template>
  <nav class="sidebar">
    <div class="sidebar-top">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="sidebar-btn"
        :class="{ active: isActive(item.path) }"
        :title="item.label"
      >
        <!-- Timer / Clock icon -->
        <svg
          v-if="item.icon === 'timer'"
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="13" r="8" />
          <path d="M12 9v4l2.5 1.5" />
          <path d="M5 3L2 6" />
          <path d="M22 6l-3-3" />
          <path d="M12 2v2" />
        </svg>

        <!-- Tasks / Checklist icon -->
        <svg
          v-else-if="item.icon === 'tasks'"
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M4 7h16" />
          <path d="M4 12h16" />
          <path d="M4 17h10" />
          <rect x="2" y="5" width="2" height="2" rx="0.5" fill="currentColor" stroke="none" />
          <rect x="2" y="10" width="2" height="2" rx="0.5" fill="currentColor" stroke="none" />
          <rect x="2" y="15" width="2" height="2" rx="0.5" fill="currentColor" stroke="none" />
        </svg>

        <!-- Stats / Chart icon -->
        <svg
          v-else-if="item.icon === 'stats'"
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="3" y="12" width="4" height="9" rx="1" />
          <rect x="10" y="7" width="4" height="14" rx="1" />
          <rect x="17" y="3" width="4" height="18" rx="1" />
        </svg>
      </router-link>
    </div>

    <div class="sidebar-bottom">
      <button
        class="sidebar-btn"
        :title="`Theme: ${themeStore.preference}`"
        @click="themeStore.cycleTheme()"
      >
        <!-- Sun icon (light) -->
        <svg
          v-if="themeStore.resolvedTheme === 'light'"
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="5" />
          <path
            d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"
          />
        </svg>
        <!-- Moon icon (dark) -->
        <svg
          v-else
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
        </svg>
      </button>
      <router-link
        to="/settings"
        class="sidebar-btn"
        :class="{ active: isActive('/settings') }"
        title="Settings"
      >
        <!-- Settings / Gear icon -->
        <svg
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="3" />
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
          />
        </svg>
      </router-link>
    </div>
  </nav>
</template>

<style scoped>
.sidebar {
  position: fixed;
  top: var(--titlebar-height);
  left: 0;
  bottom: 0;
  width: var(--sidebar-width);
  background: var(--bg-primary);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 8px 0;
  z-index: 90;
}

.sidebar-top {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.sidebar-bottom {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.sidebar-btn {
  position: relative;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  color: var(--text-muted);
  transition:
    background 0.15s ease,
    color 0.15s ease,
    transform 0.1s ease;
}

.sidebar-btn:hover {
  background: var(--bg-secondary);
  color: var(--text-secondary);
}

.sidebar-btn:active {
  transform: scale(0.92);
}

.sidebar-btn.active {
  background: var(--bg-card);
  color: var(--color-work);
}

.sidebar-btn.active::before {
  content: "";
  position: absolute;
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  background: var(--color-work);
  border-radius: 0 2px 2px 0;
}
</style>
