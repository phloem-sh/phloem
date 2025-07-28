# Context Management

## Overview
Manages caching, learning, and shell history integration. Maintains knowledge base in `~/.phloem/PHLOEM.md` and tracks command success patterns.

## Directory Structure

```
~/.phloem/
├── PHLOEM.md              # Knowledge base with learned patterns
├── config.toml            # User configuration
├── cache/
│   └── suggestions.db     # Cache with success tracking
└── backups/
    ├── PHLOEM_20240124_143022.md  # Timestamped backups
    └── PHLOEM_20240123_091205.md  # (keeps last 5)
```

*Note: Models are managed by Ollama separately, not stored in ~/.phloem/*

## Caching System

### Strategy
Only caches suggestions proven to work well:

```sql
-- Only cache suggestions with proven success
SELECT suggestion FROM suggestions 
WHERE use_count >= 5 
  AND success_rate > 0.7
  AND created_at > datetime('now', '-7 days')
ORDER BY (success_rate * 0.6 + confidence * 0.4) DESC
```

### Success Tracking
Each suggestion tracks:
- `use_count`: How many times it's been used
- `success_count`: How many times it executed successfully  
- `success_rate`: Calculated as success_count / use_count
- `confidence`: AI's original confidence score

### Cache Statistics
View cache performance with `phloem config`:
```
Cache Statistics:
- Total suggestions: 157
- Ready for reuse: 23 (14.6%)
- Average success rate: 82.3%
- High success (>80%): 31
```

## PHLOEM.md Format

### Structure
The PHLOEM.md file serves as a living document that learns from successful command patterns:

```markdown
# Phloem Context

## User Profile
- **OS**: macOS 14.2 (Darwin)
- **Shell**: zsh
- **Terminal**: iTerm2
- **Preferred Style**: Concise commands with explanations

## Environment
- **Detected Tools**: kubectl, docker, git, npm, python, rust
- **Container Runtime**: Docker Desktop
- **Cloud Provider**: AWS (detected from ~/.aws/)
- **Kubernetes Context**: dev-cluster

## Command Patterns

### Kubernetes
Last updated: 2024-01-15
User prefers:
- `kubectl get pods` over `k get po`
- Full resource names over abbreviations
- Namespace explicit when not default

Example successful commands:
- "get running pods" → `kubectl get pods --field-selector=status.phase=Running`
- "describe failing pod" → `kubectl describe pod $(kubectl get pods --field-selector=status.phase!=Running -o name | head -1)`

### Docker
Last updated: 2024-01-14
User prefers:
- Interactive mode for debugging
- Volume mounts for development

Example successful commands:
- "run ubuntu container" → `docker run -it ubuntu:latest bash`
- "list running containers" → `docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"`

### Git
Last updated: 2024-01-13
User prefers:
- Descriptive commit messages
- Branch naming with feature/ prefix

## Recent Context
- Working on microservices project
- Using Kubernetes for orchestration
- Debugging pod connectivity issues
- Frequently needs to check logs and describe resources

## Learning Notes
- User often types "get pods" instead of full kubectl command
- Prefers explanatory flags when available
- Likes to see resource status in tabular format
```

### Dynamic Updates
The PHLOEM.md file is updated after each successful command:

1. **Command Execution**: User runs suggested command successfully
2. **Pattern Recognition**: System identifies command pattern and context
3. **Context Update**: Append to relevant section with timestamp
4. **Pruning**: Remove old entries to keep file manageable (<50KB)

## Configuration System (config.toml)

```toml
[general]
# Maximum context file size before pruning
max_context_size_kb = 50

# Number of recent commands to remember
recent_commands_limit = 100

# Enable learning from successful commands
learning_enabled = true

[model]
# Model configuration
model_path = "~/.phloem/models/gemma-3n"
max_tokens = 100
temperature = 0.0

[cache]
# Cache configuration
max_cache_entries = 1000
cache_ttl_hours = 24

[output]
# Output formatting preferences
show_explanations = true
use_colors = true
max_suggestions = 3

[privacy]
# Privacy settings
collect_usage_stats = false
share_anonymous_data = false
```

## Cache Database Schema

### SQLite Tables

```sql
-- Command suggestions cache
CREATE TABLE suggestions (
    id INTEGER PRIMARY KEY,
    prompt_hash TEXT UNIQUE,
    prompt TEXT,
    suggestion TEXT,
    explanation TEXT,
    confidence REAL,
    created_at TIMESTAMP,
    last_used TIMESTAMP,
    use_count INTEGER DEFAULT 1
);

-- Command execution history
CREATE TABLE history (
    id INTEGER PRIMARY KEY,
    command TEXT,
    prompt TEXT,
    success BOOLEAN,
    exit_code INTEGER,
    executed_at TIMESTAMP,
    context_snapshot TEXT  -- JSON of environment at execution time
);

-- Environment tracking
CREATE TABLE environment (
    key TEXT PRIMARY KEY,
    value TEXT,
    detected_at TIMESTAMP,
    updated_at TIMESTAMP
);
```

## Context Learning Algorithm

### Pattern Recognition
1. **Command Categorization**: Group commands by tool (kubectl, docker, git)
2. **Style Analysis**: Identify user preferences (verbosity, formatting)
3. **Context Correlation**: Link commands to project/directory context
4. **Success Tracking**: Weight suggestions based on execution success

### Context Expansion
```python
def update_context(prompt: str, command: str, success: bool):
    # Extract command category
    category = detect_command_category(command)
    
    # Update pattern database
    if success:
        strengthen_pattern(category, prompt, command)
    else:
        weaken_pattern(category, prompt, command)
    
    # Update PHLOEM.md with new learning
    append_to_context_section(category, {
        'prompt': prompt,
        'command': command,
        'timestamp': now(),
        'success': success
    })
```

## Privacy & Security

### Local-Only Storage
- All context data remains on local machine
- No cloud synchronization or external API calls
- User controls all data retention policies

### Data Encryption
- Sensitive data in cache database encrypted at rest
- Environment variables containing secrets are filtered out
- User can opt out of any data collection

### Data Retention
```toml
[retention]
# Automatically remove old entries
max_history_days = 90
max_cache_days = 30
auto_backup_enabled = true
backup_retention_days = 7
```

## Context API

### Reading Context
```rust
pub struct ContextManager {
    pub fn load_context(&self) -> Result<Context>;
    pub fn get_environment(&self) -> HashMap<String, String>;
    pub fn get_recent_commands(&self, limit: usize) -> Vec<Command>;
    pub fn get_patterns_for_tool(&self, tool: &str) -> Vec<Pattern>;
}
```

### Updating Context
```rust
impl ContextManager {
    pub fn record_command(&mut self, prompt: &str, command: &str, success: bool);
    pub fn update_environment(&mut self, changes: HashMap<String, String>);
    pub fn prune_old_data(&mut self);
    pub fn backup_context(&self) -> Result<PathBuf>;
}
```

## Migration & Versioning

### Context Version
Each PHLOEM.md includes version header for future migrations:

```markdown
<!-- PHLOEM_VERSION: 1.0 -->
<!-- LAST_UPDATED: 2024-01-15T10:30:00Z -->
# Phloem Context
```

### Upgrade Path
- Detect old format contexts
- Migrate data preserving user patterns
- Backup before migration
- Graceful fallback for incompatible versions