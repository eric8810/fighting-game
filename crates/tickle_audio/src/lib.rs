use std::collections::HashMap;
use std::path::{Path, PathBuf};

use kira::manager::backend::DefaultBackend;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::tween::Tween;
use kira::Volume;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("failed to initialize audio manager: {0}")]
    InitError(String),
    #[error("failed to load sound '{path}': {source}")]
    LoadError { path: String, source: anyhow::Error },
    #[error("failed to play sound '{0}': {1}")]
    PlayError(String, String),
    #[error("sound not found: '{0}'")]
    NotFound(String),
}

pub type AudioResult<T> = Result<T, AudioError>;

/// Volume settings for the audio system.
#[derive(Clone, Debug)]
pub struct VolumeSettings {
    pub master: f64,
    pub sfx: f64,
    pub music: f64,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self {
            master: 1.0,
            sfx: 1.0,
            music: 1.0,
        }
    }
}

/// Core audio system wrapping kira's AudioManager.
pub struct AudioSystem {
    manager: AudioManager,
    sound_cache: HashMap<String, StaticSoundData>,
    current_music: Option<StaticSoundHandle>,
    current_music_id: Option<String>,
    volume: VolumeSettings,
    assets_root: PathBuf,
}

impl AudioSystem {
    /// Create a new AudioSystem. `assets_root` is the base directory for audio files
    /// (e.g. `./assets`). Sound files are loaded relative to this root.
    pub fn new(assets_root: impl Into<PathBuf>) -> AudioResult<Self> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|e| AudioError::InitError(e.to_string()))?;

        log::info!("Audio system initialized");

        Ok(Self {
            manager,
            sound_cache: HashMap::new(),
            current_music: None,
            current_music_id: None,
            volume: VolumeSettings::default(),
            assets_root: assets_root.into(),
        })
    }

    /// Load a sound file into the cache. `id` is the lookup key, `relative_path`
    /// is relative to the assets root (e.g. "sounds/hit_light.ogg").
    pub fn load_sound(&mut self, id: &str, relative_path: &str) -> AudioResult<()> {
        let full_path = self.assets_root.join(relative_path);
        let sound_data =
            StaticSoundData::from_file(&full_path).map_err(|e| AudioError::LoadError {
                path: full_path.display().to_string(),
                source: e.into(),
            })?;
        self.sound_cache.insert(id.to_string(), sound_data);
        log::debug!("Loaded sound '{}' from {}", id, relative_path);
        Ok(())
    }

    /// Play a cached sound effect. Returns a handle to control the playing sound.
    pub fn play_sound(&mut self, id: &str) -> AudioResult<StaticSoundHandle> {
        let sound_data = self
            .sound_cache
            .get(id)
            .ok_or_else(|| AudioError::NotFound(id.to_string()))?;

        let effective_volume = self.volume.master * self.volume.sfx;
        let settings = StaticSoundSettings::new().volume(Volume::Amplitude(effective_volume));
        let data = sound_data.clone().with_settings(settings);

        let handle = self
            .manager
            .play(data)
            .map_err(|e| AudioError::PlayError(id.to_string(), e.to_string()))?;

        Ok(handle)
    }

    /// Play music from a cached sound. Stops any currently playing music first.
    /// If `looping` is true, the music will loop indefinitely.
    pub fn play_music(&mut self, id: &str, looping: bool) -> AudioResult<()> {
        // Stop current music if playing
        self.stop_music();

        let sound_data = self
            .sound_cache
            .get(id)
            .ok_or_else(|| AudioError::NotFound(id.to_string()))?;

        let effective_volume = self.volume.master * self.volume.music;
        let mut settings = StaticSoundSettings::new().volume(Volume::Amplitude(effective_volume));
        if looping {
            settings = settings.loop_region(..);
        }
        let data = sound_data.clone().with_settings(settings);

        let handle = self
            .manager
            .play(data)
            .map_err(|e| AudioError::PlayError(id.to_string(), e.to_string()))?;

        self.current_music = Some(handle);
        self.current_music_id = Some(id.to_string());
        log::info!("Playing music '{}' (looping={})", id, looping);
        Ok(())
    }

    /// Stop the currently playing music track.
    pub fn stop_music(&mut self) {
        if let Some(ref mut handle) = self.current_music {
            handle.stop(Tween::default());
            log::info!("Stopped music");
        }
        self.current_music = None;
        self.current_music_id = None;
    }

    /// Set master volume (0.0 to 1.0).
    pub fn set_master_volume(&mut self, volume: f64) {
        self.volume.master = volume.clamp(0.0, 1.0);
        self.update_music_volume();
    }

    /// Set sound effects volume (0.0 to 1.0).
    pub fn set_sfx_volume(&mut self, volume: f64) {
        self.volume.sfx = volume.clamp(0.0, 1.0);
    }

    /// Set music volume (0.0 to 1.0).
    pub fn set_music_volume(&mut self, volume: f64) {
        self.volume.music = volume.clamp(0.0, 1.0);
        self.update_music_volume();
    }

    /// Get current volume settings.
    pub fn volume(&self) -> &VolumeSettings {
        &self.volume
    }

    /// Returns true if a sound with the given id is loaded in the cache.
    pub fn is_loaded(&self, id: &str) -> bool {
        self.sound_cache.contains_key(id)
    }

    /// Returns the id of the currently playing music, if any.
    pub fn current_music_id(&self) -> Option<&str> {
        self.current_music_id.as_deref()
    }

    fn update_music_volume(&mut self) {
        if let Some(ref mut handle) = self.current_music {
            let effective = self.volume.master * self.volume.music;
            handle.set_volume(Volume::Amplitude(effective), Tween::default());
        }
    }
}

// ---------------------------------------------------------------------------
// Audio events -- these are game-level events that trigger sounds
// ---------------------------------------------------------------------------

/// Strength of a hit, used to select the appropriate sound effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitStrength {
    Light,
    Medium,
    Heavy,
}

impl HitStrength {
    /// Determine hit strength from damage value.
    pub fn from_damage(damage: i32) -> Self {
        if damage >= 1500 {
            HitStrength::Heavy
        } else if damage >= 800 {
            HitStrength::Medium
        } else {
            HitStrength::Light
        }
    }

    /// Returns the sound id for this hit strength.
    pub fn sound_id(&self) -> &'static str {
        match self {
            HitStrength::Light => "hit_light",
            HitStrength::Medium => "hit_medium",
            HitStrength::Heavy => "hit_heavy",
        }
    }
}

/// Audio event that the game loop can dispatch to the audio system.
#[derive(Clone, Debug)]
pub enum AudioEvent {
    PlayHitSound { strength: HitStrength },
    PlayActionSound { id: String },
    PlayMusic { id: String, looping: bool },
    StopMusic,
}

/// Process a batch of audio events. Errors are logged but do not halt the game.
pub fn process_audio_events(audio: &mut AudioSystem, events: &[AudioEvent]) {
    for event in events {
        let result = match event {
            AudioEvent::PlayHitSound { strength } => {
                audio.play_sound(strength.sound_id()).map(|_| ())
            }
            AudioEvent::PlayActionSound { id } => audio.play_sound(id).map(|_| ()),
            AudioEvent::PlayMusic { id, looping } => audio.play_music(id, *looping),
            AudioEvent::StopMusic => {
                audio.stop_music();
                Ok(())
            }
        };
        if let Err(e) = result {
            log::warn!("Audio event error: {}", e);
        }
    }
}

/// Load the standard set of fighting game sound effects.
/// Call this during initialization after creating the AudioSystem.
/// Tries .ogg first, falls back to .wav.
pub fn load_default_sounds(audio: &mut AudioSystem) -> AudioResult<()> {
    let sounds = [
        ("hit_light", "sounds/hit_light"),
        ("hit_medium", "sounds/hit_medium"),
        ("hit_heavy", "sounds/hit_heavy"),
    ];
    for (id, base_path) in &sounds {
        let ogg = format!("{}.ogg", base_path);
        let wav = format!("{}.wav", base_path);
        if Path::new(&audio.assets_root.join(&ogg)).exists() {
            audio.load_sound(id, &ogg)?;
        } else if Path::new(&audio.assets_root.join(&wav)).exists() {
            audio.load_sound(id, &wav)?;
        } else {
            log::warn!("Default sound file not found: {} (.ogg or .wav)", base_path);
        }
    }
    Ok(())
}

/// Load the default music tracks.
/// Tries .ogg first, falls back to .wav.
pub fn load_default_music(audio: &mut AudioSystem) -> AudioResult<()> {
    let tracks = [("stage_theme", "music/stage_theme")];
    for (id, base_path) in &tracks {
        let ogg = format!("{}.ogg", base_path);
        let wav = format!("{}.wav", base_path);
        if Path::new(&audio.assets_root.join(&ogg)).exists() {
            audio.load_sound(id, &ogg)?;
        } else if Path::new(&audio.assets_root.join(&wav)).exists() {
            audio.load_sound(id, &wav)?;
        } else {
            log::warn!("Default music file not found: {} (.ogg or .wav)", base_path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_settings_default() {
        let v = VolumeSettings::default();
        assert_eq!(v.master, 1.0);
        assert_eq!(v.sfx, 1.0);
        assert_eq!(v.music, 1.0);
    }

    #[test]
    fn test_hit_strength_from_damage() {
        assert_eq!(HitStrength::from_damage(0), HitStrength::Light);
        assert_eq!(HitStrength::from_damage(500), HitStrength::Light);
        assert_eq!(HitStrength::from_damage(799), HitStrength::Light);
        assert_eq!(HitStrength::from_damage(800), HitStrength::Medium);
        assert_eq!(HitStrength::from_damage(1200), HitStrength::Medium);
        assert_eq!(HitStrength::from_damage(1500), HitStrength::Heavy);
        assert_eq!(HitStrength::from_damage(3000), HitStrength::Heavy);
    }

    #[test]
    fn test_hit_strength_sound_ids() {
        assert_eq!(HitStrength::Light.sound_id(), "hit_light");
        assert_eq!(HitStrength::Medium.sound_id(), "hit_medium");
        assert_eq!(HitStrength::Heavy.sound_id(), "hit_heavy");
    }

    #[test]
    fn test_audio_system_creation() {
        // This test verifies AudioSystem can be created (requires audio backend).
        // On CI without audio, this may fail -- that's expected.
        let result = AudioSystem::new("./test_assets");
        // We just check it doesn't panic; it may error if no audio device is available.
        match result {
            Ok(sys) => {
                assert_eq!(sys.volume().master, 1.0);
                assert!(sys.current_music_id().is_none());
                assert!(!sys.is_loaded("nonexistent"));
            }
            Err(e) => {
                // No audio device available (CI environment) -- acceptable
                log::warn!("Audio init skipped (no device): {}", e);
            }
        }
    }

    #[test]
    fn test_volume_clamping() {
        let result = AudioSystem::new("./test_assets");
        if let Ok(mut sys) = result {
            sys.set_master_volume(1.5);
            assert_eq!(sys.volume().master, 1.0);

            sys.set_master_volume(-0.5);
            assert_eq!(sys.volume().master, 0.0);

            sys.set_sfx_volume(0.5);
            assert_eq!(sys.volume().sfx, 0.5);

            sys.set_music_volume(0.75);
            assert_eq!(sys.volume().music, 0.75);
        }
    }

    #[test]
    fn test_play_sound_not_found() {
        let result = AudioSystem::new("./test_assets");
        if let Ok(mut sys) = result {
            let err = sys.play_sound("nonexistent").unwrap_err();
            assert!(matches!(err, AudioError::NotFound(_)));
        }
    }

    #[test]
    fn test_play_music_not_found() {
        let result = AudioSystem::new("./test_assets");
        if let Ok(mut sys) = result {
            let err = sys.play_music("nonexistent", false).unwrap_err();
            assert!(matches!(err, AudioError::NotFound(_)));
        }
    }

    #[test]
    fn test_stop_music_when_none_playing() {
        let result = AudioSystem::new("./test_assets");
        if let Ok(mut sys) = result {
            // Should not panic
            sys.stop_music();
            assert!(sys.current_music_id().is_none());
        }
    }

    #[test]
    fn test_audio_event_variants() {
        let events = vec![
            AudioEvent::PlayHitSound {
                strength: HitStrength::Light,
            },
            AudioEvent::PlayActionSound {
                id: "block".to_string(),
            },
            AudioEvent::PlayMusic {
                id: "stage_theme".to_string(),
                looping: true,
            },
            AudioEvent::StopMusic,
        ];
        // Verify events can be constructed and cloned
        let _cloned = events.clone();
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn test_process_audio_events_graceful_errors() {
        let result = AudioSystem::new("./test_assets");
        if let Ok(mut sys) = result {
            // Processing events for missing sounds should log warnings, not panic
            let events = vec![
                AudioEvent::PlayHitSound {
                    strength: HitStrength::Heavy,
                },
                AudioEvent::StopMusic,
            ];
            process_audio_events(&mut sys, &events);
        }
    }
}
