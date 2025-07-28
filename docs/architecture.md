# Phloem System Architecture

## Overview
Phloem is a secure, fast command-line assistant that translates natural language into executable shell commands using local AI models via Ollama.

## Core Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Rust CLI      │    │     Ollama       │    │  Local Context │
│   (phloem)      │◄──►│   HTTP API       │    │  (~/.phloem)    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
        │                        │                       │
        ▼                        ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ CLI Interface   │    │  AI Models       │    │   PHLOEM.md     │
│ Cache System    │    │  (Gemma, Llama)  │    │   SQLite Cache  │
│ Context Manager │    │  Local Inference │    │   Shell History │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Component Breakdown

### 1. Rust CLI Core (`src/`)
**Purpose**: Fast, secure command-line interface with intelligent caching
- **CLI Interface** (`src/cli/`): Argument parsing, interactive suggestions, output formatting
- **AI Integration** (`src/ai/`): HTTP client for Ollama API communication
- **Context Management** (`src/context/`): Smart caching, learning, shell history integration
- **Configuration** (`src/config/`): User settings and defaults
- **Utilities** (`src/utils/`): Environment detection, command validation

### 2. Ollama Integration
**Purpose**: Local AI model inference via HTTP API
- **Model Management**: Ollama handles model downloading, loading, and lifecycle
- **HTTP API**: Simple REST interface for inference requests
- **Model Flexibility**: Support for various models (Gemma, Llama, CodeLlama, etc.)
- **Local Inference**: All processing happens on the user's machine

### 3. Local Context System (`~/.phloem/`)
**Purpose**: Maintain user-specific command context and learning
- **PHLOEM.md**: Evolving knowledge base with learned patterns
- **SQLite Cache**: Smart caching with success rate tracking (>70% success, 5+ uses)
- **Shell History**: Integration with bash/zsh history for context
- **Environment Detection**: OS, shell, installed tools awareness

## Data Flow

1. **User Input**: `phloem "list running containers"`
2. **Cache Check**: Check SQLite cache for existing suggestions with high success rate
3. **Context Loading**: Read ~/.phloem/PHLOEM.md and recent shell history
4. **Ollama Request**: HTTP POST to `http://localhost:11434/api/generate` with prompt + context
5. **AI Inference**: Ollama processes request using loaded model (e.g., gemma3n:e2b)
6. **Response Processing**: Parse and validate AI-generated commands
7. **Interactive Display**: Show suggestions with keyboard navigation
8. **Execution & Learning**: Track command success and update cache/context

## Performance Characteristics

- **Cold Start**: < 3 seconds (first request to Ollama)
- **Warm Response**: < 1 second (Ollama model loaded)
- **Cached Response**: < 100ms (SQLite lookup)
- **Memory Usage**: ~50MB (Rust binary), models managed by Ollama
- **Binary Size**: ~5MB (optimized Rust executable)

## Security & Privacy

- **Local-First**: No external API calls, all processing via Ollama
- **Command Validation**: Built-in filtering of dangerous operations
- **No Telemetry**: Zero data collection or external communication
- **Context Privacy**: All learning data stays in ~/.phloem/
- **Safe Execution**: Commands are suggested, not automatically executed

## Smart Caching Strategy

### Cache Criteria
Only cache suggestions that prove reliable:
```sql
-- Cache entry requirements
use_count >= 5 AND success_rate > 0.7 AND last_used > datetime('now', '-7 days')
```

### Learning Process
1. **Initial Use**: AI generates fresh suggestions
2. **Usage Tracking**: Record success/failure of executed commands
3. **Pattern Recognition**: Identify consistently successful command patterns
4. **Cache Promotion**: Move high-success patterns to instant cache
5. **Context Update**: Add successful patterns to PHLOEM.md

## File Structure

```
~/.phloem/
├── PHLOEM.md              # Knowledge base with learned patterns
├── config.toml            # User configuration
├── cache/
│   └── suggestions.db     # SQLite cache with success tracking
└── backups/               # PHLOEM.md backups

src/
├── main.rs               # Entry point
├── lib.rs                # Library exports
├── cli/                  # Command-line interface
│   ├── args.rs          # Argument parsing (clap)
│   ├── commands.rs      # Command handling
│   └── output.rs        # Interactive suggestion display
├── ai/                   # Ollama integration
│   ├── ollama_client.rs # HTTP client for Ollama API
│   ├── prompt.rs        # Prompt engineering
│   └── response.rs      # Response parsing
├── context/              # Context management
│   ├── cache.rs         # SQLite cache operations
│   ├── manager.rs       # Context coordination
│   └── storage.rs       # File system operations
├── config/               # Configuration
│   ├── settings.rs      # User settings
│   └── defaults.rs      # Default values
└── utils/                # Utilities
    ├── environment.rs   # OS/shell detection
    ├── shell.rs         # Shell integration
    └── validation.rs    # Command validation
```

## Extensibility

### Configuration
- **Model Selection**: Configure which Ollama model to use
- **Cache Behavior**: Adjust caching thresholds and retention
- **Prompt Engineering**: Customize system prompts and context

### Shell Integration
- **Completion Scripts**: Generated for bash, zsh, fish
- **History Integration**: Automatic shell history analysis
- **Aliases**: Can be integrated with shell aliases

### Model Support
- **Multiple Models**: Easy switching between Ollama-supported models
- **Custom Models**: Support for user-fine-tuned models via Ollama
- **Model Updates**: Automatic model updates through Ollama

## Why This Architecture?

### Rust Core
- **Performance**: Exceptional speed for CLI operations and caching
- **Safety**: Memory safety without garbage collection overhead
- **Binary Distribution**: Single executable, easy installation

### Ollama Integration
- **Simplicity**: HTTP API eliminates complex subprocess management
- **Model Management**: Ollama handles all model lifecycle complexity
- **Flexibility**: Easy model switching and updates
- **Community**: Large ecosystem of available models

### Local-First Design
- **Privacy**: All data stays on user's machine
- **Speed**: Local caching provides instant responses
- **Reliability**: Works offline once models are downloaded
- **Personalization**: Learns user-specific patterns and preferences