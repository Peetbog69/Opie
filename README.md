# Opie 🤖

**Local-first AI coding assistant with zero token costs.**

Opie is a Rust-based AI agent that runs entirely on your machine using local LLMs. No API keys, no usage limits, no privacy concerns—just a fast, capable coding assistant powered by open-source models.

![Opie Demo](demo.gif)

## Features

✨ **100% Local** - All inference runs on your hardware  
🚀 **Streaming Responses** - Real-time word-by-word output  
🛠️ **5 Powerful Tools** - Read/write files, search code, patch files, run commands  
💾 **Session Persistence** - Save and resume conversations  
🧠 **Smart Context Management** - Auto-trims to stay within token limits  
⚡ **Fast** - Optimized for 3B-7B models on consumer hardware  

## Quick Start

### Prerequisites

- **Rust** (1.75+): Install from [rustup.rs](https://rustup.rs)
- **llama-server** or **Ollama** for local inference

### 1. Install a Local LLM Server

**Option A: llama.cpp (Recommended)**

```bash
# Clone and build llama.cpp
git clone https://github.com/ggml-org/llama.cpp
cd llama.cpp
mkdir build && cd build
cmake .. -DGGML_CUDA=ON  # Remove -DGGML_CUDA=ON if no GPU
cmake --build . --config Release

# Download a model (Qwen 2.5 3B Instruct - 1.9GB)
mkdir -p ../models && cd ../models
wget https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf

# Start the server (keep this running)
cd ../build/bin
./llama-server -m ../../models/qwen2.5-3b-instruct-q4_k_m.gguf -c 8192 --port 8080
```

**Option B: Ollama (Easier, slightly slower)**

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull qwen2.5:3b

# Configure Opie to use Ollama (edit ~/.opie/config.toml):
# server_url = "http://localhost:11434"
```

### 2. Install Opie

```bash
git clone https://github.com/Peetbog69/Opie.git
cd Opie
cargo build --release

# Optional: Install globally
sudo cp target/release/opie /usr/local/bin/
```

### 3. Initialize Configuration

```bash
opie init
# Creates ~/.opie/config.toml with default settings
```

### 4. Start Chatting!

```bash
opie chat
```

## Usage

### Basic Commands

```bash
# Start a new chat session
opie chat

# Resume a saved session
opie chat --load my-project

# List all saved sessions
opie list

# Delete a session
opie delete old-session

# Show current configuration
opie config
```

### In-Chat Commands

- `/save <name>` - Save current session and exit
- `quit` or `exit` - Exit without saving

### Example Conversations

```
You: What files are in the src directory?
Opie: [runs terminal: ls src/]
      The src directory contains: main.rs, lib.rs, agent.rs, ...

You: Read src/main.rs and explain what it does
Opie: [reads file]
      This is the main entry point. It uses clap for CLI parsing...

You: Find all TODO comments in the codebase
Opie: [searches files]
      Found 3 TODO comments:
      - src/agent.rs:45: TODO: Add error recovery
      - src/tools/mod.rs:12: TODO: Implement rate limiting
      ...

You: Change the port from 8080 to 3000 in config.toml
Opie: [searches for port, then patches]
      ✓ Updated port to 3000 in config.toml

You: Run the tests
Opie: [executes: cargo test]
      Running 12 tests...
      test result: ok. 12 passed; 0 failed
```

## Available Tools

Opie has access to 5 built-in tools:

1. **read_file** - Read file contents with line numbers
2. **write_file** - Create or overwrite files
3. **terminal** - Execute shell commands
4. **search_files** - Search for patterns in files (grep-based)
5. **patch** - Make targeted find-and-replace edits

Tools are invoked automatically when needed—you just chat naturally.

## Recommended Models

Models are listed by size (smaller = faster, larger = smarter):

| Model | Size | Quality | Speed | Best For |
|-------|------|---------|-------|----------|
| Qwen 2.5 3B Instruct | 1.9GB | Good | Fast | Daily coding tasks |
| Llama 3.2 3B Instruct | 2.0GB | Good | Fast | General use |
| Qwen 2.5 7B Instruct | 4.4GB | Better | Medium | Complex reasoning |
| DeepSeek Coder 6.7B | 3.8GB | Great | Medium | Code-heavy work |
| Llama 3.1 8B Instruct | 4.9GB | Best | Slow | Maximum quality |

Download GGUF models from [Hugging Face](https://huggingface.co/models?library=gguf&sort=trending).

### Model Selection Tips

- **3B models**: Great for quick queries, code search, file operations
- **7B+ models**: Better at understanding complex requirements and generating code
- Use Q4_K_M quantization for best quality/size balance
- Ensure your model has "Instruct" or "Chat" in the name

## Configuration

Config file: `~/.opie/config.toml`

```toml
# llama-server endpoint
server_url = "http://localhost:8080"

# For Ollama, use:
# server_url = "http://localhost:11434"
```

Sessions are saved to `~/.opie/sessions/` as JSON files.

## Architecture

Opie is built with:

- **Rust** - Fast, safe, and reliable
- **llama.cpp** - Efficient local inference
- **Tokio** - Async runtime
- **SSE Streaming** - Real-time response delivery

### Project Structure

```
src/
├── agent.rs        # Tool loop and streaming logic
├── inference/      # LLM inference (local.rs for llama-server)
├── session.rs      # Context management and trimming
├── storage.rs      # Session persistence (save/load)
├── tools/          # Tool implementations
│   ├── read_file.rs
│   ├── write_file.rs
│   ├── terminal.rs
│   ├── search_files.rs
│   └── patch.rs
└── main.rs         # CLI interface
```

## Development Phases

Opie was built in 7 phases:

1. ✅ Basic chat with local LLM
2. ✅ Tool system (read, write, terminal)
3. ✅ Streaming responses
4. ✅ Code tools (search, patch)
5. ✅ Enhanced system prompt
6. ✅ Context management with auto-trimming
7. ✅ Session persistence

## Performance

On typical hardware (8GB RAM, modern CPU):

- **3B models**: ~30-50 tokens/sec
- **7B models**: ~10-20 tokens/sec
- **Context window**: 8192 tokens (configurable)
- **Memory usage**: ~4-6GB for 3B models

GPU acceleration via CUDA/Metal significantly improves speed.

## Troubleshooting

### llama-server not responding

```bash
# Check if it's running
curl http://localhost:8080/health

# Restart it with verbose logging
./llama-server -m model.gguf -c 8192 --port 8080 -v
```

### Context limit errors

The default limit is 6000 tokens. Long conversations auto-trim older messages. To adjust:

Edit `src/session.rs` and change:
```rust
max_context: 6000,  // Increase this value
```

### Model quality issues

Try a larger model (7B+) or adjust temperature in `src/inference/local.rs`:
```rust
temperature: Some(0.3),  // Lower = more focused, higher = more creative
```

## Roadmap

Potential future features:

- [ ] Multi-model support (switch models on the fly)
- [ ] Web search integration
- [ ] Git workflow tools (commit, diff, PR creation)
- [ ] Multi-file context awareness
- [ ] REPL improvements (history, tab completion)
- [ ] Anthropic/OpenAI API fallback for complex tasks

## Contributing

Contributions welcome! Areas that need work:

- Additional tools (web search, API calls, etc.)
- Better error handling and recovery
- Performance optimizations
- Documentation improvements
- Model compatibility testing

## License

MIT License - see LICENSE file for details.

## Acknowledgments

Built on the shoulders of giants:

- [llama.cpp](https://github.com/ggml-org/llama.cpp) - Local inference engine
- [Qwen](https://github.com/QwenLM/Qwen) - Excellent open-source models
- Inspired by [Hermes](https://github.com/raphaelsty/hermes) and Claude Code

---

**Made with ❤️ for the local-first AI community**

Star the repo if Opie helps you code faster! ⭐
