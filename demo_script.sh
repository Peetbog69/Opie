#!/bin/bash
# Demo script for Opie - automated terminal session

echo "=== Opie Demo ==="
echo ""
sleep 1

echo "$ opie chat"
sleep 1
echo ""
echo "Loading configuration..."
sleep 0.5
echo "Connecting to llama-server at http://localhost:8080..."
sleep 0.5
echo "Connected! (provider: local)"
echo ""
sleep 0.5

echo "Type 'quit', 'exit', or '/save <name>' to save and exit."
echo ""
sleep 1

# Demo 1: Read a file
echo "You: What's in src/main.rs?"
sleep 1
echo "Opie:   [read_file] Running..."
sleep 1.5
echo "This is the main entry point for Opie. It uses clap for CLI argument"
sleep 0.3
echo "parsing and provides subcommands for chat, init, config, list, save,"
sleep 0.3
echo "and delete. The chat command starts an interactive session with the"
sleep 0.3
echo "local AI model via llama-server."
echo ""
sleep 2

# Demo 2: Search code
echo "You: Find all TODO comments"
sleep 1
echo "Opie:   [search_files] Running..."
sleep 1.5
echo "Found 2 TODO comments:"
echo ""
sleep 0.3
echo "src/agent.rs:67: // TODO: Add retry logic for failed tool calls"
sleep 0.3
echo "src/tools/patch.rs:45: // TODO: Support fuzzy matching"
echo ""
sleep 2

# Demo 3: Terminal command
echo "You: How many lines of Rust code do we have?"
sleep 1
echo "Opie:   [terminal] Running..."
sleep 1.5
echo "You have 1,847 lines of Rust code across 12 files."
echo ""
sleep 2

# Demo 4: Save session
echo "You: /save demo"
sleep 1
echo "✓ Session saved as 'demo'"
sleep 1
echo ""

echo "=== Demo Complete ==="
echo ""
echo "Features shown:"
echo "  ✓ Reading files"
echo "  ✓ Searching code"
echo "  ✓ Running commands"
echo "  ✓ Session persistence"
