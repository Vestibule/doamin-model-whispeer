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

    /// Output markdown file for transcription
    #[arg(long)]
    emit_md: Option<PathBuf>,

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
    if let Some(audio_path) = &args.stt_input {
        // Use default model if not provided
        let model_path = args.model.as_ref().map(|p| p.as_path()).unwrap_or_else(|| {
            std::path::Path::new("models/ggml-base.bin")
        });

        match whisper::transcribe_audio(model_path, audio_path) {
            Ok(text) => {
                // If --emit-md is provided, write to file
                if let Some(output_path) = &args.emit_md {
                    match write_markdown_transcript(output_path, &text) {
                        Ok(_) => {
                            println!("Transcription written to: {:?}", output_path);
                        }
                        Err(e) => {
                            eprintln!("Error writing markdown: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    // Otherwise print to stdout
                    println!("Transcription:");
                    println!("{}", text);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Run in GUI mode
        domain_model_note_taking_lib::run()
    }
}

fn write_markdown_transcript(path: &PathBuf, text: &str) -> std::io::Result<()> {
    use std::fs;
    use std::io::Write;

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    writeln!(file, "# Transcription\n")?;
    writeln!(file, "{}", text)?;
    
    Ok(())
}

fn initialize_markdown_file(path: &PathBuf) -> std::io::Result<()> {
    use std::fs;
    use std::io::Write;

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    writeln!(file, "# Live Transcription\n")?;
    writeln!(file, "*Recording started...*\n")?;
    
    Ok(())
}

fn append_to_markdown(path: &PathBuf, utterance_id: usize, text: &str) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .append(true)
        .open(path)?;
    
    writeln!(file, "## Segment {}\n", utterance_id)?;
    writeln!(file, "{}\n", text)?;
    
    Ok(())
}

fn transcription_worker(
    session: domain_model_note_taking_lib::audio_session::AudioSession,
    model_path: PathBuf,
    md_path: PathBuf,
) {
    use std::collections::HashSet;
    use std::time::Duration;

    println!("Transcription worker started");
    
    let mut processed_ids = HashSet::new();
    
    loop {
        std::thread::sleep(Duration::from_millis(500));
        
        let utterances = session.get_utterances();
        
        for utterance in utterances {
            if processed_ids.contains(&utterance.id) {
                continue;
            }
            
            println!("Transcribing segment {}...", utterance.id);
            
            match whisper::transcribe_audio(&model_path, &utterance.file_path) {
                Ok(text) => {
                    if let Err(e) = append_to_markdown(&md_path, utterance.id, &text) {
                        eprintln!("Error appending to markdown: {}", e);
                    } else {
                        println!("Segment {} transcribed: {}", utterance.id, text.chars().take(50).collect::<String>());
                    }
                    processed_ids.insert(utterance.id);
                }
                Err(e) => {
                    eprintln!("Error transcribing segment {}: {}", utterance.id, e);
                }
            }
        }
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
    
    // Get model path and output markdown path
    let model_path = args.model.as_ref().map(|p| p.as_path()).unwrap_or_else(|| {
        std::path::Path::new("models/ggml-base.bin")
    });
    
    let emit_md = args.emit_md.clone();
    
    if let Some(ref md_path) = emit_md {
        println!("Transcription output: {:?}", md_path);
        // Initialize the markdown file
        if let Err(e) = initialize_markdown_file(md_path) {
            eprintln!("Error initializing markdown file: {}", e);
            return Err(e.into());
        }
    }
    println!();

    let config = AudioSessionConfig {
        silence_duration_ms: args.max_chunk_ms,
        min_utterance_duration_ms: 300,
        output_dir,
        vad_mode,
        device_name: None,
        gain: 2.0,
        enable_agc: true,
        agc_target_level: 0.3,
        push_to_talk: false, // CLI mode uses VAD-based segmentation
    };

    let session = AudioSession::new(config)?;
    
    // Start transcription thread if markdown output is enabled
    if let Some(md_path) = emit_md {
        let model_path_owned = model_path.to_path_buf();
        let session_clone = session.clone();
        
        std::thread::spawn(move || {
            transcription_worker(session_clone, model_path_owned, md_path);
        });
    }
    
    session.start_recording()?;

    Ok(())
}
