#!/bin/bash
# Test script for MCP server JSON-RPC protocol

SERVER="./target/release/mcp-server"

# Build if needed
if [ ! -f "$SERVER" ]; then
    echo "Building mcp-server..."
    cargo build --release --bin mcp-server
fi

echo "=== MCP Server Test ==="
echo ""

case "${1:-list}" in
    list|tools)
        echo "📋 Listing available tools..."
        echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | $SERVER | jq '.result.tools[] | {name, description}'
        ;;
    
    init|initialize)
        echo "🔧 Initializing server..."
        echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | $SERVER | jq
        ;;
    
    normalize)
        echo "🔄 Testing normalize_terms..."
        cat << 'EOF' | $SERVER | jq
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "normalize_terms",
    "arguments": {
      "input_lang": "fr",
      "transcript": "Un système simple avec un User qui a un email unique et un nom."
    }
  }
}
EOF
        ;;
    
    *)
        echo "Usage: $0 [list|init|normalize]"
        echo ""
        echo "Commands:"
        echo "  list       - List available tools (default)"
        echo "  init       - Initialize the server"
        echo "  normalize  - Test normalize_terms with example"
        exit 1
        ;;
esac
