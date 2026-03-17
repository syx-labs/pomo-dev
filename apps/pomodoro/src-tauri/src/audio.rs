use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct AudioEngine {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    layers: HashMap<String, SoundLayer>,
    sounds_dir: PathBuf,
    master_volume: f32,
}

// SAFETY: AudioEngine is always accessed behind a Mutex, so only one thread
// accesses it at a time. The OutputStream/Sink types are not Send/Sync due to
// platform audio handles, but single-threaded access through Mutex is safe.
unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}

struct SoundLayer {
    sink: Sink,
    volume: f32,
    is_playing: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AudioState {
    pub layers: Vec<LayerState>,
    pub master_volume: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LayerState {
    pub name: String,
    pub volume: f32,
    pub is_playing: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SoundPreset {
    pub id: String,
    pub name: String,
    pub layers: String,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PresetLayer {
    pub sound: String,
    pub volume: f32,
}

impl AudioEngine {
    pub fn new(sounds_dir: PathBuf) -> Result<Self, String> {
        let (stream, stream_handle) =
            OutputStream::try_default().map_err(|e| format!("Failed to open audio output: {e}"))?;

        Ok(Self {
            _stream: Some(stream),
            stream_handle: Some(stream_handle),
            layers: HashMap::new(),
            sounds_dir,
            master_volume: 0.8,
        })
    }

    /// Create a fallback engine that accepts all calls but produces no audio.
    pub fn silent() -> Self {
        Self {
            _stream: None,
            stream_handle: None,
            layers: HashMap::new(),
            sounds_dir: PathBuf::new(),
            master_volume: 0.0,
        }
    }

    pub fn play_layer(&mut self, name: &str, volume: f32) -> Result<(), String> {
        let stream_handle = match &self.stream_handle {
            Some(h) => h,
            None => return Ok(()), // silent mode
        };

        // If layer already exists and is playing, just update volume
        if let Some(layer) = self.layers.get_mut(name) {
            if !layer.is_playing {
                layer.sink.play();
                layer.is_playing = true;
            }
            layer.volume = volume;
            layer.sink.set_volume(volume * self.master_volume);
            return Ok(());
        }

        // Load the sound file
        let path = self.sounds_dir.join(format!("{name}.ogg"));
        let file = File::open(&path)
            .map_err(|e| format!("Sound file not found: {}: {e}", path.display()))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader)
            .map_err(|e| format!("Failed to decode {name}.ogg: {e}"))?;

        let sink = Sink::try_new(stream_handle)
            .map_err(|e| format!("Failed to create audio sink: {e}"))?;

        sink.append(source.repeat_infinite());
        sink.set_volume(volume * self.master_volume);

        self.layers.insert(
            name.to_string(),
            SoundLayer {
                sink,
                volume,
                is_playing: true,
            },
        );

        Ok(())
    }

    pub fn stop_layer(&mut self, name: &str) {
        if let Some(layer) = self.layers.remove(name) {
            layer.sink.stop();
        }
    }

    pub fn set_layer_volume(&mut self, name: &str, volume: f32) {
        if let Some(layer) = self.layers.get_mut(name) {
            layer.volume = volume;
            layer.sink.set_volume(volume * self.master_volume);
        }
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume;
        for layer in self.layers.values() {
            layer.sink.set_volume(layer.volume * self.master_volume);
        }
    }

    pub fn stop_all(&mut self) {
        for (_, layer) in self.layers.drain() {
            layer.sink.stop();
        }
    }

    pub fn fade_out(&mut self) {
        for layer in self.layers.values_mut() {
            layer.sink.set_volume(0.0);
            layer.sink.pause();
            layer.is_playing = false;
        }
    }

    pub fn fade_in(&mut self) {
        for layer in self.layers.values_mut() {
            layer.sink.play();
            layer.sink.set_volume(layer.volume * self.master_volume);
            layer.is_playing = true;
        }
    }

    pub fn get_state(&self) -> AudioState {
        let layers = self
            .layers
            .iter()
            .map(|(name, layer)| LayerState {
                name: name.clone(),
                volume: layer.volume,
                is_playing: layer.is_playing,
            })
            .collect();

        AudioState {
            layers,
            master_volume: self.master_volume,
        }
    }

    pub fn available_sounds(&self) -> Vec<String> {
        let mut sounds = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.sounds_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("ogg") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        sounds.push(stem.to_string());
                    }
                }
            }
        }
        sounds.sort();
        sounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Silent engine ───────────────────────────────────────────────────

    #[test]
    fn silent_engine_has_no_stream() {
        let engine = AudioEngine::silent();
        assert!(engine._stream.is_none());
        assert!(engine.stream_handle.is_none());
        assert_eq!(engine.master_volume, 0.0);
    }

    #[test]
    fn silent_engine_play_layer_is_noop() {
        let mut engine = AudioEngine::silent();
        // Should succeed silently (no audio output)
        let result = engine.play_layer("rain", 0.5);
        assert!(result.is_ok());
        // No layer should be created since there's no stream_handle
        assert!(engine.layers.is_empty());
    }

    #[test]
    fn silent_engine_stop_all_is_noop() {
        let mut engine = AudioEngine::silent();
        engine.stop_all(); // Should not panic
    }

    #[test]
    fn silent_engine_get_state() {
        let engine = AudioEngine::silent();
        let state = engine.get_state();
        assert!(state.layers.is_empty());
        assert_eq!(state.master_volume, 0.0);
    }

    // ── Master volume ───────────────────────────────────────────────────

    #[test]
    fn set_master_volume_updates_value() {
        let mut engine = AudioEngine::silent();
        engine.set_master_volume(0.5);
        assert_eq!(engine.master_volume, 0.5);
        assert_eq!(engine.get_state().master_volume, 0.5);
    }

    // ── stop_layer on nonexistent layer ─────────────────────────────────

    #[test]
    fn stop_nonexistent_layer_is_noop() {
        let mut engine = AudioEngine::silent();
        engine.stop_layer("nonexistent"); // Should not panic
    }

    // ── set_layer_volume on nonexistent layer ───────────────────────────

    #[test]
    fn set_volume_nonexistent_layer_is_noop() {
        let mut engine = AudioEngine::silent();
        engine.set_layer_volume("nonexistent", 0.5); // Should not panic
    }

    // ── fade_out / fade_in on empty engine ──────────────────────────────

    #[test]
    fn fade_out_empty_engine() {
        let mut engine = AudioEngine::silent();
        engine.fade_out(); // Should not panic
    }

    #[test]
    fn fade_in_empty_engine() {
        let mut engine = AudioEngine::silent();
        engine.fade_in(); // Should not panic
    }

    // ── available_sounds with nonexistent dir ───────────────────────────

    #[test]
    fn available_sounds_nonexistent_dir() {
        let engine = AudioEngine::silent();
        let sounds = engine.available_sounds();
        assert!(sounds.is_empty());
    }

    // ── available_sounds with temp dir ──────────────────────────────────

    #[test]
    fn available_sounds_finds_ogg_files() {
        let dir = std::env::temp_dir().join("pomodoro_test_sounds");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("rain.ogg"), b"fake").unwrap();
        std::fs::write(dir.join("cafe.ogg"), b"fake").unwrap();
        std::fs::write(dir.join("notes.txt"), b"not a sound").unwrap();

        let mut engine = AudioEngine::silent();
        engine.sounds_dir = dir.clone();

        let sounds = engine.available_sounds();
        assert_eq!(sounds, vec!["cafe", "rain"]); // sorted

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── SoundPreset / PresetLayer serialization ─────────────────────────

    #[test]
    fn preset_layer_round_trip() {
        let layers = vec![
            PresetLayer { sound: "rain".to_string(), volume: 0.5 },
            PresetLayer { sound: "cafe".to_string(), volume: 0.3 },
        ];
        let json = serde_json::to_string(&layers).unwrap();
        let parsed: Vec<PresetLayer> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].sound, "rain");
        assert_eq!(parsed[1].volume, 0.3);
    }

    #[test]
    fn audio_state_serialization() {
        let state = AudioState {
            layers: vec![LayerState {
                name: "rain".to_string(),
                volume: 0.5,
                is_playing: true,
            }],
            master_volume: 0.8,
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: AudioState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.layers.len(), 1);
        assert_eq!(parsed.master_volume, 0.8);
    }
}
