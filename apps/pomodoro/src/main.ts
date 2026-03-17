import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router";
import "./styles/global.css";
import { useThemeStore } from "./stores/theme";

const app = createApp(App);

const pinia = createPinia();
app.use(pinia);
app.use(router);

// Initialize theme before mount to prevent flash of wrong theme
useThemeStore();

app.mount("#app");
