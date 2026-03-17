<script setup lang="ts">
import { ref } from "vue";
import { useSoundStore } from "@/stores/sound";

const sound = useSoundStore();

const showInput = ref(false);
const presetName = ref("");

function startSave() {
  showInput.value = true;
  presetName.value = "";
}

function cancelSave() {
  showInput.value = false;
  presetName.value = "";
}

async function confirmSave() {
  const name = presetName.value.trim();
  if (!name) return;
  await sound.savePreset(name);
  showInput.value = false;
  presetName.value = "";
}
</script>

<template>
  <div class="sound-presets">
    <div class="presets-header">
      <span class="presets-title">Presets</span>
      <button
        v-if="!showInput && sound.activeLayers.length > 0"
        class="save-btn"
        @click="startSave"
      >
        + Save
      </button>
    </div>

    <!-- Save input -->
    <div v-if="showInput" class="save-row">
      <input
        v-model="presetName"
        class="save-input"
        placeholder="Preset name..."
        @keyup.enter="confirmSave"
        @keyup.escape="cancelSave"
      />
      <button class="save-confirm" @click="confirmSave">Save</button>
      <button class="save-cancel" @click="cancelSave">Cancel</button>
    </div>

    <!-- Preset pills -->
    <div v-if="sound.presets.length > 0" class="presets-list">
      <div
        v-for="preset in sound.presets"
        :key="preset.id"
        class="preset-pill"
        :class="{ active: sound.activePresetId === preset.id }"
        @click="sound.loadPresetById(preset.id)"
      >
        <span class="preset-name">{{ preset.name }}</span>
        <button
          v-if="!preset.is_default"
          class="preset-delete"
          @click.stop="sound.deletePresetById(preset.id)"
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          >
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>

    <div v-else class="presets-empty">No presets yet</div>
  </div>
</template>

<style scoped>
.sound-presets {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.presets-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.presets-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.save-btn {
  background: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 12px;
  padding: 4px 10px;
  cursor: pointer;
  transition:
    color 0.15s,
    border-color 0.15s;
}

.save-btn:hover {
  color: var(--color-work);
  border-color: var(--color-work);
}

.save-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.save-input {
  flex: 1;
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 13px;
  padding: 6px 10px;
  outline: none;
}

.save-input:focus {
  border-color: var(--color-work);
}

.save-confirm {
  background: var(--color-work);
  border: none;
  border-radius: var(--radius-sm);
  color: var(--bg-primary);
  font-size: 12px;
  padding: 6px 12px;
  cursor: pointer;
}

.save-cancel {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 12px;
  padding: 6px 8px;
  cursor: pointer;
}

.presets-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.preset-pill {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-subtle);
  background: var(--bg-card);
  cursor: pointer;
  transition:
    border-color 0.15s,
    background 0.15s,
    transform 0.1s;
}

.preset-pill:hover {
  border-color: var(--border-default);
  background: var(--bg-hover);
}

.preset-pill:active {
  transform: scale(0.97);
}

.preset-pill.active {
  border-color: var(--color-work);
  background: var(--color-work-alpha-10);
}

.preset-name {
  font-size: 13px;
  color: var(--text-secondary);
}

.preset-pill.active .preset-name {
  color: var(--color-work);
}

.preset-delete {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  transition: color 0.15s;
}

.preset-delete:hover {
  color: var(--color-priority-high);
}

.presets-empty {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
