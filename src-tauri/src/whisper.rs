use anyhow::{Context, Result};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub fn transcribe_audio(model_path: &Path, audio_path: &Path) -> Result<String> {
    // Load the Whisper model
    let ctx = WhisperContext::new_with_params(
        model_path.to_str().context("Invalid model path")?,
        WhisperContextParameters::default(),
    )
    .context("Failed to load Whisper model")?;

    // Read the audio file
    let audio_data = load_audio(audio_path).context("Failed to load audio file")?;

    // Configure transcription parameters
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(4);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_language(Some("fr")); // French by default, can be made configurable

    // Create a state for transcription
    let mut state = ctx.create_state().context("Failed to create whisper state")?;

    // Run the transcription
    state
        .full(params, &audio_data)
        .context("Failed to run transcription")?;

    // Get the number of segments
    let num_segments = state.full_n_segments();

    // Collect all transcribed text
    let mut full_text = String::new();
    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            let text = segment.to_str_lossy().context("Failed to get segment text")?;
            full_text.push_str(&text);
            full_text.push(' ');
        }
    }

    Ok(full_text.trim().to_string())
}

/// Load audio file and convert to the format expected by Whisper
/// Whisper expects 16kHz mono f32 samples
fn load_audio(path: &Path) -> Result<Vec<f32>> {
    // For now, we'll use a simple approach assuming the input is already in the right format
    // In production, you'd want to use a library like `symphonia` or `hound` to decode various formats
    
    // This is a placeholder - you'll need to implement proper audio loading
    // based on your audio format (WAV, MP3, etc.)
    
    // For WAV files specifically:
    use std::fs::File;
    use std::io::Read;
    
    let mut file = File::open(path).context("Failed to open audio file")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).context("Failed to read audio file")?;
    
    // Simple WAV parsing (assuming 16-bit PCM, 16kHz, mono)
    // Skip the WAV header (44 bytes typically)
    if buffer.len() < 44 {
        anyhow::bail!("Audio file too small to be a valid WAV");
    }
    
    let audio_data = &buffer[44..];
    let mut samples = Vec::with_capacity(audio_data.len() / 2);
    
    // Convert 16-bit PCM to f32
    for chunk in audio_data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample as f32 / 32768.0);
    }
    
    Ok(samples)
}
