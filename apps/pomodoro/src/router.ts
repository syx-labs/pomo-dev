import { createRouter, createWebHistory } from "vue-router";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", component: () => import("./views/TimerView.vue") },
    { path: "/tasks", component: () => import("./views/TasksView.vue") },
    { path: "/stats", component: () => import("./views/AnalyticsView.vue") },
    { path: "/settings", component: () => import("./views/SettingsView.vue") },
  ],
});
