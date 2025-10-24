use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_ms: u64,
}

pub struct SpeechToText {
    context: Arc<Mutex<Option<WhisperContext>>>,
    model_path: PathBuf,
}

impl SpeechToText {
    /// Create a new SpeechToText instance
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            context: Arc::new(Mutex::new(None)),
            model_path,
        }
    }

    /// Initialize the Whisper model (lazy loading)
    fn ensure_model_loaded(&self) -> Result<()> {
        let mut context = self.context.lock().unwrap();
        
        if context.is_none() {
            log::info!("Loading Whisper model from {:?}", self.model_path);
            let ctx = WhisperContext::new_with_params(
                &self.model_path.to_string_lossy(),
                WhisperContextParameters::default(),
            )
            .context("Failed to load Whisper model")?;
            *context = Some(ctx);
            log::info!("Whisper model loaded successfully");
        }
        
        Ok(())
    }

    /// Transcribe audio from a WAV file
    pub fn transcribe_file(&self, audio_path: &PathBuf) -> Result<TranscriptionResult> {
        self.ensure_model_loaded()?;
        
        log::info!("Transcribing audio file: {}", audio_path.display());
        let start = std::time::Instant::now();
        
        // Read and convert audio
        let audio_data = self.read_wav_file(audio_path)?;
        log::info!("Audio loaded: {} samples", audio_data.len());
        
        let context = self.context.lock().unwrap();
        let ctx = context.as_ref().unwrap();
        
        // Create transcription parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_language(Some("en"));
        params.set_translate(false);
        
        // Create a new state for this transcription
        let mut state = ctx.create_state().context("Failed to create Whisper state")?;
        
        log::info!("Running Whisper inference...");
        // Run the transcription
        state
            .full(params, &audio_data)
            .context("Failed to run Whisper transcription")?;
        
        // Extract the transcription text
        let num_segments = state.full_n_segments();
        log::info!("Transcription complete: {} segments", num_segments);
        
        let mut full_text = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i as i32) {
                // Use safe API to extract text
                match segment.to_str() {
                    Ok(text) => {
                        log::debug!("Segment {}: '{}'", i, text);
                        full_text.push_str(text);
                        full_text.push(' ');
                    }
                    Err(e) => {
                        log::error!("Failed to extract text from segment {}: {:?}", i, e);
                    }
                }
            }
        }
        
        let duration_ms = start.elapsed().as_millis() as u64;
        log::info!("Transcription result: '{}' (took {}ms)", full_text.trim(), duration_ms);
        
        Ok(TranscriptionResult {
            text: full_text.trim().to_string(),
            language: Some("en".to_string()),
            duration_ms,
        })
    }

    /// Read a WAV file and convert it to f32 samples at 16kHz mono
    fn read_wav_file(&self, path: &PathBuf) -> Result<Vec<f32>> {
        use std::fs::File;
        use std::io::Read;
        
        let mut file = File::open(path).context("Failed to open audio file")?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).context("Failed to read audio file")?;
        
        // Parse WAV header (simplified - assumes 16-bit PCM mono 16kHz)
        if buffer.len() < 44 {
            anyhow::bail!("Invalid WAV file: too short");
        }
        
        // Skip WAV header (44 bytes)
        let audio_data = &buffer[44..];
        
        // Convert i16 samples to f32 normalized to [-1.0, 1.0]
        let samples: Vec<f32> = audio_data
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect();
        
        Ok(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires a Whisper model file
    fn test_speech_to_text_creation() {
        let model_path = PathBuf::from("models/ggml-base.en.bin");
        let stt = SpeechToText::new(model_path);
        assert!(stt.context.lock().unwrap().is_none());
    }
}
