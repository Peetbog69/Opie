# Opie

Local-first AI assistant built in Rust. Zero-cost inference for most tasks, with optional API fallback.

## Philosophy

- **Local by default** - Run models on your hardware, no API costs
- **Clean & minimal** - Small codebase, easy to understand and modify
- **Cost-conscious** - Track spend, use API only when needed
- **Fast iteration** - Build features incrementally

## Status

**Phase 1: Core Loop** (Day 1 - Complete)
- [x] Project structure
- [x] Config system
- [x] Local inference (via llama-server HTTP)
- [x] Working chat loop skeleton
- [ ] Test with a real model

## Quick Start

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Get a model and start llama-server

**Option A: llama.cpp**
```bash
# Download llama.cpp
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
make

# Download a model
mkdir -p ~/models
cd ~/models
wget https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf

# Start server
./llama-server -m ~/models/Llama-3.2-3B-Instruct-Q4_K_M.gguf -c 8192
```

**Option B: Ollama (easier)**
```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model and run as server
ollama serve  # In one terminal
ollama pull llama3.2  # In another
```

### 3. Run Opie
```bash
cd ~/projects/opie
cargo run -- init  # Creates config
cargo run -- chat  # Start chatting
```

## Configuration

`~/.opie/config.toml`:
```toml
server_url = "http://localhost:8080"  # llama-server endpoint
api_mode = "never"                     # "never", "fallback", "auto", "always"
anthropic_api_key = ""                 # optional, for API fallback

[history_dir]
# Conversation history saved here
```

## Architecture

Clean, simple layers:

```
src/
├── config.rs           # TOML config loading (120 lines)
├── session.rs          # Conversation state (80 lines)
├── inference/
│   ├── mod.rs          # Provider trait (20 lines)
│   └── local.rs        # llama-server HTTP client (90 lines)
└── main.rs             # CLI entry point (110 lines)
```

**Total: ~420 lines of Rust vs. 50K+ lines in Hermes**

## Roadmap

- [x] **Phase 1**: Core chat loop with local inference (Day 1)
- [ ] **Phase 2**: Basic tool system (read/write files, run commands) (Week 1)
- [ ] **Phase 3**: Smart API routing (complexity detection, fallback) (Week 2)
- [ ] **Phase 4**: Memory & skills (lightweight versions of Hermes features) (Week 3+)

## Design Decisions

**Why llama-server over direct llama.cpp bindings?**
- Avoids complex C++ build dependencies
- Easier to swap backends (Ollama, vLLM, anything with HTTP)
- Cleaner Rust code
- Can still use full llama.cpp features (just run the server)

**Why HTTP over native libs?**
- Works with any inference backend
- Simple testing and debugging
- Clean separation of concerns

## Development

```bash
cargo check      # Fast compile check
cargo build      # Build debug binary
cargo run        # Run with default args
cargo build --release  # Optimized build
```

## License

MIT
