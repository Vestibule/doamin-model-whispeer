#!/bin/bash

# Test script for MCP server with new signatures
BINARY="./target/release/mcp-server"

echo "Testing MCP Server with new signatures..."
echo ""

# Test 1: normalize_terms
echo "Test 1: normalize_terms (extract from transcript)"
cat <<EOF | $BINARY
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"normalize_terms","arguments":{"input_lang":"en","transcript":"We have an entity User with attributes id and email. Another entity Order must reference User."}}}
EOF
echo ""
echo ""

# Test 2: emit_markdown with audience
echo "Test 2: emit_markdown (with technical audience)"
cat <<EOF | $BINARY
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"emit_markdown","arguments":{"model":{"entities":[{"id":"user","name":"User","attributes":[{"name":"id","type":"uuid","required":true},{"name":"email","type":"email","required":true}]}],"relations":[],"invariants":[]},"audience":"technical"}}}
EOF
echo ""
echo ""

# Test 3: emit_mermaid with class style
echo "Test 3: emit_mermaid (class diagram style)"
cat <<EOF | $BINARY
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"emit_mermaid","arguments":{"model":{"entities":[{"id":"User","name":"User","attributes":[{"name":"id","type":"uuid","required":true}]},{"id":"Order","name":"Order","attributes":[{"name":"orderId","type":"uuid","required":true}]}],"relations":[{"id":"user_orders","name":"has_orders","from":{"entityId":"User"},"to":{"entityId":"Order"},"cardinality":{"from":"1","to":"0..n"}}],"invariants":[]},"style":"class"}}}
EOF
echo ""
echo ""

# Test 4: validate_model with schema_path
echo "Test 4: validate_model (with schema path)"
cat <<EOF | $BINARY
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"validate_model","arguments":{"model":{"entities":[{"id":"User","name":"User","attributes":[{"name":"id","type":"uuid","required":true}],"primaryKey":["id"]}],"relations":[],"invariants":[]},"schema_path":"../domain_model.schema.json"}}}
EOF
echo ""

echo "Tests completed!"
