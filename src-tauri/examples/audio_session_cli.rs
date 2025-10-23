use domain_model_note_taking_lib::audio_session::{AudioSession, AudioSessionConfig};
use std::path::PathBuf;
use webrtc_vad::VadMode;

fn main() -> anyhow::Result<()> {
    println!("=== Audio Session Simulator ===");
    println!("This will record audio and automatically detect and save utterances.");
    println!();

    // Configuration personnalisable
    let config = AudioSessionConfig {
        silence_duration_ms: 800,  // 800ms de silence pour terminer une utterance
        min_utterance_duration_ms: 500,  // Ignorer les utterances < 500ms
        output_dir: PathBuf::from("/tmp/utterances"),
        vad_mode: VadMode::Aggressive,
    };

    println!("Configuration:");
    println!("  - Silence threshold: {}ms", config.silence_duration_ms);
    println!("  - Min utterance duration: {}ms", config.min_utterance_duration_ms);
    println!("  - VAD mode: {:?}", config.vad_mode);
    println!("  - Output directory: {:?}", config.output_dir);
    println!();

    // Créer la session
    let session = AudioSession::new(config)?;

    // Démarrer l'enregistrement (bloquant)
    session.start_recording()?;

    Ok(())
}
