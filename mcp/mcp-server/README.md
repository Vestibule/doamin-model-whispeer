# MCP Domain Model Server

A Model Context Protocol (MCP) server that provides tools for working with domain models.

## Building

```bash
cargo build --release
```

Binary output: `target/release/mcp-server`

## Protocol

The server implements JSON-RPC 2.0 over stdin/stdout and follows the MCP specification.

## Tools

### 1. `normalize_terms`

Extract domain model structure from natural language transcript.

**Signature:**
```typescript
normalize_terms(input_lang: string, transcript: string) -> {
  entities: Array<{
    name: string,
    attrs: Array<{
      name: string,
      type: string,
      pk?: boolean,
      unique?: boolean
    }>
  }>,
  relations: Array<{
    from: string,
    to: string,
    name: string,
    cardinality: string
  }>,
  invariants: string[]
}
```

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "normalize_terms",
    "arguments": {
      "input_lang": "en",
      "transcript": "We have an entity User with attributes id and email."
    }
  }
}
```

### 2. `emit_markdown`

Generate Markdown documentation from a domain model.

**Signature:**
```typescript
emit_markdown(model: DomainModel, audience?: string) -> {
  markdown: string
}
```

**Parameters:**
- `model`: Domain model structure (entities, relations, invariants)
- `audience`: Optional, one of `"technical"` or `"business"`

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "emit_markdown",
    "arguments": {
      "model": {
        "entities": [...],
        "relations": [],
        "invariants": []
      },
      "audience": "technical"
    }
  }
}
```

### 3. `emit_mermaid`

Generate Mermaid diagram from a domain model.

**Signature:**
```typescript
emit_mermaid(model: DomainModel, style?: 'er' | 'class') -> {
  mermaid: string
}
```

**Parameters:**
- `model`: Domain model structure
- `style`: Diagram style, either `"er"` (Entity-Relationship, default) or `"class"` (Class Diagram)

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "emit_mermaid",
    "arguments": {
      "model": {
        "entities": [...],
        "relations": [...],
        "invariants": []
      },
      "style": "class"
    }
  }
}
```

### 4. `validate_model`

Validate a domain model for consistency and correctness.

**Signature:**
```typescript
validate_model(model: DomainModel, schema_path?: string) -> {
  ok: boolean,
  errors?: string[]
}
```

**Parameters:**
- `model`: Domain model to validate
- `schema_path`: Optional path to JSON schema file for additional validation

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "validate_model",
    "arguments": {
      "model": {
        "entities": [...],
        "relations": [],
        "invariants": []
      },
      "schema_path": "../domain_model.schema.json"
    }
  }
}
```

## Domain Model Structure

```typescript
interface DomainModel {
  entities: Entity[];
  relations: Relation[];
  invariants: Invariant[];
}

interface Entity {
  id: string;
  name: string;
  description?: string;
  attributes: Attribute[];
  primaryKey?: string[];
}

interface Attribute {
  name: string;
  type: string;
  description?: string;
  required?: boolean;
  unique?: boolean;
}

interface Relation {
  id: string;
  name: string;
  description?: string;
  from: {
    entityId: string;
    label?: string;
  };
  to: {
    entityId: string;
    label?: string;
  };
  cardinality: {
    from: string; // "0..1" | "1" | "0..n" | "1..n" | "*"
    to: string;
  };
}

interface Invariant {
  id: string;
  name: string;
  description?: string;
  type: string;
  expression: string;
  severity?: string;
}
```

## Testing

Run the test script:

```bash
./test_new_signatures.sh
```

## MCP Integration

To use this server with an MCP client, configure it to launch:

```json
{
  "mcpServers": {
    "domain-model": {
      "command": "/path/to/mcp-server",
      "args": []
    }
  }
}
```
