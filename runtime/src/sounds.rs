//! Sound playback for the duck's speaker.
//!
//! Replaces `sounds.py`. Uses the `rodio` crate for cross-platform audio.

use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, Sink};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// Audio playback manager that loads .wav files from a directory.
pub struct Sounds {
    _stream: OutputStream,
    sound_files: HashMap<String, PathBuf>,
    volume: f32,
}

impl Sounds {
    /// Initialize audio output and scan a directory for .wav files.
    pub fn new(volume: f32, sound_directory: &Path) -> Result<Self> {
        let stream = rodio::OutputStreamBuilder::open_default_stream()
            .context("Failed to initialize audio output")?;

        let mut sound_files = HashMap::new();

        if sound_directory.exists() {
            for entry in std::fs::read_dir(sound_directory)
                .context("Failed to read sound directory")?
            {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("wav") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        tracing::info!("Loaded sound: {}", name);
                        sound_files.insert(name.to_string(), path);
                    }
                }
            }
        } else {
            tracing::warn!(
                "Sound directory not found: {}",
                sound_directory.display()
            );
        }

        if sound_files.is_empty() {
            tracing::warn!("No .wav sound files found");
        }

        Ok(Self {
            _stream: stream,
            sound_files,
            volume,
        })
    }

    /// Play a specific sound by filename.
    pub fn play(&self, name: &str) -> Result<()> {
        if let Some(path) = self.sound_files.get(name) {
            self.play_file(path)?;
            tracing::info!("Playing: {}", name);
        } else {
            tracing::warn!("Sound '{}' not found", name);
        }
        Ok(())
    }

    /// Play a random sound from the loaded set.
    pub fn play_random(&self) -> Result<()> {
        if self.sound_files.is_empty() {
            tracing::warn!("No sounds available to play");
            return Ok(());
        }

        let keys: Vec<&String> = self.sound_files.keys().collect();
        let idx = rand::random::<usize>() % keys.len();
        let name = keys[idx].clone();

        self.play(&name)
    }

    fn play_file(&self, path: &Path) -> Result<()> {
        let file = BufReader::new(
            File::open(path).with_context(|| format!("Failed to open {}", path.display()))?,
        );
        let source =
            Decoder::new(file).with_context(|| format!("Failed to decode {}", path.display()))?;

        let sink = Sink::connect_new(&self._stream.mixer());
        sink.set_volume(self.volume);
        sink.append(source);
        sink.detach(); // Play in background without blocking

        Ok(())
    }
}
