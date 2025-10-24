# Orchestrate Command

## Overview

The `orchestrate` Tauri command processes a transcript and returns a complete domain model with markdown documentation and a Mermaid diagram.

## Usage

### From TypeScript/JavaScript

```typescript
import { orchestrate } from './lib/tauri';

const result = await orchestrate("User entity has email and password attributes");

console.log(result.markdown);  // Markdown documentation
console.log(result.mermaid);   // Mermaid diagram code
console.log(result.model);     // Domain model JSON
```

### From Svelte

```svelte
<script lang="ts">
  import { orchestrate, type OrchestrateResult } from './lib/tauri';

  let transcript = "";
  let result: OrchestrateResult | null = null;

  async function handleOrchestrate() {
    result = await orchestrate(transcript);
  }
</script>

<textarea bind:value={transcript} />
<button on:click={handleOrchestrate}>Process</button>

{#if result}
  <pre>{result.markdown}</pre>
  <pre>{result.mermaid}</pre>
  <pre>{JSON.stringify(result.model, null, 2)}</pre>
{/if}
```

## Flow

1. **LLM Processing** (`llm_integration.rs`)
   - Takes the transcript as input
   - Uses configured LLM (Ollama or External) to generate a domain model
   - Returns structured JSON following the DomainModel schema

2. **Mermaid Generation** (`mcp_client.rs::emit_mermaid`)
   - Takes the domain model
   - Calls MCP server's `emit_mermaid` tool
   - Returns Mermaid diagram code (Entity-Relationship style by default)

3. **Markdown Generation** (`mcp_client.rs::emit_markdown`)
   - Takes the domain model
   - Calls MCP server's `emit_markdown` tool
   - Returns formatted markdown documentation

## Environment Variables

### Required

- `LLM_API_KEY` - API key for external LLM provider (if not using Ollama)
- `LLM_ENDPOINT` - Endpoint URL for external LLM provider (if not using Ollama)

### Optional

- `LLM_PROVIDER` - Set to `"ollama"` to use local Ollama, defaults to `"external"`
- `OLLAMA_BASE_URL` - Ollama server URL (default: `http://localhost:11434`)
- `OLLAMA_MODEL` - Ollama model to use (default: `llama2`)
- `MCP_SERVER_PATH` - Path to MCP server binary (default: `../mcp/mcp-server/target/release/mcp-server`)

## Example .env File

```bash
# Using Ollama (local)
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=llama3

# Using external provider
LLM_PROVIDER=external
LLM_API_KEY=your_api_key_here
LLM_ENDPOINT=https://api.openai.com/v1/chat/completions

# MCP Server
MCP_SERVER_PATH=../mcp/mcp-server/target/release/mcp-server
```

## Response Structure

```typescript
interface OrchestrateResult {
  markdown: string;      // Markdown documentation
  mermaid: string;       // Mermaid diagram code
  model: DomainModel;    // Full domain model
}

interface DomainModel {
  entities: Entity[];
  relations: Relation[];
  invariants: Invariant[];
}
```

## Error Handling

The command returns a `Result<OrchestrateResult, String>`. Errors are returned as formatted strings containing:

- LLM initialization errors
- Domain model generation errors
- MCP server communication errors
- Mermaid/Markdown generation errors

## Dependencies

- `llm_integration.rs` - LLM integration layer
- `llm_router.rs` - LLM provider abstraction
- `mcp_client.rs` - MCP server client
- MCP server binary (must be built and accessible)
