use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{debug, info, warn};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use webrtc_vad::{Vad, VadMode};

/// Wrapper pour rendre Vad thread-safe
/// SAFETY: Vad est toujours utilisé derrière un Mutex, donc l'accès concurrent est contrôlé
struct SendVad(Vad);

unsafe impl Send for SendVad {}

impl SendVad {
    fn is_voice_segment(&mut self, frame: &[i16]) -> Option<bool> {
        self.0.is_voice_segment(frame).ok()
    }
}

/// Configuration pour la session audio
pub struct AudioSessionConfig {
    /// Durée minimale du silence pour considérer la fin d'une utterance (en ms)
    pub silence_duration_ms: u32,
    /// Durée minimale d'une utterance valide (en ms)
    pub min_utterance_duration_ms: u32,
    /// Répertoire où sauvegarder les fichiers WAV temporaires
    pub output_dir: PathBuf,
    /// Mode VAD (Quality, LowBitrate, Aggressive, VeryAggressive)
    pub vad_mode: VadMode,
    /// Nom optionnel de l'interface audio à utiliser
    pub device_name: Option<String>,
}

impl Clone for AudioSessionConfig {
    fn clone(&self) -> Self {
        Self {
            silence_duration_ms: self.silence_duration_ms,
            min_utterance_duration_ms: self.min_utterance_duration_ms,
            output_dir: self.output_dir.clone(),
            vad_mode: match self.vad_mode {
                VadMode::Quality => VadMode::Quality,
                VadMode::LowBitrate => VadMode::LowBitrate,
                VadMode::Aggressive => VadMode::Aggressive,
                VadMode::VeryAggressive => VadMode::VeryAggressive,
            },
            device_name: self.device_name.clone(),
        }
    }
}

impl std::fmt::Debug for AudioSessionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vad_mode_repr = match self.vad_mode {
            VadMode::Quality => "Quality",
            VadMode::LowBitrate => "LowBitrate",
            VadMode::Aggressive => "Aggressive",
            VadMode::VeryAggressive => "VeryAggressive",
        };
        f.debug_struct("AudioSessionConfig")
            .field("silence_duration_ms", &self.silence_duration_ms)
            .field("min_utterance_duration_ms", &self.min_utterance_duration_ms)
            .field("output_dir", &self.output_dir)
            .field("vad_mode", &vad_mode_repr)
            .field("device_name", &self.device_name)
            .finish()
    }
}

impl Default for AudioSessionConfig {
    fn default() -> Self {
        Self {
            silence_duration_ms: 1000,
            min_utterance_duration_ms: 300,
            output_dir: std::env::temp_dir(),
            vad_mode: VadMode::Aggressive,
            device_name: None,
        }
    }
}

/// Représente un segment d'utterance enregistré
#[derive(Debug, Clone)]
pub struct Utterance {
    pub id: usize,
    pub file_path: PathBuf,
    pub duration_ms: u32,
    pub sample_count: usize,
}

/// Gestionnaire de session audio avec détection d'utterances
#[derive(Clone)]
pub struct AudioSession {
    config: AudioSessionConfig,
    vad: Arc<Mutex<SendVad>>,
    utterances: Arc<Mutex<Vec<Utterance>>>,
    current_buffer: Arc<Mutex<Vec<i16>>>,
    silence_frames: Arc<Mutex<u32>>,
    utterance_counter: Arc<Mutex<usize>>,
    is_speaking: Arc<Mutex<bool>>,
    stop_flag: Arc<AtomicBool>,
}

impl AudioSession {
    /// Crée une nouvelle session audio
    pub fn new(config: AudioSessionConfig) -> Result<Self> {
        info!("Creating audio session with config: {:?}", config);
        let vad = Vad::new_with_rate_and_mode(
            webrtc_vad::SampleRate::Rate16kHz,
            match config.vad_mode {
                VadMode::Quality => VadMode::Quality,
                VadMode::LowBitrate => VadMode::LowBitrate,
                VadMode::Aggressive => VadMode::Aggressive,
                VadMode::VeryAggressive => VadMode::VeryAggressive,
            },
        );

        // Créer le répertoire de sortie s'il n'existe pas
        std::fs::create_dir_all(&config.output_dir)
            .context("Failed to create output directory")?;

        Ok(Self {
            config,
            vad: Arc::new(Mutex::new(SendVad(vad))),
            utterances: Arc::new(Mutex::new(Vec::new())),
            current_buffer: Arc::new(Mutex::new(Vec::new())),
            silence_frames: Arc::new(Mutex::new(0)),
            utterance_counter: Arc::new(Mutex::new(0)),
            is_speaking: Arc::new(Mutex::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Démarre la capture audio et la détection d'utterances
    pub fn start_recording(&self) -> Result<()> {
        let host = cpal::default_host();
        
        // Select device based on config
        let device = if let Some(ref device_name) = self.config.device_name {
            info!("Looking for audio device: {}", device_name);
            host.input_devices()
                .context("Failed to enumerate input devices")?
                .find(|d| d.name().map(|n| n == *device_name).unwrap_or(false))
                .ok_or_else(|| anyhow::anyhow!("Audio device '{}' not found", device_name))?
        } else {
            host.default_input_device()
                .context("No input device available")?
        };

        let config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        info!("Audio input device: {}", device.name()?);
        info!("Sample rate: {} Hz", config.sample_rate().0);
        info!("Channels: {}", config.channels());

        // Clone des Arc pour le stream
        let vad = Arc::clone(&self.vad);
        let current_buffer = Arc::clone(&self.current_buffer);
        let silence_frames = Arc::clone(&self.silence_frames);
        let utterance_counter = Arc::clone(&self.utterance_counter);
        let is_speaking = Arc::clone(&self.is_speaking);
        let utterances = Arc::clone(&self.utterances);
        let session_config = self.config.clone();

        // Buffer pour le VAD (480 samples = 30ms à 16kHz)
        let vad_frame_size = 480;
        let vad_buffer = Arc::new(Mutex::new(Vec::new()));

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Convertir f32 à i16
                let samples: Vec<i16> = data
                    .iter()
                    .map(|&sample| (sample * 32767.0) as i16)
                    .collect();

                let mut vad_buf = vad_buffer.lock().unwrap();
                vad_buf.extend_from_slice(&samples);

                // Traiter les frames du VAD
                while vad_buf.len() >= vad_frame_size {
                    let frame: Vec<i16> = vad_buf.drain(..vad_frame_size).collect();
                    
                    // Détection de voix
                    let is_voice = vad.lock().unwrap().is_voice_segment(&frame).unwrap_or(false);

                    let mut buffer = current_buffer.lock().unwrap();
                    let mut silence = silence_frames.lock().unwrap();
                    let mut speaking = is_speaking.lock().unwrap();

                    if is_voice {
                        // Voix détectée
                        if !*speaking {
                            debug!("Voice activity started");
                        }
                        *silence = 0;
                        *speaking = true;
                        buffer.extend_from_slice(&frame);
                    } else if *speaking {
                        // Silence pendant qu'on parle
                        *silence += 30; // 30ms par frame
                        buffer.extend_from_slice(&frame);

                        // Vérifier si le silence est assez long pour terminer l'utterance
                        if *silence >= session_config.silence_duration_ms {
                            let duration_ms = (buffer.len() as u32 * 1000) / 16000;
                            
                            // Sauvegarder l'utterance si elle est assez longue
                            if duration_ms >= session_config.min_utterance_duration_ms {
                                let mut counter = utterance_counter.lock().unwrap();
                                *counter += 1;
                                let utterance_id = *counter;

                                let file_path = session_config.output_dir.join(
                                    format!("utterance_{:04}.wav", utterance_id)
                                );

                                // Sauvegarder en WAV
                                if let Err(e) = save_wav(&file_path, &buffer, 16000) {
                                    warn!("Failed to save utterance: {}", e);
                                } else {
                                    info!("Saved utterance {} to {:?} ({}ms)", 
                                             utterance_id, file_path, duration_ms);
                                    
                                    let utterance = Utterance {
                                        id: utterance_id,
                                        file_path,
                                        duration_ms,
                                        sample_count: buffer.len(),
                                    };
                                    
                                    utterances.lock().unwrap().push(utterance);
                                }
                            }

                            // Réinitialiser pour la prochaine utterance
                            buffer.clear();
                            *silence = 0;
                            *speaking = false;
                        }
                    }
                }
            },
            move |err| {
                eprintln!("Stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;

        info!("Recording started. Waiting for stop signal...");
        info!("Utterances will be saved to: {:?}", self.config.output_dir);

        // Garder le stream actif jusqu'au signal d'arrêt
        while !self.stop_flag.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        info!("Stop signal received, ending recording");
        drop(stream);

        Ok(())
    }

    /// Arrête l'enregistrement en cours
    pub fn stop(&self) {
        info!("Stopping recording...");
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    /// Récupère toutes les utterances enregistrées
    pub fn get_utterances(&self) -> Vec<Utterance> {
        self.utterances.lock().unwrap().clone()
    }
}

/// Sauvegarde des samples audio au format WAV
fn save_wav(path: &Path, samples: &[i16], sample_rate: u32) -> Result<()> {
    let mut file = File::create(path).context("Failed to create WAV file")?;

    let num_samples = samples.len() as u32;
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = num_samples * num_channels as u32 * bits_per_sample as u32 / 8;

    // Header RIFF
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_size).to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // Chunk fmt
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // Taille du chunk fmt
    file.write_all(&1u16.to_le_bytes())?; // Format PCM
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    // Chunk data
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;

    // Données audio
    for &sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }

    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AudioSessionConfig::default();
        assert_eq!(config.silence_duration_ms, 1000);
        assert_eq!(config.min_utterance_duration_ms, 300);
    }

    #[test]
    fn test_wav_creation() {
        // Créer quelques samples de test
        let samples: Vec<i16> = (0..16000)
            .map(|i| ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 16000.0).sin() * 32767.0) as i16)
            .collect();

        let temp_path = std::env::temp_dir().join("test_audio.wav");
        let result = save_wav(&temp_path, &samples, 16000);
        
        assert!(result.is_ok());
        assert!(temp_path.exists());
        
        // Nettoyer
        std::fs::remove_file(temp_path).ok();
    }
}
