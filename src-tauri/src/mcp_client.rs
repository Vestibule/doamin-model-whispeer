use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

pub struct McpClient {
    binary_path: String,
}

impl McpClient {
    pub fn new(binary_path: String) -> Self {
        Self { binary_path }
    }

    /// Launch the MCP server binary and initialize the connection
    async fn spawn_server(&self) -> Result<tokio::process::Child> {
        let child = Command::new(&self.binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn MCP server")?;

        Ok(child)
    }

    /// Send a JSON-RPC request and receive a response
    async fn call_method(
        stdin: &mut tokio::process::ChildStdin,
        stdout: &mut BufReader<tokio::process::ChildStdout>,
        method: &str,
        params: Value,
        id: u64,
    ) -> Result<JsonRpcResponse> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)?;
        stdin.write_all(request_json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        let mut response_line = String::new();
        stdout.read_line(&mut response_line).await?;

        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .context("Failed to parse JSON-RPC response")?;

        if let Some(error) = response.error {
            anyhow::bail!("JSON-RPC error {}: {}", error.code, error.message);
        }

        Ok(response)
    }

    /// Call the emit_mermaid tool with a domain model
    pub async fn emit_mermaid(
        &self,
        model: Value,
        style: Option<&str>,
    ) -> Result<String> {
        let mut child = self.spawn_server().await?;

        let mut stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let mut stdout_reader = BufReader::new(stdout);

        // Step 1: Initialize
        let _init_response = Self::call_method(
            &mut stdin,
            &mut stdout_reader,
            "initialize",
            json!({}),
            1,
        )
        .await?;

        // Step 2: Call emit_mermaid tool
        let mut arguments = json!({ "model": model });
        if let Some(s) = style {
            arguments["style"] = json!(s);
        }

        let tool_response = Self::call_method(
            &mut stdin,
            &mut stdout_reader,
            "tools/call",
            json!({
                "name": "emit_mermaid",
                "arguments": arguments
            }),
            2,
        )
        .await?;

        // Extract mermaid string from response
        let mermaid = tool_response
            .result
            .and_then(|r| r.get("mermaid").cloned())
            .and_then(|v| v.as_str().map(String::from))
            .context("Failed to extract 'mermaid' field from response")?;

        // Clean up
        drop(stdin);
        drop(stdout_reader);
        let _ = child.wait().await;

        Ok(mermaid)
    }

    /// Call the emit_markdown tool with a domain model
    pub async fn emit_markdown(
        &self,
        model: Value,
        audience: Option<&str>,
    ) -> Result<String> {
        let mut child = self.spawn_server().await?;

        let mut stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let mut stdout_reader = BufReader::new(stdout);

        // Initialize
        let _init_response = Self::call_method(
            &mut stdin,
            &mut stdout_reader,
            "initialize",
            json!({}),
            1,
        )
        .await?;

        // Call emit_markdown tool
        let mut arguments = json!({ "model": model });
        if let Some(aud) = audience {
            arguments["audience"] = json!(aud);
        }

        let tool_response = Self::call_method(
            &mut stdin,
            &mut stdout_reader,
            "tools/call",
            json!({
                "name": "emit_markdown",
                "arguments": arguments
            }),
            2,
        )
        .await?;

        let markdown = tool_response
            .result
            .and_then(|r| r.get("markdown").cloned())
            .and_then(|v| v.as_str().map(String::from))
            .context("Failed to extract 'markdown' field from response")?;

        drop(stdin);
        drop(stdout_reader);
        let _ = child.wait().await;

        Ok(markdown)
    }

    /// Call the normalize_terms tool with a transcript
    pub async fn normalize_terms(
        &self,
        input_lang: &str,
        transcript: &str,
    ) -> Result<Value> {
        let mut child = self.spawn_server().await?;

        let mut stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let mut stdout_reader = BufReader::new(stdout);

        // Initialize
        let _init_response = Self::call_method(
            &mut stdin,
            &mut stdout_reader,
            "initialize",
            json!({}),
            1,
        )
        .await?;

        // Call normalize_terms tool
        let tool_response = Self::call_method(
            &mut stdin,
            &mut stdout_reader,
            "tools/call",
            json!({
                "name": "normalize_terms",
                "arguments": {
                    "input_lang": input_lang,
                    "transcript": transcript
                }
            }),
            2,
        )
        .await?;

        let result = tool_response
            .result
            .context("Failed to get result from normalize_terms")?;

        drop(stdin);
        drop(stdout_reader);
        let _ = child.wait().await;

        Ok(result)
    }

    /// Call the validate_model tool
    pub async fn validate_model(
        &self,
        model: Value,
        schema_path: Option<&str>,
    ) -> Result<bool> {
        let mut child = self.spawn_server().await?;

        let mut stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let mut stdout_reader = BufReader::new(stdout);

        // Initialize
        let _init_response = Self::call_method(
            &mut stdin,
            &mut stdout_reader,
            "initialize",
            json!({}),
            1,
        )
        .await?;

        // Call validate_model tool
        let mut arguments = json!({ "model": model });
        if let Some(path) = schema_path {
            arguments["schema_path"] = json!(path);
        }

        let tool_response = Self::call_method(
            &mut stdin,
            &mut stdout_reader,
            "tools/call",
            json!({
                "name": "validate_model",
                "arguments": arguments
            }),
            2,
        )
        .await?;

        let is_valid = tool_response
            .result
            .and_then(|r| r.get("ok").cloned())
            .and_then(|v| v.as_bool())
            .context("Failed to extract 'ok' field from response")?;

        drop(stdin);
        drop(stdout_reader);
        let _ = child.wait().await;

        Ok(is_valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_emit_mermaid() -> Result<()> {
        let client = McpClient::new("../mcp/mcp-server/target/release/mcp-server".to_string());

        let model = json!({
            "entities": [
                {
                    "id": "User",
                    "name": "User",
                    "attributes": [
                        {"name": "id", "type": "uuid", "required": true},
                        {"name": "email", "type": "email", "required": true}
                    ]
                }
            ],
            "relations": [],
            "invariants": []
        });

        let mermaid = client.emit_mermaid(model, Some("er")).await?;
        assert!(mermaid.contains("erDiagram"));
        assert!(mermaid.contains("User"));

        Ok(())
    }

    #[tokio::test]
    async fn test_emit_mermaid_class_style() -> Result<()> {
        let client = McpClient::new("../mcp/mcp-server/target/release/mcp-server".to_string());

        let model = json!({
            "entities": [
                {
                    "id": "User",
                    "name": "User",
                    "attributes": [
                        {"name": "id", "type": "uuid", "required": true}
                    ]
                }
            ],
            "relations": [],
            "invariants": []
        });

        let mermaid = client.emit_mermaid(model, Some("class")).await?;
        assert!(mermaid.contains("classDiagram"));
        assert!(mermaid.contains("class User"));

        Ok(())
    }
}
