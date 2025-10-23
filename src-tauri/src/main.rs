// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use std::path::PathBuf;

mod whisper;

#[derive(Parser, Debug)]
#[command(name = "app")]
#[command(about = "Domain Model Note Taking Application", long_about = None)]
struct Args {
    /// Path to the audio file for speech-to-text
    #[arg(long)]
    stt_input: Option<PathBuf>,

    /// Path to the Whisper model file
    #[arg(long)]
    model: Option<PathBuf>,

    /// Enable audio streaming mode with VAD
    #[arg(long)]
    stream: bool,

    /// VAD threshold (0.0 to 1.0, higher = more aggressive)
    #[arg(long, default_value = "0.5")]
    vad_threshold: f32,

    /// Maximum chunk duration in milliseconds
    #[arg(long, default_value = "1000")]
    max_chunk_ms: u32,

    /// Output directory for audio chunks
    #[arg(long)]
    output_dir: Option<PathBuf>,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // Handle streaming mode
    if args.stream {
        if let Err(e) = run_streaming_mode(&args) {
            eprintln!("Error in streaming mode: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // If CLI arguments are provided, run in CLI mode
    if let (Some(audio_path), Some(model_path)) = (&args.stt_input, &args.model) {
        match whisper::transcribe_audio(model_path, audio_path) {
            Ok(text) => {
                println!("Transcription:");
                println!("{}", text);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else if args.stt_input.is_some() || args.model.is_some() {
        eprintln!("Error: Both --stt-input and --model must be provided together");
        std::process::exit(1);
    } else {
        // Run in GUI mode
        domain_model_note_taking_lib::run()
    }
}

fn run_streaming_mode(args: &Args) -> anyhow::Result<()> {
    use domain_model_note_taking_lib::audio_session::{AudioSession, AudioSessionConfig};
    use webrtc_vad::VadMode;

    println!("=== Audio Streaming Mode ===");
    println!("VAD Threshold: {}", args.vad_threshold);
    println!("Max Chunk Duration: {}ms", args.max_chunk_ms);
    println!();

    // Map threshold to VadMode
    // 0.0-0.25 = Quality, 0.25-0.5 = LowBitrate, 0.5-0.75 = Aggressive, 0.75-1.0 = VeryAggressive
    let vad_mode = match args.vad_threshold {
        t if t < 0.25 => VadMode::Quality,
        t if t < 0.5 => VadMode::LowBitrate,
        t if t < 0.75 => VadMode::Aggressive,
        _ => VadMode::VeryAggressive,
    };

    let output_dir = args.output_dir.clone().unwrap_or_else(|| {
        std::env::temp_dir().join("audio_chunks")
    });

    let vad_mode_str = match vad_mode {
        VadMode::Quality => "Quality",
        VadMode::LowBitrate => "LowBitrate",
        VadMode::Aggressive => "Aggressive",
        VadMode::VeryAggressive => "VeryAggressive",
    };

    println!("VAD Mode: {}", vad_mode_str);
    println!("Output Directory: {:?}", output_dir);
    println!();

    let config = AudioSessionConfig {
        silence_duration_ms: args.max_chunk_ms,
        min_utterance_duration_ms: 300,
        output_dir,
        vad_mode,
    };

    let session = AudioSession::new(config)?;
    session.start_recording()?;

    Ok(())
}
