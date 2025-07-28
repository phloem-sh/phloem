# Phloem Documentation

## Overview
Phloem is a secure, fast command-line assistant that translates natural language into executable shell commands using local AI models via Ollama.

## 🚀 Features (v0.1.0)
- **🧠 Smart Caching**: Only caches commands after 5+ successful uses with >70% success rate
- **⚡ Command Validation**: Real-time executable validation using `which` and PATH scanning
- **🔄 Progressive Learning**: Learns from shell history and successful patterns stored in PHLOEM.md
- **🎯 Interactive Interface**: Navigate suggestions with keyboard shortcuts
- **📊 Cache Analytics**: View cache statistics and success rates
- **🔒 Privacy-First**: All processing happens locally via Ollama

## Supported Platforms
- **macOS**: Intel (x86_64) and Apple Silicon (ARM64)
- **Linux**: x86_64 (Debian, Ubuntu, and other distributions)

## Example Usage
```bash
# Natural language to commands
phloem "list running containers"
phloem "find files larger than 100MB"
phloem "git commit with message hello world"

# Interactive interface controls
# Enter → Execute command immediately
# Tab → Copy to clipboard  
# Escape → Modify/follow-up on command
# Escape Escape → Exit to static view

# With explanations
phloem --explain "compress this directory"

# Validates real executables
phloem "docker logs for container"
# ✅ Suggests actual docker commands
# ❌ Rejects invalid/non-existent commands
```

## Documentation Index

### Architecture & Design
- **[Architecture](./architecture.md)** - System architecture and component overview
- **[Context Management](./context-management.md)** - ~/.phloem folder structure and learning system
- **[Rust Implementation](./rust-implementation.md)** - Rust core structure and modules

## Installation

### For Users
```bash
# Quick install (recommended)
curl -fsSL https://raw.githubusercontent.com/phloem-sh/phloem/main/install.sh | sh

# Initialize Phloem
phloem init
```

### For Developers
```bash
# Clone the repository
git clone git@github.com:phloem-sh/phloem.git
cd phloem

# Build and install
cargo build --release
cargo install --path .

# Initialize for development
phloem init
```

## Architecture

### Core Components
- **Rust CLI**: Fast command parsing, context management, and user interface
- **Ollama Integration**: Local AI model inference via HTTP API
- **SQLite Cache**: Intelligent caching with success rate tracking
- **Context Learning**: Progressive improvement through usage patterns

### Privacy & Security
- **Local-Only**: All processing happens on your machine via Ollama
- **No Telemetry**: No data collection or external API calls
- **Command Validation**: Built-in filtering and executable verification
- **Context Control**: Full control over learning data in ~/.phloem/

### Context Learning System
The `~/.phloem/PHLOEM.md` file evolves with your usage:
- **Environment Detection**: OS, shell, installed tools
- **Command Patterns**: Your preferred command styles and formats  
- **Success Tracking**: Learns from commands you actually run
- **Project Context**: Adapts to your current working environment

## Project Structure

```
~/.phloem/
├── PHLOEM.md              # Evolving knowledge base
├── config.toml            # Configuration settings
├── cache/
│   └── suggestions.db     # Smart cache with success tracking
└── backups/               # PHLOEM.md backups

src/
├── cli/                   # Command-line interface & interactions  
├── ai/                    # Ollama integration & prompt engineering
├── context/               # Caching, learning, shell history
├── config/                # Configuration management
└── utils/                 # Environment detection, validation
```

## Commands

```bash
phloem init                    # Initialize setup
phloem config                  # Show configuration & cache stats
phloem doctor                  # Run diagnostics  
phloem clear --cache          # Clear suggestion cache
phloem clear --context        # Reset learning context
phloem "your natural language query"
```

## Technical Decisions

### Why Rust?
- **Performance**: Exceptional speed for CLI operations
- **Safety**: Memory safety without garbage collection overhead
- **Ecosystem**: Rich crate ecosystem for CLI tools and system integration

### Why Ollama?
- **Local Inference**: No API dependencies, works completely offline
- **Model Flexibility**: Support for various models (Gemma, Llama, etc.)
- **Easy Setup**: Simple installation and model management
- **Privacy**: All processing stays on your machine

### Why SQLite Cache?
- **Speed**: Sub-100ms lookups for repeated queries
- **Reliability**: ACID compliance and crash resistance
- **Portability**: Single file, no external dependencies
- **Intelligence**: Track success rates and usage patterns

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines and detailed technical specifications.

---

**Secure, fast command suggestions using local models** 🚀