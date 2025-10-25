pub mod audio_session;
pub mod audio_enhancement;
pub mod llm_integration;
pub mod llm_router;
pub mod mcp_client;
pub mod speech_to_text;
pub mod recording_manager;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::Manager;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestrateResult {
    pub markdown: String,
    pub mermaid: String,
    pub model: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn orchestrate(transcript: String) -> Result<OrchestrateResult, String> {
    use crate::llm_integration::LlmIntegration;
    use crate::mcp_client::McpClient;
    use std::env;

    log::info!("[Orchestrate] Starting orchestration for transcript: {}", &transcript[..transcript.len().min(100)]);

    // 1. Generate domain model from transcript using LLM
    log::info!("[Orchestrate] Initializing LLM integration...");
    let llm_integration = LlmIntegration::new()
        .map_err(|e| {
            log::error!("[Orchestrate] Failed to initialize LLM: {}", e);
            format!("Failed to initialize LLM: {}", e)
        })?;
    log::info!("[Orchestrate] LLM integration initialized successfully");
    
    log::info!("[Orchestrate] Generating domain model from transcript...");
    let model = llm_integration
        .process_request(&transcript)
        .await
        .map_err(|e| {
            log::error!("[Orchestrate] Failed to generate domain model: {}", e);
            format!("Failed to generate domain model: {}", e)
        })?;
    log::info!("[Orchestrate] Domain model generated successfully");

    // 2. Get MCP server path from environment
    let mcp_server_path = env::var("MCP_SERVER_PATH")
        .unwrap_or_else(|_| "../mcp/mcp-server/target/release/mcp-server".to_string());
    log::info!("[Orchestrate] Using MCP server at: {}", mcp_server_path);
    
    let mcp_client = McpClient::new(mcp_server_path);

    // 3. Generate Mermaid diagram from model
    log::info!("[Orchestrate] Generating Mermaid diagram...");
    let mermaid = mcp_client
        .emit_mermaid(model.clone(), Some("er"))
        .await
        .map_err(|e| {
            log::error!("[Orchestrate] Failed to generate mermaid: {}", e);
            format!("Failed to generate mermaid: {}", e)
        })?;
    log::info!("[Orchestrate] Mermaid diagram generated successfully");

    // 4. Generate Markdown documentation from model
    log::info!("[Orchestrate] Generating Markdown documentation...");
    let markdown = mcp_client
        .emit_markdown(model.clone(), None)
        .await
        .map_err(|e| {
            log::error!("[Orchestrate] Failed to generate markdown: {}", e);
            format!("Failed to generate markdown: {}", e)
        })?;
    log::info!("[Orchestrate] Markdown documentation generated successfully");

    log::info!("[Orchestrate] Orchestration completed successfully");
    Ok(OrchestrateResult {
        markdown,
        mermaid,
        model,
    })
}

#[tauri::command]
async fn start_recording(state: tauri::State<'_, Arc<Mutex<Option<recording_manager::RecordingManager>>>>) -> Result<String, String> {
    log::info!("[Command] start_recording called");
    let manager_guard = state.lock().unwrap();
    let manager = manager_guard.as_ref().ok_or("Recording manager not initialized")?;
    log::info!("[Command] RecordingManager found, calling start_recording");
    manager.start_recording()
        .map_err(|e| {
            log::error!("[Command] Failed to start recording: {}", e);
            format!("Failed to start recording: {}", e)
        })
}

#[tauri::command]
async fn stop_recording(state: tauri::State<'_, Arc<Mutex<Option<recording_manager::RecordingManager>>>>) -> Result<String, String> {
    log::info!("[Command] stop_recording called");
    let manager_guard = state.lock().unwrap();
    let manager = manager_guard.as_ref().ok_or("Recording manager not initialized")?;
    log::info!("[Command] RecordingManager found, calling stop_recording");
    manager.stop_recording()
        .map_err(|e| {
            log::error!("[Command] Failed to stop recording: {}", e);
            format!("Failed to stop recording: {}", e)
        })
}

#[tauri::command]
async fn transcribe_audio(
    audio_path: String,
    app: tauri::AppHandle,
) -> Result<speech_to_text::TranscriptionResult, String> {
    use crate::speech_to_text::SpeechToText;
    use std::env;
    use std::path::PathBuf;
    
    let model_path = if let Ok(path) = env::var("WHISPER_MODEL_PATH") {
        PathBuf::from(path)
    } else {
        // Try to get from bundled resources
        let resource_path = app.path().resolve("ggml-small.bin", tauri::path::BaseDirectory::Resource)
            .map_err(|e| format!("Failed to resolve resource path: {}", e))?;
        
        if resource_path.exists() {
            resource_path
        } else {
            // Fallback to dev mode path
            PathBuf::from("models/whisper/ggml-small.bin")
        }
    };
    
    let stt = SpeechToText::new(model_path);
    let audio_path_buf = PathBuf::from(audio_path);
    
    stt.transcribe_file(&audio_path_buf)
        .map_err(|e| format!("Transcription failed: {}", e))
}

#[tauri::command]
async fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    
    let host = cpal::default_host();
    let default_device = host.default_input_device();
    let default_name = default_device.as_ref().and_then(|d| d.name().ok());
    
    let devices: Vec<AudioDevice> = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate audio devices: {}", e))?
        .filter_map(|device| {
            device.name().ok().map(|name| AudioDevice {
                is_default: Some(name.clone()) == default_name,
                name,
            })
        })
        .collect();
    
    Ok(devices)
}

#[tauri::command]
async fn set_audio_device(
    device_name: String,
    state: tauri::State<'_, Arc<Mutex<Option<recording_manager::RecordingManager>>>>,
) -> Result<String, String> {
    let manager_guard = state.lock().unwrap();
    let manager = manager_guard.as_ref().ok_or("Recording manager not initialized")?;
    
    manager.set_audio_device(device_name.clone())
        .map_err(|e| format!("Failed to set audio device: {}", e))?;
    
    Ok(format!("Audio device set to: {}", device_name))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize RecordingManager
            let model_path = if let Ok(path_str) = std::env::var("WHISPER_MODEL_PATH") {
                std::path::PathBuf::from(path_str)
            } else {
                // Try to get from bundled resources first
                if let Ok(resource_path) = app.path().resolve("ggml-small.bin", tauri::path::BaseDirectory::Resource) {
                    if resource_path.exists() {
                        log::info!("[Setup] Using bundled model from resources: {:?}", resource_path);
                        resource_path
                    } else {
                        // Fallback to dev mode path
                        let base_dir = std::env::current_dir()
                            .unwrap_or_else(|_| std::path::PathBuf::from("."));
                        
                        // If we're in src-tauri directory, go up one level
                        let project_root = if base_dir.ends_with("src-tauri") {
                            base_dir.parent().unwrap_or(&base_dir).to_path_buf()
                        } else {
                            base_dir
                        };
                        
                        let dev_path = project_root.join("models/whisper/ggml-small.bin");
                        log::info!("[Setup] Using dev mode model path: {:?}", dev_path);
                        dev_path
                    }
                } else {
                    log::error!("[Setup] Failed to resolve resource path, using dev mode fallback");
                    let base_dir = std::env::current_dir()
                        .unwrap_or_else(|_| std::path::PathBuf::from("."));
                    let project_root = if base_dir.ends_with("src-tauri") {
                        base_dir.parent().unwrap_or(&base_dir).to_path_buf()
                    } else {
                        base_dir
                    };
                    project_root.join("models/whisper/ggml-small.bin")
                }
            };
            
            log::info!("[Setup] Initializing RecordingManager with model path: {:?}", model_path);
            
            // Verify model file exists
            if !model_path.exists() {
                log::error!("[Setup] Whisper model file not found at: {:?}", model_path);
                log::error!("[Setup] Please ensure the model file is present or set WHISPER_MODEL_PATH environment variable");
            } else {
                log::info!("[Setup] Whisper model file found");
            }
            
            let manager = recording_manager::RecordingManager::new(
                model_path,
                app.handle().clone(),
            );
            
            app.manage(Arc::new(Mutex::new(Some(manager))));
            log::info!("[Setup] RecordingManager initialized successfully");
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            orchestrate,
            start_recording,
            stop_recording,
            transcribe_audio,
            list_audio_devices,
            set_audio_device
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
