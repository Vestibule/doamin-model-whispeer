# LLM Router Implementation

This module implements a router that directs LLM requests to different providers based on environment configuration. The LLM never communicates directly with the UI - it only emits structured tool calls.

## Architecture

```
User Request → LlmIntegration → LlmRouter → [Ollama | External Provider]
                    ↓                               ↓
              Tool Calls ← Parse JSON Response ←  JSON
                    ↓
              McpClient → Execute Tools → Return Results
```

## Components

### 1. `llm_router.rs`
The core router that handles provider detection and API communication.

**Provider Detection:**
- If `LLM_PROVIDER=ollama`: Use local Ollama instance
- Otherwise: Use external provider (OpenAI, Anthropic, etc.)

**Configuration:**

You can configure using either environment variables or a `.env` file.

**Option 1: Using `.env` file (recommended)**

Create a `.env` file in `src-tauri/` directory:

```bash
# For Ollama
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434  # Optional
OLLAMA_MODEL=llama2                      # Optional
```

```bash
# For External Provider (e.g., OpenAI)
LLM_PROVIDER=openai
LLM_API_KEY=your_api_key
LLM_ENDPOINT=https://api.openai.com/v1/chat/completions
```

See `.env.example` for a complete template.

**Option 2: Using shell environment variables**

```bash
export LLM_PROVIDER=ollama
export OLLAMA_BASE_URL=http://localhost:11434
export OLLAMA_MODEL=llama2
```

```bash
export LLM_PROVIDER=openai
export LLM_API_KEY=your_api_key
export LLM_ENDPOINT=https://api.openai.com/v1/chat/completions
```

### 2. `llm_integration.rs`
Integration layer that coordinates between the LLM router and MCP tools.

**Key Features:**
- Sends system prompt defining available tools and schemas
- Parses LLM responses to extract tool calls
- Executes tool calls via MCP client
- Returns structured results (never plain text to UI)

### 3. DomainModel Output Format

The LLM is constrained by the system prompt to return ONLY a valid DomainModel JSON:

```json
{
  "entities": [
    {
      "id": "User",
      "name": "User",
      "attributes": [
        {"name": "email", "type": "email", "required": true},
        {"name": "password", "type": "string", "required": true}
      ]
    }
  ],
  "relations": [],
  "invariants": []
}
```

**The system prompt enforces:**
- Strict schema compliance
- No additional fields allowed
- All required fields must be present
- Enum values must match exactly
- Regex patterns must be respected

## API Endpoints

### Ollama
- **URL:** `POST {OLLAMA_BASE_URL}/api/generate`
- **Request Body:**
  ```json
  {
    "model": "llama2",
    "prompt": "<system_prompt>\n\nUser: <user_prompt>",
    "stream": false,
    "format": "json"
  }
  ```
- **Response:**
  ```json
  {
    "response": "{\"tool_calls\": [...]}",
    "done": true
  }
  ```

### External Provider
- **URL:** `POST {LLM_ENDPOINT}`
- **Headers:**
  - `Authorization: Bearer {LLM_API_KEY}`
  - `Content-Type: application/json`
- **Request Body:**
  ```json
  {
    "messages": [
      {"role": "system", "content": "<system_prompt>"},
      {"role": "user", "content": "<user_prompt>"}
    ],
    "temperature": 0.7,
    "response_format": {"type": "json_object"}
  }
  ```

## Usage Example

```rust
use crate::llm_integration::LlmIntegration;

#[tokio::main]
async fn main() -> Result<()> {
    // Create integration with MCP server binary path
    let integration = LlmIntegration::new(
        "./mcp/mcp-server/target/release/mcp-server".to_string()
    )?;

    // Process user request
    let result = integration
        .process_request("Extract entities from: User has email and password")
        .await?;

    println!("Results: {}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

## Available Tools

The system prompt defines these tools for the LLM:

1. **normalize_terms**
   - Extract domain model from natural language transcript
   - Parameters: `input_lang` (string), `transcript` (string)

2. **emit_markdown**
   - Generate Markdown documentation of a domain model
   - Parameters: `model` (object), `audience` (optional: "technical" or "business")

3. **emit_mermaid**
   - Generate Mermaid diagram of a domain model
   - Parameters: `model` (object), `style` (optional: "er" or "class")

4. **validate_model**
   - Validate a domain model for consistency
   - Parameters: `model` (object), `schema_path` (optional string)

## Testing

Run unit tests:
```bash
cd src-tauri
cargo test --lib llm_router
cargo test --lib llm_integration
```

Run integration tests (requires environment variables):
```bash
export LLM_PROVIDER=ollama
cargo test --lib llm_integration -- --ignored
```

## Dependencies

Added to `Cargo.toml`:
```toml
reqwest = { version = "0.11", features = ["json"] }
dotenvy = "0.15"  # For .env file support
```

## Security Note

**IMPORTANT:** Never commit your `.env` file to version control!

Add to `.gitignore`:
```
.env
```

Use `.env.example` as a template for other developers.

## Design Principles

1. **No Direct UI Communication:** The LLM never generates text for the user. It only emits structured tool calls.

2. **Provider Agnostic:** Easy to switch between Ollama and external providers via environment variables.

3. **Structured Output:** All LLM responses are parsed into `LlmResponse` with typed `ToolCall` structures.

4. **Error Handling:** Comprehensive error context at each layer for debugging.

5. **Testable:** Clear separation of concerns allows unit testing of each component.
