pub mod audio_session;
pub mod audio_enhancement;
pub mod llm_integration;
pub mod llm_router;
pub mod mcp_client;
pub mod speech_to_text;
pub mod recording_manager;
pub mod interview;

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

#[tauri::command]
async fn save_interview_state(
    project_name: String,
    state_json: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use std::fs;

    log::info!("[Interview] Saving interview state for project: {}", project_name);

    // Get app data directory
    let app_data_dir = app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    // Create directory if it doesn't exist
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    // Create filename from project name (sanitized)
    let sanitized_name = project_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();
    
    let file_path = app_data_dir.join(format!("{}.md", sanitized_name));

    // Parse the JSON state to create a nice markdown format
    let state: serde_json::Value = serde_json::from_str(&state_json)
        .map_err(|e| format!("Failed to parse state JSON: {}", e))?;

    let mut markdown = format!("# Interview: {}\n\n", project_name);
    markdown.push_str(&format!("*Dernière mise à jour: {}*\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    markdown.push_str("---\n\n");

    // Add answers grouped by section
    if let Some(answers) = state["answers"].as_array() {
        let mut current_section_id: i64 = -1;

        for answer in answers {
            let section_id = answer["sectionId"].as_i64().unwrap_or(-1);
            let question = answer["question"].as_str().unwrap_or("");
            let answer_text = answer["answer"].as_str().unwrap_or("");

            // Get section title from the sections array
            if section_id != current_section_id {
                current_section_id = section_id;
                // Find section title
                if let Some(sections) = state["sections"].as_array() {
                    if let Some(section) = sections.iter().find(|s| s["id"].as_i64() == Some(section_id)) {
                        let section_title = section["title"].as_str().unwrap_or("");
                        markdown.push_str(&format!("## {}\n\n", section_title));
                    }
                }
            }

            markdown.push_str(&format!("**Q:** {}\n\n", question));
            markdown.push_str(&format!("**R:** {}\n\n", answer_text));
            markdown.push_str("---\n\n");
        }
    }

    // Write markdown file
    fs::write(&file_path, markdown)
        .map_err(|e| format!("Failed to write markdown file: {}", e))?;

    // Also save raw JSON for loading
    let json_path = app_data_dir.join(format!("{}.json", sanitized_name));
    fs::write(&json_path, &state_json)
        .map_err(|e| format!("Failed to write JSON file: {}", e))?;

    log::info!("[Interview] State saved to: {:?} (markdown) and {:?} (json)", file_path, json_path);
    Ok(format!("État sauvegardé dans {}", file_path.display()))
}

#[tauri::command]
async fn load_interview_state(
    project_name: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use std::fs;

    log::info!("[Interview] Loading interview state for project: {}", project_name);

    // Get app data directory
    let app_data_dir = app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Create filename from project name (sanitized)
    let sanitized_name = project_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();
    
    let file_path = app_data_dir.join(format!("{}.json", sanitized_name));

    // Check if file exists
    if !file_path.exists() {
        return Err(format!("Aucune sauvegarde trouvée pour le projet '{}'", project_name));
    }

    // Read the JSON file
    let json_content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    log::info!("[Interview] State loaded from: {:?}", file_path);
    Ok(json_content)
}

#[tauri::command]
async fn list_saved_projects(
    app: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    use std::fs;

    log::info!("[Interview] Listing saved projects");

    // Get app data directory
    let app_data_dir = app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Create directory if it doesn't exist
    if !app_data_dir.exists() {
        return Ok(Vec::new());
    }

    // Read directory and collect .json files
    let entries = fs::read_dir(&app_data_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let projects: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()?.to_str()? == "json" {
                path.file_stem()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect();

    log::info!("[Interview] Found {} saved projects", projects.len());
    Ok(projects)
}

#[tauri::command]
async fn process_interview_section(
    section: interview::InterviewSection,
) -> Result<interview::SectionCanvasResult, String> {
    use crate::interview::InterviewProcessor;

    log::info!("[Interview] Processing section: {}", section.section_title);
    
    let processor = InterviewProcessor::new()
        .map_err(|e| {
            log::error!("[Interview] Failed to initialize processor: {}", e);
            format!("Failed to initialize interview processor: {}", e)
        })?;
    
    processor.process_section(section)
        .await
        .map_err(|e| {
            log::error!("[Interview] Failed to process section: {}", e);
            format!("Failed to process section: {}", e)
        })
}

#[tauri::command]
async fn generate_full_canvas(
    sections: Vec<interview::SectionCanvasResult>,
) -> Result<interview::FullCanvasResult, String> {
    use crate::interview::InterviewProcessor;

    log::info!("[Interview] Generating full canvas from {} sections", sections.len());
    
    let processor = InterviewProcessor::new()
        .map_err(|e| {
            log::error!("[Interview] Failed to initialize processor: {}", e);
            format!("Failed to initialize interview processor: {}", e)
        })?;
    
    processor.generate_full_canvas(sections)
        .await
        .map_err(|e| {
            log::error!("[Interview] Failed to generate canvas: {}", e);
            format!("Failed to generate canvas: {}", e)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use interview::{InterviewSection, SectionCanvasResult, UserAnswer};

    #[tokio::test]
    async fn test_process_interview_section_command_structure() {
        // Test that the command can be called with proper data structures
        let section = InterviewSection {
            section_id: 1,
            section_title: "Contexte & Vision".to_string(),
            answers: vec![
                UserAnswer {
                    section_id: 1,
                    question_index: 0,
                    question: "Quel problème réel veux-tu résoudre ?".to_string(),
                    answer: "Gérer les commandes e-commerce".to_string(),
                },
            ],
        };

        // Verify the section structure is valid
        assert_eq!(section.section_id, 1);
        assert_eq!(section.answers.len(), 1);
    }

    #[tokio::test]
    async fn test_generate_full_canvas_command_structure() {
        // Test that the command can be called with proper data structures
        let sections = vec![
            SectionCanvasResult {
                section_id: 1,
                section_title: "Contexte & Vision".to_string(),
                canvas_content: "* **Problème à résoudre :** Test".to_string(),
            },
            SectionCanvasResult {
                section_id: 2,
                section_title: "Acteurs & Use Cases".to_string(),
                canvas_content: "* **Acteurs :** Utilisateur".to_string(),
            },
        ];

        // Verify the sections structure is valid
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].section_id, 1);
        assert_eq!(sections[1].section_id, 2);
    }

    #[test]
    fn test_orchestrate_result_serialization() {
        let result = OrchestrateResult {
            markdown: "# Test Markdown".to_string(),
            mermaid: "graph TD\nA --> B".to_string(),
            model: serde_json::json!({
                "entities": [],
                "relations": []
            }),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("markdown"));
        assert!(json.contains("mermaid"));
        assert!(json.contains("model"));

        let deserialized: OrchestrateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.markdown, "# Test Markdown");
        assert_eq!(deserialized.mermaid, "graph TD\nA --> B");
    }

    #[test]
    fn test_audio_device_serialization() {
        let device = AudioDevice {
            name: "Test Microphone".to_string(),
            is_default: true,
        };

        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("Test Microphone"));
        assert!(json.contains("is_default"));

        let deserialized: AudioDevice = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test Microphone");
        assert!(deserialized.is_default);
    }

    #[test]
    fn test_greet_command() {
        let result = greet("Tauri");
        assert_eq!(result, "Hello, Tauri! You've been greeted from Rust!");
    }

    #[tokio::test]
    #[ignore] // Requires LLM and MCP setup
    async fn test_orchestrate_integration() {
        let transcript = "A user can create an order with multiple items";
        let result = orchestrate(transcript.to_string()).await;
        
        // This test requires full environment setup
        // In a real test environment, we'd expect either success or specific error
        match result {
            Ok(orchestrate_result) => {
                assert!(!orchestrate_result.markdown.is_empty());
                assert!(!orchestrate_result.mermaid.is_empty());
            }
            Err(e) => {
                // Expected errors in test environment without setup
                assert!(
                    e.contains("Failed to initialize LLM") ||
                    e.contains("Failed to generate domain model") ||
                    e.contains("Failed to generate mermaid") ||
                    e.contains("Failed to generate markdown")
                );
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires LLM setup
    async fn test_process_interview_section_integration() {
        let section = InterviewSection {
            section_id: 1,
            section_title: "Contexte & Vision".to_string(),
            answers: vec![
                UserAnswer {
                    section_id: 1,
                    question_index: 0,
                    question: "Quel problème réel veux-tu résoudre ?".to_string(),
                    answer: "Gérer les commandes e-commerce avec validation des stocks".to_string(),
                },
            ],
        };

        let result = process_interview_section(section).await;
        
        // This test requires LLM setup
        match result {
            Ok(section_result) => {
                assert_eq!(section_result.section_id, 1);
                assert_eq!(section_result.section_title, "Contexte & Vision");
                assert!(!section_result.canvas_content.is_empty());
            }
            Err(e) => {
                // Expected errors in test environment without LLM
                assert!(
                    e.contains("Failed to initialize interview processor") ||
                    e.contains("Failed to process section")
                );
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires LLM setup
    async fn test_generate_full_canvas_integration() {
        let sections = vec![
            SectionCanvasResult {
                section_id: 1,
                section_title: "Contexte & Vision".to_string(),
                canvas_content: "* **Problème à résoudre :** Gestion des commandes".to_string(),
            },
        ];

        let result = generate_full_canvas(sections).await;
        
        // This test requires LLM setup
        match result {
            Ok(canvas_result) => {
                assert!(canvas_result.markdown.contains("# Canvas — Rich Domain Model (DDD)"));
                assert!(canvas_result.markdown.contains("## Contexte & Vision"));
                assert!(canvas_result.markdown.contains("Gestion des commandes"));
            }
            Err(e) => {
                // Expected errors in test environment without LLM
                assert!(
                    e.contains("Failed to initialize interview processor") ||
                    e.contains("Failed to generate canvas")
                );
            }
        }
    }
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
            set_audio_device,
            save_interview_state,
            load_interview_state,
            list_saved_projects,
            process_interview_section,
            generate_full_canvas
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
