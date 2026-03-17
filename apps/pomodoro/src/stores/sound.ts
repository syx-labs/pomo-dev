import { defineStore } from "pinia";
import { ref, computed } from "vue";
import {
  playSound,
  stopSound,
  setSoundVolume,
  setMasterVolume,
  fadeSounds,
  stopAllSounds,
  getAudioState,
  getAvailableSounds,
  saveSoundPreset,
  listSoundPresets,
  loadSoundPreset,
  deleteSoundPreset,
  type AudioState,
  type SoundPreset,
  type PresetLayer,
} from "@/lib/tauri";
import { useToast } from "@/composables/useToast";

export const useSoundStore = defineStore("sound", () => {
  const audioState = ref<AudioState>({ layers: [], master_volume: 1.0 });
  const presets = ref<SoundPreset[]>([]);
  const availableSounds = ref<string[]>([]);
  const activePresetId = ref<string | null>(null);
  const loading = ref(false);

  const { showToast } = useToast();

  const isAnySoundPlaying = computed(() => audioState.value.layers.some((l) => l.is_playing));

  const activeLayers = computed(() => audioState.value.layers.filter((l) => l.is_playing));

  async function refreshState() {
    try {
      audioState.value = await getAudioState();
    } catch {
      // silent — state refresh is best-effort
    }
  }

  async function init() {
    loading.value = true;
    try {
      const [state, sounds, presetList] = await Promise.all([
        getAudioState(),
        getAvailableSounds(),
        listSoundPresets(),
      ]);
      audioState.value = state;
      availableSounds.value = sounds;
      presets.value = presetList;
    } catch {
      showToast("Failed to load sound state", "warning");
    }
    loading.value = false;
  }

  async function play(name: string, volume = 0.5) {
    try {
      await playSound(name, volume);
      activePresetId.value = null;
      await refreshState();
    } catch {
      showToast("Failed to play sound", "error");
    }
  }

  async function stop(name: string) {
    try {
      await stopSound(name);
      activePresetId.value = null;
      await refreshState();
    } catch {
      showToast("Failed to stop sound", "error");
    }
  }

  async function setVolume(name: string, vol: number) {
    try {
      await setSoundVolume(name, vol);
      // Update local state immediately for responsiveness
      const layer = audioState.value.layers.find((l) => l.name === name);
      if (layer) layer.volume = vol;
    } catch {
      // silent — debounced volume changes can fail transiently
    }
  }

  async function setMaster(vol: number) {
    try {
      await setMasterVolume(vol);
      audioState.value.master_volume = vol;
    } catch {
      showToast("Failed to set master volume", "error");
    }
  }

  async function fadeIn() {
    try {
      await fadeSounds("in");
    } catch {
      // silent
    }
  }

  async function fadeOut() {
    try {
      await fadeSounds("out");
    } catch {
      // silent
    }
  }

  async function stopAll() {
    try {
      await stopAllSounds();
      activePresetId.value = null;
      await refreshState();
    } catch {
      showToast("Failed to stop sounds", "error");
    }
  }

  async function savePreset(name: string) {
    try {
      const layers: PresetLayer[] = activeLayers.value.map((l) => ({
        sound: l.name,
        volume: l.volume,
      }));
      await saveSoundPreset(name, JSON.stringify(layers));
      presets.value = await listSoundPresets();
      showToast(`Preset "${name}" saved`, "success");
    } catch {
      showToast("Failed to save preset", "error");
    }
  }

  async function loadPresetById(id: string) {
    try {
      await loadSoundPreset(id);
      activePresetId.value = id;
      await refreshState();
    } catch {
      showToast("Failed to load preset", "error");
    }
  }

  async function deletePresetById(id: string) {
    try {
      await deleteSoundPreset(id);
      if (activePresetId.value === id) activePresetId.value = null;
      presets.value = await listSoundPresets();
    } catch {
      showToast("Failed to delete preset", "error");
    }
  }

  return {
    audioState,
    presets,
    availableSounds,
    activePresetId,
    loading,
    isAnySoundPlaying,
    activeLayers,
    init,
    play,
    stop,
    setVolume,
    setMaster,
    fadeIn,
    fadeOut,
    stopAll,
    savePreset,
    loadPresetById,
    deletePresetById,
  };
});
