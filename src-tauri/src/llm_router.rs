use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

/// LLM Provider configuration
#[derive(Debug, Clone)]
pub enum LlmProvider {
    Ollama { base_url: String },
    External { api_key: String, endpoint: String },
}

impl LlmProvider {
    /// Detect provider from environment variables
    /// Automatically loads .env file if present
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists (silently ignore if not found)
        let _ = dotenvy::dotenv();
        
        let provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "external".to_string());
        
        match provider.to_lowercase().as_str() {
            "ollama" => {
                let base_url = env::var("OLLAMA_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".to_string());
                Ok(Self::Ollama { base_url })
            }
            _ => {
                let api_key = env::var("LLM_API_KEY")
                    .context("LLM_API_KEY environment variable not set")?;
                let endpoint = env::var("LLM_ENDPOINT")
                    .context("LLM_ENDPOINT environment variable not set")?;
                Ok(Self::External { api_key, endpoint })
            }
        }
    }
}

/// Tool call structure that LLM should emit
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

/// LLM response containing tool calls
#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    pub tool_calls: Vec<ToolCall>,
}

/// Direct DomainModel response (no tool_calls wrapper)
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainModelResponse {
    pub entities: Vec<Value>,
    pub relations: Vec<Value>,
    pub invariants: Vec<Value>,
}

/// Ollama API response structure
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    pub response: String,
    pub done: bool,
}

/// LLM Router that handles communication with different providers
pub struct LlmRouter {
    provider: LlmProvider,
    client: reqwest::Client,
}

impl LlmRouter {
    /// Create a new LLM router with provider from environment
    pub fn new() -> Result<Self> {
        let provider = LlmProvider::from_env()?;
        let client = reqwest::Client::new();
        Ok(Self { provider, client })
    }

    /// Send a prompt to the LLM and get structured tool calls back
    /// The LLM should never communicate directly with the UI
    pub async fn generate_tool_calls(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<LlmResponse> {
        match &self.provider {
            LlmProvider::Ollama { base_url } => {
                self.generate_with_ollama(base_url, system_prompt, user_prompt)
                    .await
            }
            LlmProvider::External { api_key, endpoint } => {
                self.generate_with_external(endpoint, api_key, system_prompt, user_prompt)
                    .await
            }
        }
    }

    /// Generate a DomainModel directly (no tool_calls wrapper)
    pub async fn generate_domain_model(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<DomainModelResponse> {
        match &self.provider {
            LlmProvider::Ollama { base_url } => {
                self.generate_domain_model_ollama(base_url, system_prompt, user_prompt)
                    .await
            }
            LlmProvider::External { api_key, endpoint } => {
                self.generate_domain_model_external(endpoint, api_key, system_prompt, user_prompt)
                    .await
            }
        }
    }

    /// Generate tool calls using Ollama local API
    async fn generate_with_ollama(
        &self,
        base_url: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<LlmResponse> {
        let url = format!("{}/api/generate", base_url);
        
        let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2".to_string());
        
        let request_body = json!({
            "model": model,
            "prompt": format!("{}\n\nUser: {}", system_prompt, user_prompt),
            "stream": false,
            "format": "json"
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama API error: {}", response.status());
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        // Parse the JSON response from Ollama into structured tool calls
        let llm_response: LlmResponse = serde_json::from_str(&ollama_response.response)
            .context("Failed to parse tool calls from Ollama response")?;

        Ok(llm_response)
    }

    /// Generate tool calls using external provider API
    async fn generate_with_external(
        &self,
        endpoint: &str,
        api_key: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<LlmResponse> {
        // This is a generic implementation - adjust based on your actual external provider
        let request_body = json!({
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.7,
            "response_format": {"type": "json_object"}
        });

        let response = self
            .client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to external provider")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("External API error {}: {}", status, error_text);
        }

        let response_json: Value = response
            .json()
            .await
            .context("Failed to parse external provider response")?;

        // Extract the content from the provider's response format
        // Adjust this based on your actual provider's response structure
        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .context("Failed to extract content from external provider response")?;

        let llm_response: LlmResponse = serde_json::from_str(content)
            .context("Failed to parse tool calls from external provider response")?;

        Ok(llm_response)
    }

    /// Generate DomainModel using Ollama
    async fn generate_domain_model_ollama(
        &self,
        base_url: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<DomainModelResponse> {
        let url = format!("{}/api/generate", base_url);
        let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "domain-model-mistral".to_string());
        
        let request_body = json!({
            "model": model,
            "prompt": format!("{}\n\nUser: {}", system_prompt, user_prompt),
            "stream": false,
            "format": "json"
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama API error: {}", response.status());
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        log::info!("[LLM Router] Ollama raw response: {}", &ollama_response.response);

        let domain_model: DomainModelResponse = serde_json::from_str(&ollama_response.response)
            .map_err(|e| {
                log::error!("[LLM Router] Failed to parse DomainModel. Error: {}. Response was: {}", e, &ollama_response.response);
                anyhow::anyhow!("Failed to parse DomainModel from Ollama response: {}. The LLM did not follow the expected schema.", e)
            })?;

        Ok(domain_model)
    }

    /// Generate DomainModel using external provider
    async fn generate_domain_model_external(
        &self,
        endpoint: &str,
        api_key: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<DomainModelResponse> {
        let request_body = json!({
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.7,
            "response_format": {"type": "json_object"}
        });

        let response = self
            .client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to external provider")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("External API error {}: {}", status, error_text);
        }

        let response_json: Value = response
            .json()
            .await
            .context("Failed to parse external provider response")?;

        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .context("Failed to extract content from external provider response")?;

        let domain_model: DomainModelResponse = serde_json::from_str(content)
            .context("Failed to parse DomainModel from external provider response")?;

        Ok(domain_model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_detection_ollama() {
        env::set_var("LLM_PROVIDER", "ollama");
        let provider = LlmProvider::from_env().unwrap();
        matches!(provider, LlmProvider::Ollama { .. });
    }

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCall {
            name: "normalize_terms".to_string(),
            arguments: json!({
                "input_lang": "en",
                "transcript": "User entity has email"
            }),
        };

        let response = LlmResponse {
            tool_calls: vec![tool_call],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("normalize_terms"));
        assert!(json.contains("tool_calls"));
    }
}
