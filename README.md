# Phloem

> Secure, fast command suggestions using local models

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)

Translates natural language into shell commands using local AI models via Ollama.

## Quick Start

### Installation

**Supported Platforms:**
- macOS (Intel & Apple Silicon)
- Linux (x86_64) - Debian, Ubuntu, and other distributions

```bash
# Quick install (recommended)
curl -fsSL https://raw.githubusercontent.com/phloem-sh/phloem/main/install.sh | sh

# OR download from releases
# https://github.com/phloem-sh/phloem/releases

# Initialize Phloem
phloem init
```

### Basic Usage
```bash
# Natural language to commands
phloem "list running containers"
phloem "find files larger than 100MB"
phloem "git commit with message hello world"

# With explanations
phloem --explain "compress this directory"

# Validates real executables
phloem "memgraph query to get all nodes"
# ✅ Suggests: cypher-shell -a bolt://localhost:7687 "MATCH (n) RETURN n"
# ❌ Not: memgraph query "..." (checks with 'which' first)
```

### Interactive Controls
- **Enter** → Execute command immediately
- **Tab** → Copy to clipboard  
- **Escape** → Modify/follow-up on command
- **Escape Escape** → Exit to static view
- **F** → Alternative follow-up key

## How It Works

### Caching
```bash
# First few uses: AI generates fresh suggestions
phloem "docker logs for container"

# After 5+ successful uses with >70% success rate:
# → Instantly returns cached: docker logs <container_name>

# View cache statistics
phloem config
```

### Learning
Phloem evolves with your usage through `~/.phloem/PHLOEM.md`:

```markdown
### Docker
Last updated: 2024-01-24
✓ Validated executable: `docker`
Context: "list running containers"  
Full command: `docker ps -a --format "table {{.Names}}\t{{.Status}}"`

✓ Successful execution:
"docker logs for container" → `docker logs my-app`
```

### Validation
- Validates commands using `which` and system PATH
- Scans `/usr/local/bin`, `/usr/bin`, `/bin` for available tools
- Rejects pseudo-commands and API-style syntax
- Learns valid executables progressively

## Commands

```bash
phloem init                    # Initialize setup
phloem config                  # Show configuration & cache stats
phloem doctor                  # Run diagnostics  
phloem clear --cache          # Clear suggestion cache
phloem clear --context        # Reset learning context
phloem "your natural language query"
```

## Project Structure

```
~/.phloem/
├── PHLOEM.md              # Evolving knowledge base
├── config.toml              # Configuration
├── cache/
│   └── suggestions.db       # Smart cache with success tracking
└── backups/                 # PHLOEM.md backups

src/
├── cli/                     # Command-line interface & interactions  
├── ai/                      # Ollama integration & prompt engineering
├── context/                 # Caching, learning, shell history
├── config/                  # Configuration management
└── utils/                   # Environment detection, validation
```

## Development Setup

For contributors and developers who want to build locally:

```bash
# Clone the repository
# SSH (if you have GitHub SSH keys set up)
git clone git@github.com:phloem-sh/phloem.git
# OR HTTPS
git clone https://github.com/phloem-sh/phloem.git
cd phloem

# Build and install
cargo build --release
cargo install --path .

# Initialize for development
phloem init
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.