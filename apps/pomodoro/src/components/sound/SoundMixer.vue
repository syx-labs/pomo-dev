<script setup lang="ts">
import { ref } from "vue";
import { useSoundStore } from "@/stores/sound";

const sound = useSoundStore();

const debounceTimers = ref<Record<string, ReturnType<typeof setTimeout>>>({});

const DEFAULT_SOUNDS = ["rain", "cafe", "whitenoise", "nature", "lofi"];

const SOUND_LABELS: Record<string, string> = {
  rain: "Rain",
  cafe: "Cafe",
  whitenoise: "White Noise",
  nature: "Nature",
  lofi: "Lo-Fi",
};

function getLabel(name: string): string {
  return SOUND_LABELS[name] ?? name.charAt(0).toUpperCase() + name.slice(1);
}

function isPlaying(name: string): boolean {
  return sound.audioState.layers.find((l) => l.name === name)?.is_playing ?? false;
}

function getVolume(name: string): number {
  return sound.audioState.layers.find((l) => l.name === name)?.volume ?? 0.5;
}

function toggleSound(name: string) {
  if (isPlaying(name)) {
    sound.stop(name);
  } else {
    sound.play(name, 0.5);
  }
}

function onVolumeChange(name: string, event: Event) {
  const target = event.target as HTMLInputElement;
  const vol = Number(target.value);
  clearTimeout(debounceTimers.value[name]);
  debounceTimers.value[name] = setTimeout(() => {
    sound.setVolume(name, vol);
  }, 100);
}

function onMasterChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const vol = Number(target.value);
  clearTimeout(debounceTimers.value["__master"]);
  debounceTimers.value["__master"] = setTimeout(() => {
    sound.setMaster(vol);
  }, 100);
}
</script>

<template>
  <div class="sound-mixer">
    <!-- Master Volume -->
    <div class="master-row">
      <svg
        class="master-icon"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
        <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
        <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
      </svg>
      <span class="master-label">Master</span>
      <div class="slider-wrap">
        <input
          type="range"
          class="range-slider"
          min="0"
          max="1"
          step="0.01"
          :value="sound.audioState.master_volume"
          @input="onMasterChange"
        />
      </div>
      <span class="volume-value">{{ Math.round(sound.audioState.master_volume * 100) }}%</span>
    </div>

    <div class="divider" />

    <!-- Sound Layers -->
    <div
      v-for="name in DEFAULT_SOUNDS"
      :key="name"
      class="sound-row"
      :class="{ active: isPlaying(name) }"
    >
      <!-- Rain icon -->
      <svg
        v-if="name === 'rain'"
        class="sound-icon"
        :class="{ on: isPlaying(name) }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
      >
        <path
          d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        />
        <path
          d="M12 6v4l-2 6"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
        <circle cx="10" cy="16" r="1.5" fill="currentColor" />
      </svg>
      <!-- Cafe icon -->
      <svg
        v-else-if="name === 'cafe'"
        class="sound-icon"
        :class="{ on: isPlaying(name) }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
      >
        <path
          d="M5 9v5c0 2.21 3.13 4 7 4s7-1.79 7-4V9"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        />
        <path
          d="M19 9h1a2 2 0 0 1 0 4h-1"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
        <path
          d="M5 9h14"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
        <path
          d="M7 3v3M10 2v4M13 3v3"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
      </svg>
      <!-- White Noise icon -->
      <svg
        v-else-if="name === 'whitenoise'"
        class="sound-icon"
        :class="{ on: isPlaying(name) }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
      >
        <path
          d="M2 12h2l2-4 2 8 2-6 2 4 2-2 2 6 2-8 2 4h2"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <!-- Nature icon -->
      <svg
        v-else-if="name === 'nature'"
        class="sound-icon"
        :class="{ on: isPlaying(name) }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
      >
        <path
          d="M12 22V8"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
        <path
          d="M5 12s2-5 7-8c5 3 7 8 7 8s-3 2-7 2-7-2-7-2z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linejoin="round"
        />
        <path
          d="M8 18s1.5-2 4-2 4 2 4 2"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
      </svg>
      <!-- Lo-Fi icon -->
      <svg
        v-else-if="name === 'lofi'"
        class="sound-icon"
        :class="{ on: isPlaying(name) }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
      >
        <circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" stroke-width="1.5" />
        <path
          d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
      </svg>
      <!-- Fallback icon -->
      <svg
        v-else
        class="sound-icon"
        :class="{ on: isPlaying(name) }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
      >
        <circle cx="12" cy="12" r="8" fill="none" stroke="currentColor" stroke-width="1.5" />
      </svg>

      <span class="sound-label">{{ getLabel(name) }}</span>
      <button class="toggle-dot" :class="{ on: isPlaying(name) }" @click="toggleSound(name)" />
      <div class="slider-wrap">
        <input
          type="range"
          class="range-slider"
          :class="{ disabled: !isPlaying(name) }"
          min="0"
          max="1"
          step="0.01"
          :value="getVolume(name)"
          :disabled="!isPlaying(name)"
          @input="onVolumeChange(name, $event)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.sound-mixer {
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.master-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.master-icon {
  color: var(--text-secondary);
  flex-shrink: 0;
}

.master-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  min-width: 48px;
}

.divider {
  height: 1px;
  background: var(--border-subtle);
}

.sound-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.sound-icon {
  flex-shrink: 0;
  color: var(--text-muted);
  transition: color 0.15s;
}

.sound-icon.on {
  color: var(--color-work);
}

.sound-label {
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 76px;
}

.toggle-dot {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid var(--bg-hover);
  background: transparent;
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  transition:
    background 0.15s,
    border-color 0.15s,
    transform 0.1s;
}

.toggle-dot:hover {
  border-color: var(--text-muted);
}

.toggle-dot:active {
  transform: scale(0.85);
}

.toggle-dot.on {
  background: var(--color-work);
  border-color: var(--color-work);
}

.slider-wrap {
  flex: 1;
  display: flex;
  align-items: center;
}

.volume-value {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-muted);
  min-width: 32px;
  text-align: right;
}

/* Custom range slider */
.range-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 4px;
  background: var(--bg-hover);
  border-radius: 2px;
  outline: none;
  transition: opacity 0.15s;
}

.range-slider.disabled {
  opacity: 0.3;
}

.range-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--color-work);
  cursor: pointer;
  border: none;
}

.range-slider:disabled::-webkit-slider-thumb {
  background: var(--text-muted);
  cursor: default;
}

.range-slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--color-work);
  cursor: pointer;
  border: none;
}

.range-slider:disabled::-moz-range-thumb {
  background: var(--text-muted);
  cursor: default;
}

.range-slider::-moz-range-track {
  height: 4px;
  background: var(--bg-hover);
  border-radius: 2px;
  border: none;
}
</style>
