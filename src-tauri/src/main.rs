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
}

fn main() {
    let args = Args::parse();

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
