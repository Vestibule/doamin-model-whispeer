use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

/// CLI for testing MCP server with LLM integration
#[derive(Parser, Debug)]
#[command(name = "mcp-cli")]
#[command(about = "Test MCP server tools with LLM", long_about = None)]
struct Args {
    /// Enable dry-run mode for LLM (simulates LLM response)
    #[arg(long)]
    dry_run_llm: bool,

    /// Path to input transcript file (.jsonl format)
    #[arg(long, value_name = "FILE")]
    input: PathBuf,
}

#[derive(Debug, Deserialize)]
struct TranscriptLine {
    speaker: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DomainModel {
    entities: Vec<Value>,
    relations: Vec<Value>,
    invariants: Vec<Value>,
}

async fn call_llm_api(transcript: &str, dry_run: bool) -> Result<DomainModel> {
    if dry_run {
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
                        {"name": "nom", "type": "string", "required": true},
                        {"name": "biographie", "type": "text"}
                    ]
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
            
            let client = reqwest::Client::new();
            let url = format!("{}/api/generate", base_url);
            
            let request_body = json!({
                "model": model,
                "prompt": format!("{}\n\nUser: {}", system_prompt, transcript),
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

            let domain_model: DomainModel = serde_json::from_str(llm_output)
                .context("Failed to parse LLM output as DomainModel")?;

            Ok(domain_model)
        }
        _ => {
            let api_key = env::var("LLM_API_KEY")
                .context("LLM_API_KEY not set for external provider")?;
            let endpoint = env::var("LLM_ENDPOINT")
                .context("LLM_ENDPOINT not set")?;
            
            let client = reqwest::Client::new();
            
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

            let domain_model: DomainModel = serde_json::from_str(content)
                .context("Failed to parse LLM output as DomainModel")?;

            Ok(domain_model)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();

    // Read transcript from JSONL file
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

    println!("📝 Transcript loaded ({} lines)", transcript_parts.len());
    println!("🤖 Mode: {}", if args.dry_run_llm { "DRY-RUN" } else { "LIVE LLM" });
    println!();

    // Call LLM to generate DomainModel
    println!("⏳ Generating DomainModel from transcript...");
    let domain_model = call_llm_api(&full_transcript, args.dry_run_llm).await?;

    // Output result
    println!("✅ DomainModel generated successfully!\n");
    println!("{}", serde_json::to_string_pretty(&domain_model)?);

    Ok(())
}
