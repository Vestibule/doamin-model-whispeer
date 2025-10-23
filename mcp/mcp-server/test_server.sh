#!/bin/bash

# Test script for MCP server
BINARY="./target/release/mcp-server"

echo "Testing MCP Server..."
echo ""

# Test 1: Initialize
echo "Test 1: Initialize"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | $BINARY
echo ""

# Test 2: List tools
echo "Test 2: List tools"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | $BINARY
echo ""

# Test 3: Validate model
echo "Test 3: Validate model"
cat <<EOF | $BINARY
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"validate_model","arguments":{"model":{"entities":[{"id":"User","name":"User","attributes":[{"name":"id","type":"uuid","required":true},{"name":"email","type":"email","required":true}],"primaryKey":["id"]}],"relations":[],"invariants":[]}}}}
EOF
echo ""

# Test 4: Normalize terms
echo "Test 4: Normalize terms"
cat <<EOF | $BINARY
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"normalize_terms","arguments":{"model":{"entities":[{"id":"UserAccount","name":"user account","attributes":[{"name":"UserId","type":"uuid"},{"name":"EmailAddress","type":"email"}]}],"relations":[],"invariants":[]}}}}
EOF
echo ""

echo "Tests completed!"
