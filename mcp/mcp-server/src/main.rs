use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

// Domain Model Types
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomainModel {
    entities: Vec<Entity>,
    relations: Vec<Relation>,
    invariants: Vec<Invariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entity {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    attributes: Vec<Attribute>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "primaryKey")]
    primary_key: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Attribute {
    name: String,
    #[serde(rename = "type")]
    attr_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unique: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Relation {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    from: RelationEnd,
    to: RelationEnd,
    cardinality: Cardinality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelationEnd {
    #[serde(rename = "entityId")]
    entity_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cardinality {
    from: String,
    to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Invariant {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "type")]
    inv_type: String,
    expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<String>,
}

// JSON-RPC Types
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// MCP Protocol Types
#[derive(Debug, Serialize)]
struct ToolDefinition {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

fn normalize_terms(_input_lang: &str, transcript: &str) -> Result<Value> {
    // Parse transcript and extract domain model structure
    // For now, simple keyword extraction - in production this would use NLP
    let lines: Vec<&str> = transcript.lines().collect();
    
    let mut entities = Vec::new();
    let relations: Vec<Value> = Vec::new();
    let mut invariants = Vec::new();
    
    // Simple heuristic: lines with "entity", "has", "must" keywords
    for line in lines {
        let line_lower = line.to_lowercase();
        
        if line_lower.contains("entity") || line_lower.contains("has attributes") {
            // Extract entity name (simplified)
            let words: Vec<&str> = line.split_whitespace().collect();
            if let Some(name_idx) = words.iter().position(|&w| w.to_lowercase() == "entity") {
                if let Some(&name) = words.get(name_idx + 1) {
                    entities.push(json!({
                        "name": name.trim_matches(|c: char| !c.is_alphanumeric()),
                        "attrs": []
                    }));
                }
            }
        }
        
        if line_lower.contains("must") || line_lower.contains("invariant") {
            invariants.push(line.trim().to_string());
        }
    }
    
    Ok(json!({
        "entities": entities,
        "relations": relations,
        "invariants": invariants
    }))
}

async fn normalize_terms_with_llm(input_lang: &str, transcript: &str) -> Result<Value> {
    use std::env;
    
    // Load .env if available
    let _ = dotenvy::dotenv();
    
    let provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
    
    // System prompt in the specified language
    let system_prompt = match input_lang {
        "en" => r#"
You are a Domain Model normalizer. Return ONLY valid DomainModel JSON conforming to the schema. No extra fields allowed.

DomainModel Schema (STRICT):
{
  "entities": [{"id": "string", "name": "string", "attributes": [{"name": "string", "type": "string|number|integer|boolean|date|datetime|email|url|uuid|json|text", "required": boolean, "unique": boolean}]}],
  "relations": [{"id": "string", "name": "string", "from": {"entityId": "string"}, "to": {"entityId": "string"}, "cardinality": {"from": "0..1|1|0..n|1..n|*", "to": "0..1|1|0..n|1..n|*"}}],
  "invariants": [{"id": "string", "name": "string", "type": "uniqueness|referential_integrity|domain_constraint|cardinality|business_rule|temporal|aggregation", "expression": "string"}]
}

STRICT RULES:
1. NO fields outside this schema
2. All required fields MUST be present
3. Enum types MUST match exactly
4. Respond ONLY with JSON, no tool_calls
"#,
        _ => r#"
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
4. Réponds UNIQUEMENT avec ce JSON
"#,
    };
    
    let client = reqwest::Client::new();
    let llm_response_json: Value;
    
    match provider.to_lowercase().as_str() {
        "ollama" => {
            let base_url = env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            let model = env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama2".to_string());
            
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

            if !response.status().is_success() {
                anyhow::bail!("Ollama API error: {}", response.status());
            }

            let response_json: Value = response.json().await?;
            let llm_output = response_json
                .get("response")
                .and_then(|v| v.as_str())
                .context("No response from Ollama")?;

            llm_response_json = serde_json::from_str(llm_output)
                .context("Failed to parse LLM output as JSON")?;
        }
        _ => {
            let api_key = env::var("LLM_API_KEY")
                .context("LLM_API_KEY not set for external provider")?;
            let endpoint = env::var("LLM_ENDPOINT")
                .context("LLM_ENDPOINT not set")?;
            
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

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                anyhow::bail!("External API error {}: {}", status, error_text);
            }

            let response_json: Value = response.json().await?;
            let content = response_json
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .context("Failed to extract content from LLM response")?;

            llm_response_json = serde_json::from_str(content)
                .context("Failed to parse LLM output as JSON")?;
        }
    }
    
    // Validate against JSON Schema
    validate_domain_model(&llm_response_json)?;
    
    Ok(llm_response_json)
}

fn validate_domain_model(model: &Value) -> Result<()> {
    use jsonschema::Validator;
    use std::fs;
    use std::path::PathBuf;
    
    // Load the schema - try multiple possible locations
    let possible_paths = vec![
        PathBuf::from("../../domain_model.schema.json"),
        PathBuf::from("../domain_model.schema.json"),
        PathBuf::from("./domain_model.schema.json"),
    ];
    
    let mut schema_content = None;
    for path in &possible_paths {
        if path.exists() {
            schema_content = Some(fs::read_to_string(path)
                .context(format!("Failed to read {:?}", path))?);
            break;
        }
    }
    
    let schema_content = schema_content
        .ok_or_else(|| anyhow::anyhow!(
            "Could not find domain_model.schema.json in any of: {:?}",
            possible_paths
        ))?;
    let schema: Value = serde_json::from_str(&schema_content)
        .context("Failed to parse schema JSON")?;
    
    // Compile the schema
    let validator = Validator::new(&schema)
        .map_err(|e| anyhow::anyhow!("Failed to compile JSON schema: {}", e))?;
    
    // Validate against JSON Schema
    if let Err(error) = validator.validate(model) {
        anyhow::bail!(
            "DomainModel JSON Schema validation failed: {}",
            error
        );
    }
    
    // Custom business rules validation
    validate_custom_rules(model)?;
    
    Ok(())
}

fn validate_custom_rules(model: &Value) -> Result<()> {
    use std::collections::HashSet;
    
    let mut errors = Vec::new();
    
    // Extract entities array
    let entities = model.get("entities")
        .and_then(|e| e.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'entities' field"))?;
    
    // Build entity ID map for relation validation
    let mut entity_ids = HashSet::new();
    
    // Rule 1: Au moins une PK par entité
    // Rule 2: Pas de doublon d'attribut (name)
    for (idx, entity) in entities.iter().enumerate() {
        let entity_id = entity.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        
        entity_ids.insert(entity_id);
        
        // Check for primary key
        let has_primary_key = entity.get("primaryKey").is_some();
        let empty_attrs = vec![];
        let attributes = entity.get("attributes")
            .and_then(|a| a.as_array())
            .unwrap_or(&empty_attrs);
        
        // Check if any attribute has unique=true as alternative to primaryKey
        let has_unique_attr = attributes.iter().any(|attr| {
            attr.get("unique")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        });
        
        if !has_primary_key && !has_unique_attr {
            errors.push(format!(
                "Entity '{}' (index {}) must have either a primaryKey or at least one unique attribute",
                entity_id, idx
            ));
        }
        
        // Check for duplicate attribute names
        let mut attr_names = HashSet::new();
        for (attr_idx, attr) in attributes.iter().enumerate() {
            if let Some(attr_name) = attr.get("name").and_then(|v| v.as_str()) {
                if !attr_names.insert(attr_name) {
                    errors.push(format!(
                        "Entity '{}' (index {}) has duplicate attribute '{}' at index {}",
                        entity_id, idx, attr_name, attr_idx
                    ));
                }
            }
        }
    }
    
    // Rule 3: Relations pointent vers des entités existantes
    let empty_relations = vec![];
    let relations = model.get("relations")
        .and_then(|r| r.as_array())
        .unwrap_or(&empty_relations);
    
    for (idx, relation) in relations.iter().enumerate() {
        let relation_id = relation.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        
        // Check 'from' entity
        if let Some(from_entity_id) = relation.get("from")
            .and_then(|f| f.get("entityId"))
            .and_then(|v| v.as_str()) 
        {
            if !entity_ids.contains(from_entity_id) {
                errors.push(format!(
                    "Relation '{}' (index {}) references non-existent entity '{}' in 'from'",
                    relation_id, idx, from_entity_id
                ));
            }
        }
        
        // Check 'to' entity
        if let Some(to_entity_id) = relation.get("to")
            .and_then(|t| t.get("entityId"))
            .and_then(|v| v.as_str()) 
        {
            if !entity_ids.contains(to_entity_id) {
                errors.push(format!(
                    "Relation '{}' (index {}) references non-existent entity '{}' in 'to'",
                    relation_id, idx, to_entity_id
                ));
            }
        }
    }
    
    // If there are validation errors, bail with all of them
    if !errors.is_empty() {
        anyhow::bail!(
            "DomainModel custom validation failed:\n  - {}",
            errors.join("\n  - ")
        );
    }
    
    Ok(())
}

#[cfg(test)]
mod tools {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires LLM_PROVIDER to be configured
    async fn normalize_terms_roundtrip() -> Result<()> {
        // Load .env for test
        let _ = dotenvy::dotenv();
        
        let transcript = r#"
Un système de bibliothèque simple:
- Un Livre a un titre (obligatoire), un ISBN unique, et une date de publication
- Un Auteur a un nom obligatoire et une biographie optionnelle
- Un Livre est écrit par au moins un Auteur (1..n)
- Un Auteur peut écrire zéro ou plusieurs Livres (0..n)
- Invariant: L'ISBN doit être unique dans tout le système
"#;

        println!("\n🧪 Testing normalize_terms with LLM...");
        println!("Transcript:\n{}", transcript);
        
        let result = normalize_terms_with_llm("fr", transcript).await?;
        
        println!("\n✅ LLM Response:");
        println!("{}", serde_json::to_string_pretty(&result)?);
        
        // Verify structure
        assert!(result.get("entities").is_some(), "Missing 'entities' field");
        assert!(result.get("relations").is_some(), "Missing 'relations' field");
        assert!(result.get("invariants").is_some(), "Missing 'invariants' field");
        
        let entities = result.get("entities").unwrap().as_array()
            .expect("entities should be an array");
        assert!(!entities.is_empty(), "entities array should not be empty");
        
        // Check that at least one entity has the expected structure
        let first_entity = &entities[0];
        assert!(first_entity.get("id").is_some(), "Entity missing 'id'");
        assert!(first_entity.get("name").is_some(), "Entity missing 'name'");
        assert!(first_entity.get("attributes").is_some(), "Entity missing 'attributes'");
        
        println!("\n✅ All validations passed!");
        
        Ok(())
    }
    
    #[test]
    fn test_validate_domain_model_valid() -> Result<()> {
        let valid_model = json!({
            "entities": [
                {
                    "id": "User",
                    "name": "User",
                    "attributes": [
                        {"name": "email", "type": "email", "required": true}
                    ]
                }
            ],
            "relations": [],
            "invariants": []
        });
        
        // Should not panic
        validate_domain_model(&valid_model)?;
        
        Ok(())
    }
    
    #[test]
    fn test_validate_domain_model_invalid() {
        let invalid_model = json!({
            "entities": [
                {
                    "id": "User",
                    "name": "User",
                    "attributes": [
                        {"name": "email", "type": "invalid_type", "required": true}
                    ]
                }
            ],
            "relations": [],
            "invariants": []
        });
        
        let result = validate_domain_model(&invalid_model);
        assert!(result.is_err(), "Should fail validation with invalid type");
    }
    
    #[test]
    fn validate_model_conflicts() -> Result<()> {
        println!("\n🧪 Testing custom validation rules...");
        
        // Test 1: Missing primary key
        println!("\n📋 Test 1: Entity without primary key or unique attribute");
        let no_pk_model = json!({
            "entities": [
                {
                    "id": "User",
                    "name": "User",
                    "attributes": [
                        {"name": "email", "type": "email", "required": true}
                    ]
                }
            ],
            "relations": [],
            "invariants": []
        });
        
        match validate_domain_model(&no_pk_model) {
            Err(e) => {
                let error_msg = e.to_string();
                println!("   ✅ Correctly rejected: {}", error_msg);
                assert!(error_msg.contains("must have either a primaryKey"), 
                    "Should mention missing primaryKey");
            }
            Ok(_) => panic!("Should fail validation without primary key"),
        }
        
        // Test 2: Duplicate attribute names
        println!("\n📋 Test 2: Entity with duplicate attribute names");
        let duplicate_attr_model = json!({
            "entities": [
                {
                    "id": "User",
                    "name": "User",
                    "attributes": [
                        {"name": "email", "type": "email", "required": true},
                        {"name": "email", "type": "string", "required": false}
                    ],
                    "primaryKey": ["email"]
                }
            ],
            "relations": [],
            "invariants": []
        });
        
        match validate_domain_model(&duplicate_attr_model) {
            Err(e) => {
                let error_msg = e.to_string();
                println!("   ✅ Correctly rejected: {}", error_msg);
                assert!(error_msg.contains("duplicate attribute"), 
                    "Should mention duplicate attribute");
            }
            Ok(_) => panic!("Should fail validation with duplicate attributes"),
        }
        
        // Test 3: Relations pointing to non-existent entities
        println!("\n📋 Test 3: Relation pointing to non-existent entity");
        let invalid_relation_model = json!({
            "entities": [
                {
                    "id": "User",
                    "name": "User",
                    "attributes": [
                        {"name": "id", "type": "uuid", "required": true, "unique": true}
                    ]
                }
            ],
            "relations": [
                {
                    "id": "user_orders",
                    "name": "has orders",
                    "from": {"entityId": "User"},
                    "to": {"entityId": "Order"},
                    "cardinality": {"from": "1", "to": "0..n"}
                }
            ],
            "invariants": []
        });
        
        match validate_domain_model(&invalid_relation_model) {
            Err(e) => {
                let error_msg = e.to_string();
                println!("   ✅ Correctly rejected: {}", error_msg);
                assert!(error_msg.contains("non-existent entity"), 
                    "Should mention non-existent entity");
                assert!(error_msg.contains("Order"), 
                    "Should mention the missing entity name");
            }
            Ok(_) => panic!("Should fail validation with non-existent entity reference"),
        }
        
        // Test 4: Valid model with unique attribute (no primaryKey)
        println!("\n📋 Test 4: Valid entity with unique attribute instead of primaryKey");
        let valid_with_unique = json!({
            "entities": [
                {
                    "id": "User",
                    "name": "User",
                    "attributes": [
                        {"name": "email", "type": "email", "required": true, "unique": true}
                    ]
                }
            ],
            "relations": [],
            "invariants": []
        });
        
        validate_domain_model(&valid_with_unique)?;
        println!("   ✅ Valid model accepted");
        
        // Test 5: Valid model with all entities referenced
        println!("\n📋 Test 5: Valid model with proper relations");
        let valid_with_relations = json!({
            "entities": [
                {
                    "id": "User",
                    "name": "User",
                    "attributes": [
                        {"name": "id", "type": "uuid", "required": true, "unique": true}
                    ]
                },
                {
                    "id": "Order",
                    "name": "Order",
                    "attributes": [
                        {"name": "id", "type": "uuid", "required": true, "unique": true}
                    ]
                }
            ],
            "relations": [
                {
                    "id": "user_orders",
                    "name": "has orders",
                    "from": {"entityId": "User"},
                    "to": {"entityId": "Order"},
                    "cardinality": {"from": "1", "to": "0..n"}
                }
            ],
            "invariants": []
        });
        
        validate_domain_model(&valid_with_relations)?;
        println!("   ✅ Valid model with relations accepted");
        
        println!("\n✅ All custom validation tests passed!");
        
        Ok(())
    }
}

fn emit_markdown(model: &DomainModel, audience: Option<&str>) -> Result<Value> {
    let mut markdown = String::new();
    
    let title = match audience {
        Some("technical") => "# Domain Model - Technical Specification\n\n",
        Some("business") => "# Domain Model - Business Overview\n\n",
        _ => "# Domain Model\n\n",
    };
    markdown.push_str(title);
    
    // Entities section
    markdown.push_str("## Entities\n\n");
    for entity in &model.entities {
        markdown.push_str(&format!("### {}\n\n", entity.name));
        if let Some(desc) = &entity.description {
            markdown.push_str(&format!("{}\n\n", desc));
        }
        
        markdown.push_str("**Attributes:**\n\n");
        markdown.push_str("| Name | Type | Required | Unique |\n");
        markdown.push_str("|------|------|----------|--------|\n");
        
        for attr in &entity.attributes {
            markdown.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                attr.name,
                attr.attr_type,
                if attr.required.unwrap_or(false) { "✓" } else { "" },
                if attr.unique.unwrap_or(false) { "✓" } else { "" }
            ));
        }
        markdown.push_str("\n");
    }
    
    // Relations section
    markdown.push_str("## Relations\n\n");
    for relation in &model.relations {
        markdown.push_str(&format!("### {}\n\n", relation.name));
        if let Some(desc) = &relation.description {
            markdown.push_str(&format!("{}\n\n", desc));
        }
        markdown.push_str(&format!(
            "- From: {}\n- To: {}\n- Cardinality: {} to {}\n\n",
            relation.from.entity_id,
            relation.to.entity_id,
            relation.cardinality.from,
            relation.cardinality.to
        ));
    }
    
    // Invariants section
    markdown.push_str("## Business Invariants\n\n");
    for invariant in &model.invariants {
        markdown.push_str(&format!("### {}\n\n", invariant.name));
        if let Some(desc) = &invariant.description {
            markdown.push_str(&format!("{}\n\n", desc));
        }
        markdown.push_str(&format!("- Type: {}\n", invariant.inv_type));
        markdown.push_str(&format!("- Expression: `{}`\n", invariant.expression));
        if let Some(severity) = &invariant.severity {
            markdown.push_str(&format!("- Severity: {}\n", severity));
        }
        markdown.push_str("\n");
    }
    
    Ok(json!({
        "markdown": markdown
    }))
}

fn emit_mermaid(model: &DomainModel, style: Option<&str>) -> Result<Value> {
    let mut mermaid = String::new();
    
    let diagram_type = match style {
        Some("class") => "classDiagram",
        _ => "erDiagram", // default to 'er'
    };
    
    mermaid.push_str(&format!("{}\n", diagram_type));
    
    if style == Some("class") {
        // Generate class diagram
        for entity in &model.entities {
            mermaid.push_str(&format!("    class {} {{\n", entity.id));
            for attr in &entity.attributes {
                let visibility = if attr.required.unwrap_or(false) { "+" } else { "-" };
                mermaid.push_str(&format!("        {}{}: {}\n", visibility, attr.name, attr.attr_type));
            }
            mermaid.push_str("    }\n");
        }
        
        for relation in &model.relations {
            let arrow = match (relation.cardinality.from.as_str(), relation.cardinality.to.as_str()) {
                ("1", "1") => "--",
                ("1", _) => "-->",
                _ => "--*",
            };
            mermaid.push_str(&format!(
                "    {} {} {} : {}\n",
                relation.from.entity_id,
                arrow,
                relation.to.entity_id,
                relation.name
            ));
        }
        
        Ok(json!({
            "mermaid": mermaid
        }))
    } else {
        // Original ER diagram logic
        emit_er_diagram(model, &mut mermaid)?;
        Ok(json!({
            "mermaid": mermaid
        }))
    }
}

fn emit_er_diagram(model: &DomainModel, mermaid: &mut String) -> Result<()> {
    
    for entity in &model.entities {
        mermaid.push_str(&format!("    {} {{\n", entity.id));
        for attr in &entity.attributes {
            let type_str = match attr.attr_type.as_str() {
                "string" => "string",
                "number" | "integer" => "int",
                "boolean" => "bool",
                "date" | "datetime" => "date",
                "uuid" => "uuid",
                _ => "string",
            };
            let modifiers = if attr.required.unwrap_or(false) { " PK" } else { "" };
            mermaid.push_str(&format!("        {} {}{}\n", type_str, attr.name, modifiers));
        }
        mermaid.push_str("    }\n");
    }
    
    for relation in &model.relations {
        let from_card = match relation.cardinality.from.as_str() {
            "1" => "||",
            "0..1" => "|o",
            "0..n" | "*" => "}o",
            "1..n" => "}|",
            _ => "||",
        };
        
        let to_card = match relation.cardinality.to.as_str() {
            "1" => "||",
            "0..1" => "o|",
            "0..n" | "*" => "o{",
            "1..n" => "|{",
            _ => "||",
        };
        
        let label = relation.from.label.as_ref()
            .or(relation.to.label.as_ref())
            .map(|s| s.as_str())
            .unwrap_or(&relation.name);
        
        mermaid.push_str(&format!(
            "    {} {}--{} {} : \"{}\"\n",
            relation.from.entity_id,
            from_card,
            to_card,
            relation.to.entity_id,
            label
        ));
    }
    
    Ok(())
}

fn validate_model(model: &DomainModel, schema_path: Option<&str>) -> Result<Value> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    // Build entity ID map
    let entity_ids: HashMap<&str, &Entity> = model.entities
        .iter()
        .map(|e| (e.id.as_str(), e))
        .collect();
    
    // Validate entities
    for entity in &model.entities {
        // Check for duplicate attribute names
        let mut attr_names = std::collections::HashSet::new();
        for attr in &entity.attributes {
            if !attr_names.insert(&attr.name) {
                errors.push(format!(
                    "Entity '{}': Duplicate attribute name '{}'",
                    entity.id, attr.name
                ));
            }
        }
        
        // Validate primary key references
        if let Some(pk) = &entity.primary_key {
            for key in pk {
                if !entity.attributes.iter().any(|a| &a.name == key) {
                    errors.push(format!(
                        "Entity '{}': Primary key references non-existent attribute '{}'",
                        entity.id, key
                    ));
                }
            }
        }
    }
    
    // Validate relations
    for relation in &model.relations {
        // Check entity references
        if !entity_ids.contains_key(relation.from.entity_id.as_str()) {
            errors.push(format!(
                "Relation '{}': References non-existent entity '{}'",
                relation.id, relation.from.entity_id
            ));
        }
        if !entity_ids.contains_key(relation.to.entity_id.as_str()) {
            errors.push(format!(
                "Relation '{}': References non-existent entity '{}'",
                relation.id, relation.to.entity_id
            ));
        }
        
        // Validate cardinality
        let valid_cards = ["0..1", "1", "0..n", "1..n", "*"];
        if !valid_cards.contains(&relation.cardinality.from.as_str()) {
            warnings.push(format!(
                "Relation '{}': Invalid cardinality '{}'",
                relation.id, relation.cardinality.from
            ));
        }
        if !valid_cards.contains(&relation.cardinality.to.as_str()) {
            warnings.push(format!(
                "Relation '{}': Invalid cardinality '{}'",
                relation.id, relation.cardinality.to
            ));
        }
    }
    
    // Validate invariants
    for invariant in &model.invariants {
        if let Some(scope) = invariant.expression.split_whitespace().next() {
            if !entity_ids.contains_key(scope) && !scope.starts_with("forall") && !scope.starts_with("exists") {
                warnings.push(format!(
                    "Invariant '{}': Expression may reference unknown entities",
                    invariant.id
                ));
            }
        }
    }
    
    let is_valid = errors.is_empty();
    
    // If schema_path provided, validate against JSON schema
    if let Some(path) = schema_path {
        if std::path::Path::new(path).exists() {
            // Load and validate against schema (simplified - would use jsonschema crate in production)
            warnings.push(format!("Schema validation against '{}' not yet implemented", path));
        } else {
            warnings.push(format!("Schema file not found: {}", path));
        }
    }
    
    if is_valid {
        Ok(json!({
            "ok": true
        }))
    } else {
        Ok(json!({
            "ok": false,
            "errors": errors
        }))
    }
}

// Helper functions
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lower = false;
    
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && prev_lower {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_lower = false;
        } else {
            result.push(c);
            prev_lower = c.is_lowercase();
        }
    }
    
    result
}

fn to_title_case(s: &str) -> String {
    s.split(|c: char| c.is_whitespace() || c == '_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars.map(|c| c.to_lowercase()).flatten()).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn handle_initialize() -> Result<JsonRpcResponse> {
    Ok(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "domain-model-mcp-server",
                "version": "0.1.0"
            }
        })),
        error: None,
    })
}

fn handle_list_tools() -> Result<JsonRpcResponse> {
    let tools = vec![
        ToolDefinition {
            name: "generate_domain_model".to_string(),
            description: "Generate a complete DomainModel from natural language using LLM".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "transcript": {
                        "type": "string",
                        "description": "Natural language transcript describing the domain model"
                    },
                    "input_lang": {
                        "type": "string",
                        "description": "Input language code (e.g., 'en', 'fr')",
                        "default": "fr"
                    }
                },
                "required": ["transcript"]
            }),
        },
        ToolDefinition {
            name: "normalize_terms".to_string(),
            description: "Extract domain model from natural language transcript".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input_lang": {
                        "type": "string",
                        "description": "Input language code (e.g., 'en', 'fr')"
                    },
                    "transcript": {
                        "type": "string",
                        "description": "Natural language transcript describing the domain model"
                    }
                },
                "required": ["input_lang", "transcript"]
            }),
        },
        ToolDefinition {
            name: "emit_markdown".to_string(),
            description: "Generate Markdown documentation of the domain model".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "model": {
                        "type": "object",
                        "description": "The domain model to document"
                    },
                    "audience": {
                        "type": "string",
                        "description": "Target audience (e.g., 'technical', 'business')",
                        "enum": ["technical", "business"]
                    }
                },
                "required": ["model"]
            }),
        },
        ToolDefinition {
            name: "emit_mermaid".to_string(),
            description: "Generate Mermaid diagram of the domain model".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "model": {
                        "type": "object",
                        "description": "The domain model to visualize"
                    },
                    "style": {
                        "type": "string",
                        "description": "Diagram style",
                        "enum": ["er", "class"]
                    }
                },
                "required": ["model"]
            }),
        },
        ToolDefinition {
            name: "validate_model".to_string(),
            description: "Validate the domain model for consistency and correctness".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "model": {
                        "type": "object",
                        "description": "The domain model to validate"
                    },
                    "schema_path": {
                        "type": "string",
                        "description": "Optional path to JSON schema file for validation"
                    }
                },
                "required": ["model"]
            }),
        },
    ];
    
    Ok(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        result: Some(json!({
            "tools": tools
        })),
        error: None,
    })
}

async fn handle_tool_call(name: &str, params: &Value) -> Result<JsonRpcResponse> {
    let result = match name {
        "generate_domain_model" => {
            let transcript = params.get("transcript")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'transcript' parameter"))?;
            let _input_lang = params.get("input_lang")
                .and_then(|v| v.as_str())
                .unwrap_or("fr");
            
            // This will be implemented to call the LLM router
            // For now, return a placeholder
            json!({
                "status": "not_implemented",
                "message": "LLM integration required",
                "transcript_length": transcript.len()
            })
        }
        "normalize_terms" => {
            let input_lang = params.get("input_lang")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'input_lang' parameter"))?;
            let transcript = params.get("transcript")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'transcript' parameter"))?;
            normalize_terms_with_llm(input_lang, transcript).await?
        }
        "emit_markdown" => {
            let model_value = params.get("model")
                .ok_or_else(|| anyhow::anyhow!("Missing 'model' parameter"))?;
            let model: DomainModel = serde_json::from_value(model_value.clone())?;
            let audience = params.get("audience").and_then(|v| v.as_str());
            emit_markdown(&model, audience)?
        }
        "emit_mermaid" => {
            let model_value = params.get("model")
                .ok_or_else(|| anyhow::anyhow!("Missing 'model' parameter"))?;
            let model: DomainModel = serde_json::from_value(model_value.clone())?;
            let style = params.get("style").and_then(|v| v.as_str());
            emit_mermaid(&model, style)?
        }
        "validate_model" => {
            let model_value = params.get("model")
                .ok_or_else(|| anyhow::anyhow!("Missing 'model' parameter"))?;
            let model: DomainModel = serde_json::from_value(model_value.clone())?;
            let schema_path = params.get("schema_path").and_then(|v| v.as_str());
            validate_model(&model, schema_path)?
        }
        _ => return Err(anyhow::anyhow!("Unknown tool: {}", name)),
    };
    
    Ok(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        result: Some(result),
        error: None,
    })
}

async fn handle_request(req: JsonRpcRequest) -> Result<JsonRpcResponse> {
    match req.method.as_str() {
        "initialize" => handle_initialize(),
        "tools/list" => handle_list_tools(),
        "tools/call" => {
            let name = req.params.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
            let arguments = req.params.get("arguments")
                .ok_or_else(|| anyhow::anyhow!("Missing tool arguments"))?;
            handle_tool_call(name, arguments).await
        }
        _ => Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
                data: None,
            }),
        }),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = tokio::io::BufReader::new(stdin);
    let mut line = String::new();
    
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        
        if n == 0 {
            break; // EOF
        }
        
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        
        match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(req) => {
                let response = match handle_request(req).await {
                    Ok(resp) => resp,
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: format!("Internal error: {}", e),
                            data: None,
                        }),
                    },
                };
                
                let response_json = serde_json::to_string(&response)?;
                stdout.write_all(response_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
            Err(e) => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                
                let response_json = serde_json::to_string(&error_response)?;
                stdout.write_all(response_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }
    }
    
    Ok(())
}
