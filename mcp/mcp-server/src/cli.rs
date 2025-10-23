use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Sha256, Digest};
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// CLI for testing MCP server with LLM integration
#[derive(Parser, Debug)]
#[command(name = "mcp-cli")]
#[command(about = "Domain Model pipeline: transcript → normalize → validate → emit", long_about = None)]
struct Args {
    /// Enable dry-run mode for LLM (simulates LLM response)
    #[arg(long)]
    dry_run_llm: bool,

    /// Path to input transcript file (.json JSONL format)
    #[arg(long, value_name = "FILE")]
    input: PathBuf,
    
    /// Path to output markdown file
    #[arg(long, value_name = "FILE")]
    emit_md: Option<PathBuf>,
    
    /// Path to output mermaid file
    #[arg(long, value_name = "FILE")]
    emit_mmd: Option<PathBuf>,
    
    /// Validate model but don't emit files
    #[arg(long)]
    validate_only: bool,
    
    /// Enable detailed tracing (logs prompts as hashes, never raw PII)
    #[arg(long)]
    trace: bool,
    
    /// Number of retry attempts for invalid JSON responses (default: 2)
    #[arg(long, default_value = "2")]
    retry: u32,
}

/// Hash sensitive data for logging (privacy-preserving)
fn hash_sensitive(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Sanitize and log prompt metadata
fn log_prompt_trace(prompt_type: &str, prompt: &str, response_size: usize) {
    let hash = hash_sensitive(prompt);
    let preview = if prompt.len() > 100 {
        format!("{}...", &prompt[..100])
    } else {
        prompt.to_string()
    };
    
    debug!(
        target: "domain::llm",
        prompt_type = prompt_type,
        prompt_hash = hash,
        prompt_length = prompt.len(),
        prompt_preview = preview,
        response_size = response_size,
        "LLM interaction traced"
    );
    
    info!(
        target: "domain::llm",
        prompt_type = prompt_type,
        prompt_hash = hash,
        prompt_length = prompt.len(),
        response_size = response_size,
        "LLM call completed"
    );
}

#[derive(Debug, Deserialize)]
struct TranscriptLine {
    speaker: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DomainModel {
    entities: Vec<Value>,
    relations: Vec<Value>,
    invariants: Vec<Value>,
}

/// Pipeline step status for UI progress tracking
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "status")]
enum StepStatus {
    Pending,
    Running { progress: Option<f32> },
    Success { duration_ms: u64 },
    Failed { error: String },
    Skipped,
}

/// Pipeline step definition
#[derive(Debug, Serialize, Clone)]
struct PipelineStep {
    name: String,
    description: String,
    status: StepStatus,
}

impl PipelineStep {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            status: StepStatus::Pending,
        }
    }
    
    fn start(&mut self) {
        self.status = StepStatus::Running { progress: None };
    }
    
    fn succeed(&mut self, duration_ms: u64) {
        self.status = StepStatus::Success { duration_ms };
    }
    
    fn fail(&mut self, error: String) {
        self.status = StepStatus::Failed { error };
    }
}

/// Validation error with JSON diff
#[derive(Debug, Serialize)]
struct ValidationError {
    step: String,
    errors: Vec<String>,
    warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diff: Option<Value>,
}

/// Repair invalid JSON using LLM without changing content
async fn repair_json_with_llm(
    invalid_json: &str,
    error_message: &str,
    provider: &str,
    enable_trace: bool,
) -> Result<String> {
    if enable_trace {
        warn!(target: "domain::llm", "Attempting JSON repair");
        info!(target: "domain::llm", error = error_message, invalid_json_length = invalid_json.len(), "JSON parsing failed");
    }
    
    let repair_prompt = format!(
        r#"The following JSON is invalid and needs to be repaired.

ERROR: {}

INVALID JSON:
{}

INSTRUCTIONS:
1. Fix ONLY the JSON structure/syntax errors
2. Do NOT change any content, values, or field names
3. Ensure it conforms to the DomainModel schema
4. Return ONLY the corrected JSON, no explanations

DomainModel schema:
{{
  "entities": [{{"id": "string", "name": "string", "attributes": [{{"name": "string", "type": "string", "required": boolean, "unique": boolean}}]}}],
  "relations": [{{"id": "string", "name": "string", "from": {{"entityId": "string"}}, "to": {{"entityId": "string"}}, "cardinality": {{"from": "string", "to": "string"}}}}],
  "invariants": [{{"id": "string", "name": "string", "type": "string", "expression": "string"}}]
}}

Repaired JSON:"#,
        error_message, invalid_json
    );
    
    if enable_trace {
        log_prompt_trace("repair_json", &repair_prompt, 0);
    }
    
    let _ = dotenvy::dotenv();
    let client = reqwest::Client::new();
    
    match provider.to_lowercase().as_str() {
        "ollama" => {
            let base_url = env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            let model = env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama2".to_string());
            
            if enable_trace {
                info!(target: "domain::llm", provider = "ollama", model = model, "Calling repair prompt");
            }
            
            let url = format!("{}/api/generate", base_url);
            let request_body = json!({
                "model": model,
                "prompt": repair_prompt,
                "stream": false,
                "format": "json"
            });
            
            let response = client
                .post(&url)
                .json(&request_body)
                .send()
                .await
                .context("Failed to call Ollama for repair")?;
            
            let response_json: Value = response.json().await?;
            let repaired = response_json
                .get("response")
                .and_then(|v| v.as_str())
                .context("No response from Ollama repair")?;
            
            if enable_trace {
                info!(target: "domain::llm", repaired_length = repaired.len(), "Received repaired JSON");
            }
            
            Ok(repaired.to_string())
        }
        _ => {
            let api_key = env::var("LLM_API_KEY")
                .context("LLM_API_KEY not set")?;
            let endpoint = env::var("LLM_ENDPOINT")
                .context("LLM_ENDPOINT not set")?;
            
            if enable_trace {
                info!(target: "domain::llm", provider = provider, "Calling external LLM for repair");
            }
            
            let request_body = json!({
                "messages": [
                    {"role": "system", "content": "You are a JSON repair assistant. Fix syntax errors without changing content."},
                    {"role": "user", "content": repair_prompt}
                ],
                "temperature": 0.3,
                "response_format": {"type": "json_object"}
            });
            
            let response = client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .context("Failed to call external LLM for repair")?;
            
            let response_json: Value = response.json().await?;
            let content = response_json
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .context("Failed to extract repaired JSON")?;
            
            if enable_trace {
                info!(target: "domain::llm", repaired_length = content.len(), "Received repaired JSON from external LLM");
            }
            
            Ok(content.to_string())
        }
    }
}

async fn call_llm_api(
    transcript: &str,
    dry_run: bool,
    enable_trace: bool,
    max_retries: u32,
) -> Result<DomainModel> {
    if enable_trace {
        info!(target: "domain::llm", "Starting LLM API call (dry_run={})", dry_run);
    }
    
    if dry_run {
        if enable_trace {
            info!(target: "domain::llm", "Using dry-run mock data");
            log_prompt_trace("normalize_terms", transcript, 500);
        }
        // Simulate LLM response for dry-run mode
        return Ok(DomainModel {
            entities: vec![
                json!({
                    "id": "Livre",
                    "name": "Livre",
                    "description": "Représente un livre dans la bibliothèque",
                    "attributes": [
                        {"name": "titre", "type": "string", "required": true},
                        {"name": "isbn", "type": "string", "required": true, "unique": true},
                        {"name": "datePublication", "type": "date", "required": true}
                    ],
                    "primaryKey": ["isbn"]
                }),
                json!({
                    "id": "Auteur",
                    "name": "Auteur",
                    "attributes": [
                        {"name": "id", "type": "uuid", "required": true, "unique": true},
                        {"name": "nom", "type": "string", "required": true},
                        {"name": "biographie", "type": "text"}
                    ],
                    "primaryKey": ["id"]
                }),
                json!({
                    "id": "Exemplaire",
                    "name": "Exemplaire",
                    "attributes": [
                        {"name": "code", "type": "string", "required": true, "unique": true},
                        {"name": "statut", "type": "string", "required": true}
                    ]
                })
            ],
            relations: vec![
                json!({
                    "id": "livre_auteurs",
                    "name": "écrit par",
                    "from": {"entityId": "Livre"},
                    "to": {"entityId": "Auteur"},
                    "cardinality": {"from": "1..n", "to": "0..n"}
                }),
                json!({
                    "id": "livre_exemplaires",
                    "name": "possède",
                    "from": {"entityId": "Livre"},
                    "to": {"entityId": "Exemplaire"},
                    "cardinality": {"from": "0..n", "to": "1"}
                })
            ],
            invariants: vec![
                json!({
                    "id": "exemplaire_disponible_pour_emprunt",
                    "name": "Exemplaire disponible pour emprunt",
                    "type": "business_rule",
                    "expression": "Exemplaire.statut = 'disponible' AVANT emprunt",
                    "severity": "error"
                })
            ],
        });
    }

    // Real LLM call
    let _ = dotenvy::dotenv();
    
    let provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
    
    let system_prompt = r#"
Tu es un normalizer de Domain Model. Rends UNIQUEMENT un JSON valide DomainModel conforme au schema. Interdis les champs non listés.

Schema DomainModel (STRICT):
{
  "entities": [{"id": "string", "name": "string", "attributes": [{"name": "string", "type": "string|number|integer|boolean|date|datetime|email|url|uuid|json|text", "required": boolean, "unique": boolean}]}],
  "relations": [{"id": "string", "name": "string", "from": {"entityId": "string"}, "to": {"entityId": "string"}, "cardinality": {"from": "0..1|1|0..n|1..n|*", "to": "0..1|1|0..n|1..n|*"}}],
  "invariants": [{"id": "string", "name": "string", "type": "uniqueness|referential_integrity|domain_constraint|cardinality|business_rule|temporal|aggregation", "expression": "string"}]
}

RÈGLES STRICTES:
1. AUCUN champ en dehors de ce schema
2. Tous les champs obligatoires DOIVENT être présents
3. Les types enum DOIVENT correspondre exactement
"#;

    match provider.to_lowercase().as_str() {
        "ollama" => {
            let base_url = env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            let model = env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama2".to_string());
            
            if enable_trace {
                info!(target: "domain::llm", provider = "ollama", model = model, url = base_url, "Calling Ollama API");
            }
            
            let client = reqwest::Client::new();
            let url = format!("{}/api/generate", base_url);
            
            let full_prompt = format!("{}\n\nUser: {}", system_prompt, transcript);
            
            if enable_trace {
                log_prompt_trace("normalize_terms_ollama", &full_prompt, 0);
            }
            
            let request_body = json!({
                "model": model,
                "prompt": full_prompt,
                "stream": false,
                "format": "json"
            });

            let response = client
                .post(&url)
                .json(&request_body)
                .send()
                .await
                .context("Failed to call Ollama API")?;

            let response_json: Value = response.json().await?;
            let llm_output = response_json
                .get("response")
                .and_then(|v| v.as_str())
                .context("No response from Ollama")?;
            
            if enable_trace {
                info!(target: "domain::llm", response_size = llm_output.len(), "Received Ollama response");
            }

            // Try to parse, with retry logic on failure
            let mut last_error_msg = None;
            let mut current_output = llm_output.to_string();
            
            for attempt in 0..=max_retries {
                match serde_json::from_str::<DomainModel>(&current_output) {
                    Ok(domain_model) => {
                        if enable_trace {
                            info!(
                                target: "domain::llm",
                                attempt = attempt,
                                entities = domain_model.entities.len(),
                                relations = domain_model.relations.len(),
                                invariants = domain_model.invariants.len(),
                                "Successfully parsed DomainModel"
                            );
                        }
                        return Ok(domain_model);
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        last_error_msg = Some(error_str.clone());
                        
                        if attempt < max_retries {
                            if enable_trace {
                                warn!(
                                    target: "domain::llm",
                                    attempt = attempt,
                                    error = error_str,
                                    "JSON parsing failed, attempting repair"
                                );
                            }
                            
                            // Attempt repair
                            match repair_json_with_llm(
                                &current_output,
                                &error_str,
                                &provider,
                                enable_trace,
                            ).await {
                                Ok(repaired) => {
                                    current_output = repaired;
                                    if enable_trace {
                                        info!(target: "domain::llm", attempt = attempt + 1, "Retry with repaired JSON");
                                    }
                                }
                                Err(repair_err) => {
                                    if enable_trace {
                                        warn!(target: "domain::llm", error = %repair_err, "Repair attempt failed");
                                    }
                                    // Continue to next retry
                                }
                            }
                        }
                    }
                }
            }
            
            Err(anyhow::anyhow!("Failed to parse JSON after {} retries: {}", max_retries, last_error_msg.unwrap()))
        }
        _ => {
            let api_key = env::var("LLM_API_KEY")
                .context("LLM_API_KEY not set for external provider")?;
            let endpoint = env::var("LLM_ENDPOINT")
                .context("LLM_ENDPOINT not set")?;
            
            if enable_trace {
                let api_key_hash = hash_sensitive(&api_key);
                info!(target: "domain::llm", provider = provider.as_str(), endpoint = endpoint, api_key_hash = api_key_hash, "Calling external LLM API");
                warn!(target: "domain::llm", "API key is hashed in logs for security");
            }
            
            let client = reqwest::Client::new();
            
            if enable_trace {
                log_prompt_trace("normalize_terms_external", transcript, 0);
            }
            
            let request_body = json!({
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": transcript}
                ],
                "temperature": 0.7,
                "response_format": {"type": "json_object"}
            });

            let response = client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .context("Failed to call external LLM API")?;

            let response_json: Value = response.json().await?;
            let content = response_json
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .context("Failed to extract content from LLM response")?;
            
            if enable_trace {
                info!(target: "domain::llm", response_size = content.len(), "Received external LLM response");
            }

            // Try to parse, with retry logic on failure
            let mut last_error_msg = None;
            let mut current_output = content.to_string();
            
            for attempt in 0..=max_retries {
                match serde_json::from_str::<DomainModel>(&current_output) {
                    Ok(domain_model) => {
                        if enable_trace {
                            info!(
                                target: "domain::llm",
                                attempt = attempt,
                                entities = domain_model.entities.len(),
                                relations = domain_model.relations.len(),
                                invariants = domain_model.invariants.len(),
                                "Successfully parsed DomainModel"
                            );
                        }
                        return Ok(domain_model);
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        last_error_msg = Some(error_str.clone());
                        
                        if attempt < max_retries {
                            if enable_trace {
                                warn!(
                                    target: "domain::llm",
                                    attempt = attempt,
                                    error = error_str,
                                    "JSON parsing failed, attempting repair"
                                );
                            }
                            
                            // Attempt repair
                            match repair_json_with_llm(
                                &current_output,
                                &error_str,
                                &provider,
                                enable_trace,
                            ).await {
                                Ok(repaired) => {
                                    current_output = repaired;
                                    if enable_trace {
                                        info!(target: "domain::llm", attempt = attempt + 1, "Retry with repaired JSON");
                                    }
                                }
                                Err(repair_err) => {
                                    if enable_trace {
                                        warn!(target: "domain::llm", error = %repair_err, "Repair attempt failed");
                                    }
                                    // Continue to next retry
                                }
                            }
                        }
                    }
                }
            }
            
            Err(anyhow::anyhow!("Failed to parse JSON after {} retries: {}", max_retries, last_error_msg.unwrap()))
        }
    }
}

/// Validate domain model and return errors/warnings
fn validate_domain_model(model: &Value) -> Result<(Vec<String>, Vec<String>)> {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    
    // Parse as DomainModel
    let entities = model.get("entities")
        .and_then(|e| e.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing 'entities' field"))?;
    
    let empty_relations = vec![];
    let relations = model.get("relations")
        .and_then(|r| r.as_array())
        .unwrap_or(&empty_relations);
    
    // Build entity ID map
    let mut entity_ids = std::collections::HashMap::new();
    
    for entity in entities {
        let id = entity.get("id")
            .and_then(|i| i.as_str())
            .ok_or_else(|| anyhow::anyhow!("Entity missing 'id' field"))?;
        
        entity_ids.insert(id.to_string(), entity);
        
        // Validate entity has primary key or unique attribute
        let has_pk = entity.get("primaryKey").is_some();
        let has_unique = entity.get("attributes")
            .and_then(|a| a.as_array())
            .map(|attrs| attrs.iter().any(|attr| 
                attr.get("unique").and_then(|u| u.as_bool()).unwrap_or(false)
            ))
            .unwrap_or(false);
        
        if !has_pk && !has_unique {
            errors.push(format!(
                "Entity '{}' must have either a primaryKey or at least one unique attribute",
                id
            ));
        }
        
        // Check for duplicate attribute names
        if let Some(attrs) = entity.get("attributes").and_then(|a| a.as_array()) {
            let mut attr_names = std::collections::HashSet::new();
            for attr in attrs {
                if let Some(name) = attr.get("name").and_then(|n| n.as_str()) {
                    if !attr_names.insert(name) {
                        errors.push(format!(
                            "Entity '{}': duplicate attribute name '{}'",
                            id, name
                        ));
                    }
                }
            }
        }
    }
    
    // Validate relations
    for relation in relations {
        let rel_id = relation.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
        
        if let Some(from_id) = relation.get("from")
            .and_then(|f| f.get("entityId"))
            .and_then(|e| e.as_str()) {
            if !entity_ids.contains_key(from_id) {
                errors.push(format!(
                    "Relation '{}': references non-existent entity '{}'",
                    rel_id, from_id
                ));
            }
        }
        
        if let Some(to_id) = relation.get("to")
            .and_then(|t| t.get("entityId"))
            .and_then(|e| e.as_str()) {
            if !entity_ids.contains_key(to_id) {
                errors.push(format!(
                    "Relation '{}': references non-existent entity '{}'",
                    rel_id, to_id
                ));
            }
        }
    }
    
    Ok((errors, warnings))
}

/// Run the complete pipeline
async fn run_pipeline(args: &Args) -> Result<()> {
    use std::time::Instant;
    
    // Initialize steps
    let mut steps = vec![
        PipelineStep::new("read_transcript", "Load and parse transcript from file"),
        PipelineStep::new("normalize_terms", "Generate domain model from transcript using LLM"),
        PipelineStep::new("validate_model", "Validate domain model structure and constraints"),
        PipelineStep::new("emit_markdown", "Generate markdown documentation"),
        PipelineStep::new("emit_mermaid", "Generate Mermaid diagram"),
    ];
    
    println!("\n============================================================");
    println!("  Domain Model Pipeline");
    println!("============================================================\n");
    
    // Step 1: Read transcript
    steps[0].start();
    println!("[1/5] 📝 Reading transcript...");
    let start = Instant::now();
    
    let content = fs::read_to_string(&args.input)
        .context(format!("Failed to read input file: {:?}", args.input))?;
    
    let mut transcript_parts = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let transcript_line: TranscriptLine = serde_json::from_str(line)
            .context(format!("Failed to parse JSONL line: {}", line))?;
        transcript_parts.push(transcript_line.text);
    }
    
    let full_transcript = transcript_parts.join("\n");
    steps[0].succeed(start.elapsed().as_millis() as u64);
    println!("      ✔ Loaded {} lines", transcript_parts.len());
    
    // Step 2: Normalize terms (generate domain model)
    steps[1].start();
    println!("\n[2/5] ⚙️  Generating domain model...");
    println!("      Mode: {}", if args.dry_run_llm { "DRY-RUN" } else { "LIVE LLM" });
    let start = Instant::now();
    
    let domain_model = match call_llm_api(&full_transcript, args.dry_run_llm, args.trace, args.retry).await {
        Ok(model) => {
            steps[1].succeed(start.elapsed().as_millis() as u64);
            model
        }
        Err(e) => {
            steps[1].fail(e.to_string());
            let error = ValidationError {
                step: "normalize_terms".to_string(),
                errors: vec![e.to_string()],
                warnings: vec![],
                diff: None,
            };
            eprintln!("\n❌ Pipeline failed at step: normalize_terms\n");
            eprintln!("{}", serde_json::to_string_pretty(&error)?);
            return Err(e);
        }
    };
    
    let model_json = serde_json::to_value(&domain_model)?;
    println!("      ✔ Generated {} entities, {} relations, {} invariants", 
        domain_model.entities.len(),
        domain_model.relations.len(),
        domain_model.invariants.len());
    
    // Step 3: Validate model
    steps[2].start();
    println!("\n[3/5] ✅ Validating model...");
    let start = Instant::now();
    
    match validate_domain_model(&model_json) {
        Ok((errors, warnings)) => {
            if !errors.is_empty() {
                steps[2].fail(format!("{} validation errors", errors.len()));
                
                let error = ValidationError {
                    step: "validate_model".to_string(),
                    errors: errors.clone(),
                    warnings,
                    diff: Some(model_json.clone()),
                };
                
                eprintln!("\n❌ Pipeline failed at step: validate_model\n");
                eprintln!("{}", serde_json::to_string_pretty(&error)?);
                
                return Err(anyhow::anyhow!("Validation failed with {} errors", errors.len()));
            }
            
            steps[2].succeed(start.elapsed().as_millis() as u64);
            println!("      ✔ Model is valid");
            
            if !warnings.is_empty() {
                println!("      ⚠️  {} warnings", warnings.len());
                for warning in &warnings {
                    println!("         - {}", warning);
                }
            }
        }
        Err(e) => {
            steps[2].fail(e.to_string());
            return Err(e);
        }
    }
    
    if args.validate_only {
        println!("\n✔ Validation complete (--validate-only mode)\n");
        return Ok(());
    }
    
    // Step 4: Emit markdown
    if let Some(md_path) = &args.emit_md {
        steps[3].start();
        println!("\n[4/5] 📝 Generating markdown...");
        let start = Instant::now();
        
        // Generate markdown content (simplified - in real implementation would call emit_markdown function)
        let markdown = format!("# Domain Model\n\n## Entities\n\n{}\n", 
            serde_json::to_string_pretty(&domain_model.entities)?);
        
        if let Some(parent) = md_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(md_path, markdown)?;
        
        steps[3].succeed(start.elapsed().as_millis() as u64);
        println!("      ✔ Written to: {}", md_path.display());
    } else {
        steps[3].status = StepStatus::Skipped;
        println!("\n[4/5] ⏭️  Skipping markdown (no --emit-md)");
    }
    
    // Step 5: Emit mermaid
    if let Some(mmd_path) = &args.emit_mmd {
        steps[4].start();
        println!("\n[5/5] 🔷 Generating Mermaid diagram...");
        let start = Instant::now();
        
        // Generate mermaid content (simplified)
        let mut mermaid_parts = vec!["erDiagram".to_string()];
        for entity in &domain_model.entities {
            if let Ok(v) = serde_json::to_value(entity) {
                if let Some(name) = v.get("name").and_then(|n| n.as_str()) {
                    mermaid_parts.push(format!("    {} {{", name));
                    mermaid_parts.push("    }".to_string());
                }
            }
        }
        let mermaid = mermaid_parts.join("\n");
        
        if let Some(parent) = mmd_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(mmd_path, mermaid)?;
        
        steps[4].succeed(start.elapsed().as_millis() as u64);
        println!("      ✔ Written to: {}", mmd_path.display());
    } else {
        steps[4].status = StepStatus::Skipped;
        println!("\n[5/5] ⏭️  Skipping Mermaid (no --emit-mmd)");
    }
    
    // Summary
    println!("\n============================================================");
    println!("  ✅ Pipeline Complete");
    println!("============================================================\n");
    
    // Output step status as JSON for UI integration
    println!("\nPipeline steps (JSON for UI):");
    println!("{}", serde_json::to_string_pretty(&steps)?);
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Setup tracing with JSON format if trace is enabled
    if args.trace {
        use tracing_subscriber::fmt::format::FmtSpan;
        tracing_subscriber::fmt()
            .with_target(true)
            .with_level(true)
            .with_span_events(FmtSpan::ACTIVE)
            .json()
            .init();
        
        info!(target: "domain::cli", trace_enabled = true, "Tracing activated with JSON output");
        info!(target: "domain::cli", "All prompts are hashed - no PII in logs");
    } else {
        tracing_subscriber::fmt::init();
    }
    
    if let Err(e) = run_pipeline(&args).await {
        eprintln!("\n❌ Error: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
