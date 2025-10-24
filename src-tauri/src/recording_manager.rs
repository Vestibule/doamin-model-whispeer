use crate::audio_session::{AudioSession, AudioSessionConfig};
use crate::audio_enhancement::{AudioEnhancer, AudioEnhancementConfig};
use crate::speech_to_text::SpeechToText;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone)]
pub enum RecordingState {
    Idle,
    Recording,
    Processing,
}

pub struct RecordingManager {
    state: Arc<Mutex<RecordingState>>,
    session: Arc<Mutex<Option<AudioSession>>>,
    stt: Arc<SpeechToText>,
    app_handle: AppHandle,
    selected_device: Arc<Mutex<Option<String>>>,
    enhancement_config: Arc<Mutex<AudioEnhancementConfig>>,
}

impl RecordingManager {
    pub fn new(model_path: PathBuf, app_handle: AppHandle) -> Self {
        Self {
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            session: Arc::new(Mutex::new(None)),
            stt: Arc::new(SpeechToText::new(model_path)),
            app_handle,
            selected_device: Arc::new(Mutex::new(None)),
            enhancement_config: Arc::new(Mutex::new(AudioEnhancementConfig::default())),
        }
    }

    pub fn start_recording(&self) -> Result<String> {
        let mut state = self.state.lock().unwrap();
        
        if matches!(*state, RecordingState::Recording) {
            anyhow::bail!("Recording already in progress");
        }

        // Use home directory for easier access to audio files during debugging
        let output_dir = std::env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join("domain-model-audio"))
            .unwrap_or_else(|_| std::env::temp_dir().join("domain-model-audio"));
        let device_name = self.selected_device.lock().unwrap().clone();
        
        let config = AudioSessionConfig {
            output_dir: output_dir.clone(),
            device_name,
            ..Default::default()
        };

        let session = AudioSession::new(config)
            .context("Failed to create audio session")?;

        let session_clone = session.clone();
        let state_clone = Arc::clone(&self.state);
        let session_arc = Arc::clone(&self.session);
        let stt_clone = Arc::clone(&self.stt);
        let app_handle = self.app_handle.clone();
        let enhancement_config = self.enhancement_config.lock().unwrap().clone();

        // Store session
        *self.session.lock().unwrap() = Some(session.clone());
        *state = RecordingState::Recording;

        // Emit recording started event
        let _ = self.app_handle.emit("recording-state-changed", "recording");

        // Start recording in a background thread
        thread::spawn(move || {
            log::info!("Starting audio recording thread");
            
            if let Err(e) = session_clone.start_recording() {
                log::error!("Recording error: {}", e);
                let _ = app_handle.emit("recording-error", format!("{}", e));
            }
            
            // When recording stops, process utterances
            log::info!("Recording stopped, processing utterances");
            let mut state_guard = state_clone.lock().unwrap();
            *state_guard = RecordingState::Processing;
            drop(state_guard);

            let _ = app_handle.emit("recording-state-changed", "processing");

            // Get all utterances
            let session_guard = session_arc.lock().unwrap();
            if let Some(sess) = session_guard.as_ref() {
                let utterances = sess.get_utterances();
                log::info!("Found {} utterances to transcribe", utterances.len());
                
                // Transcribe all utterances
                for utterance in utterances {
                    log::info!("Transcribing utterance {}: {:?}", utterance.id, utterance.file_path);
                    
                    // Appliquer l'amélioration audio avant transcription
                    let enhanced_path = utterance.file_path.with_extension("enhanced.wav");
                    
                    // Créer l'enhancer avec le sample rate du fichier (detecté depuis le nom de fichier ou par défaut 48kHz)
                    let sample_rate = 48000; // TODO: détecter depuis le fichier WAV
                    
                    match AudioEnhancer::new(sample_rate, enhancement_config.clone()) {
                        Ok(mut enhancer) => {
                            match enhancer.process_file(&utterance.file_path, &enhanced_path) {
                                Ok(_) => {
                                    log::info!("Audio enhancement applied successfully");
                                    // Transcrire le fichier amélioré
                                    match stt_clone.transcribe_file(&enhanced_path) {
                                        Ok(result) => {
                                            log::info!("Transcription successful: {}", result.text);
                                            let _ = app_handle.emit("transcription-result", &result);
                                        }
                                        Err(e) => {
                                            log::error!("Transcription failed: {}", e);
                                            let _ = app_handle.emit("transcription-error", format!("{}", e));
                                        }
                                    }
                                    // Supprimer le fichier temporaire amélioré
                                    let _ = std::fs::remove_file(&enhanced_path);
                                }
                                Err(e) => {
                                    log::warn!("Audio enhancement failed, using original file: {}", e);
                                    // Fallback: transcrire le fichier original
                                    match stt_clone.transcribe_file(&utterance.file_path) {
                                        Ok(result) => {
                                            log::info!("Transcription successful: {}", result.text);
                                            let _ = app_handle.emit("transcription-result", &result);
                                        }
                                        Err(e) => {
                                            log::error!("Transcription failed: {}", e);
                                            let _ = app_handle.emit("transcription-error", format!("{}", e));
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to create audio enhancer: {}", e);
                            // Fallback: transcrire le fichier original
                            match stt_clone.transcribe_file(&utterance.file_path) {
                                Ok(result) => {
                                    log::info!("Transcription successful: {}", result.text);
                                    let _ = app_handle.emit("transcription-result", &result);
                                }
                                Err(e) => {
                                    log::error!("Transcription failed: {}", e);
                                    let _ = app_handle.emit("transcription-error", format!("{}", e));
                                }
                            }
                        }
                    }
                }
            }

            let mut state_guard = state_clone.lock().unwrap();
            *state_guard = RecordingState::Idle;
            drop(state_guard);

            let _ = app_handle.emit("recording-state-changed", "idle");
        });

        Ok(format!("Recording started. Audio will be saved to: {:?}", output_dir))
    }

    pub fn stop_recording(&self) -> Result<String> {
        let state = self.state.lock().unwrap();
        
        if !matches!(*state, RecordingState::Recording) {
            anyhow::bail!("No recording in progress");
        }

        // Signal the audio session to stop
        if let Some(session) = self.session.lock().unwrap().as_ref() {
            session.stop();
        }
        
        Ok("Recording stopped. Processing utterances...".to_string())
    }

    pub fn get_state(&self) -> RecordingState {
        self.state.lock().unwrap().clone()
    }

    pub fn set_audio_device(&self, device_name: String) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        if !matches!(*state, RecordingState::Idle) {
            anyhow::bail!("Cannot change audio device while recording");
        }
        
        *self.selected_device.lock().unwrap() = Some(device_name.clone());
        log::info!("Audio device set to: {}", device_name);
        
        Ok(())
    }

    pub fn get_selected_device(&self) -> Option<String> {
        self.selected_device.lock().unwrap().clone()
    }
}
