<script setup lang="ts"></script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-content" data-tauri-drag-region>
      <span class="titlebar-title" data-tauri-drag-region>Pomodoro</span>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: var(--titlebar-height);
  background: transparent;
  border-bottom: 1px solid var(--border-subtle);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding-left: 78px; /* macOS traffic light buttons */
  -webkit-app-region: drag;
}

.titlebar-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.titlebar-title {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 500;
}
</style>
