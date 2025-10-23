# MCP Server Usage

The MCP server is a JSON-RPC server that communicates via stdin/stdout, not CLI arguments.

## Building

```bash
cargo build --release --bin mcp-server
```

## Usage

The server expects JSON-RPC requests on stdin and returns responses on stdout.

### List Available Tools

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | ./target/release/mcp-server
```

Pretty-printed with Python:
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | \
  ./target/release/mcp-server | \
  python3 -m json.tool
```

### Initialize Server

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./target/release/mcp-server
```

### Call a Tool

Example: `normalize_terms`

```bash
cat << 'EOF' | ./target/release/mcp-server
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "normalize_terms",
    "arguments": {
      "input_lang": "fr",
      "transcript": "Un système avec un User qui a un email unique."
    }
  }
}
EOF
```

## Test Script

A convenience script is provided:

```bash
# List tools
./test_mcp.sh list

# Initialize server
./test_mcp.sh init

# Test normalize_terms
./test_mcp.sh normalize
```

## Available Tools

1. **generate_domain_model** - Generate complete DomainModel from natural language
2. **normalize_terms** - Extract domain model from transcript
3. **emit_markdown** - Generate Markdown documentation
4. **emit_mermaid** - Generate Mermaid ER or class diagrams
5. **validate_model** - Validate DomainModel consistency

## Integration with Warp/Claude

The MCP server is designed to be used with MCP clients like Warp AI or Claude Desktop.

Add to your MCP client configuration:

```json
{
  "mcpServers": {
    "domain-model": {
      "command": "/path/to/mcp-server/target/release/mcp-server",
      "args": []
    }
  }
}
```

## CLI Tool

For command-line usage without JSON-RPC, use the CLI tool instead:

```bash
cargo run --bin mcp-cli -- \
  --input samples/voice.json \
  --emit-md artifacts/spec.md \
  --emit-mmd artifacts/model.mmd \
  --retry 2 \
  --trace
```

See main README for CLI documentation.
