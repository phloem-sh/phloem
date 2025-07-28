# Rust Core Implementation

## Project Structure

```
src/
├── main.rs                  # Entry point and CLI setup
├── lib.rs                   # Library exports
├── cli/
│   ├── mod.rs              # CLI module
│   ├── args.rs             # Argument parsing (clap)
│   ├── commands.rs         # Command handlers
│   └── output.rs           # Interactive output formatting
├── ai/
│   ├── mod.rs              # AI integration module
│   ├── ollama_client.rs    # Ollama HTTP client
│   ├── prompt.rs           # Prompt engineering
│   └── response.rs         # Response parsing
├── context/
│   ├── mod.rs              # Context module
│   ├── manager.rs          # Context management coordination
│   ├── cache.rs            # SQLite cache operations
│   └── storage.rs          # File system operations
├── config/
│   ├── mod.rs              # Configuration module
│   ├── settings.rs         # Settings management
│   └── defaults.rs         # Default configuration values
└── utils/
    ├── mod.rs              # Utilities module
    ├── shell.rs            # Shell detection and integration
    ├── environment.rs      # Environment detection
    └── validation.rs       # Command validation
```

## Core Modules

### 1. CLI Module (src/cli/)

#### args.rs - Command Line Interface
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "phloem")]
#[command(about = "Secure, fast command suggestions using local models")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// The prompt to generate a command for
    pub prompt: Option<String>,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Show detailed explanations
    #[arg(short, long)]
    pub explain: bool,
    
    /// Number of suggestions to show
    #[arg(short = 'n', long, default_value = "3")]
    pub suggestions: usize,
    
    /// Skip cache and force fresh inference
    #[arg(long)]
    pub no_cache: bool,
    
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize phloem setup
    Init,
    /// Update model or binary
    Update { 
        #[arg(long)] 
        model: bool,
        #[arg(long)] 
        binary: bool 
    },
    /// Show configuration
    Config,
    /// Clear cache and context
    Clear { 
        #[arg(long)] 
        cache: bool,
        #[arg(long)] 
        context: bool 
    },
    /// Run diagnostics
    Doctor,
    /// Show version information
    Version,
}
```

#### commands.rs - Command Handlers
```rust
use anyhow::Result;
use crate::context::ContextManager;
use crate::ai::OllamaClient;
use crate::config::Settings;

pub struct CommandHandler {
    context: ContextManager,
    ollama_client: OllamaClient,
    settings: Settings,
}

impl CommandHandler {
    pub fn new() -> Result<Self> {
        let settings = Settings::load()?;
        let context = ContextManager::new(&settings)?;
        let ollama_client = OllamaClient::new(&settings)?;
        
        Ok(Self { context, ollama_client, settings })
    }
    
    pub async fn handle_prompt(&mut self, prompt: &str, options: PromptOptions) -> Result<Vec<Suggestion>> {
        // Check cache first if not disabled
        if !options.no_cache {
            if let Some(cached) = self.context.get_cached_suggestion(prompt)? {
                return Ok(vec![cached]);
            }
        }
        
        // Load context for prompt enhancement
        let context_data = self.context.get_relevant_context(prompt)?;
        
        // Generate suggestions via Ollama
        let suggestions = self.ollama_client.generate_suggestions(prompt, &context_data).await?;
        
        // Cache successful results
        for suggestion in &suggestions {
            self.context.cache_suggestion(prompt, suggestion)?;
        }
        
        Ok(suggestions)
    }
    
    pub fn handle_init(&mut self) -> Result<()> {
        // Initialize ~/.phloem directory structure
        self.context.initialize_directory()?;
        
        // Check Ollama connectivity
        if let Err(e) = self.ollama_client.health_check() {
            eprintln!("Warning: Ollama not accessible: {}", e);
            eprintln!("Please ensure Ollama is installed and running:");
            eprintln!("  curl -fsSL https://ollama.ai/install.sh | sh");
            eprintln!("  ollama serve");
        }
        
        println!("✓ Phloem initialized successfully");
        Ok(())
    }
    
    pub fn handle_config(&self) -> Result<()> {
        println!("Phloem Configuration:");
        println!("  Config file: {}", self.settings.config_path.display());
        println!("  Ollama URL: {}", self.settings.ollama_url);
        println!("  Model: {}", self.settings.model_name);
        println!("  Cache path: {}", self.context.get_cache_path().display());
        
        // Show cache statistics
        let stats = self.context.cache.get_cache_stats()?;
        println!("\n{}", stats);
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct PromptOptions {
    pub no_cache: bool,
    pub explain: bool,
    pub max_suggestions: usize,
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub command: String,
    pub explanation: Option<String>,
    pub confidence: f32,
}
```

#### output.rs - Interactive Output
```rust
use crate::cli::Suggestion;
use crate::context::ContextManager;
use console::{style, Color};
use crossterm::event::{self, Event, KeyCode};

pub struct OutputFormatter {
    use_colors: bool,
}

impl OutputFormatter {
    pub fn format_suggestions(
        &self,
        suggestions: &[Suggestion],
        show_explanations: bool,
        original_prompt: &str,
        context: &mut ContextManager,
    ) -> FormatResult {
        if suggestions.is_empty() {
            return FormatResult::Static("No suggestions found.".to_string());
        }

        // Interactive selection with keyboard navigation
        self.interactive_select(suggestions, show_explanations, original_prompt, context)
    }
    
    fn interactive_select(&self, suggestions: &[Suggestion], /* ... */) -> FormatResult {
        // Display suggestions with keyboard navigation:
        // Enter → Execute command immediately
        // Tab → Copy to clipboard  
        // Escape → Modify/follow-up on command
        // Escape Escape → Exit to static view
        // F → Alternative follow-up key
        
        // Implementation handles terminal raw mode, cursor movement,
        // and command execution with feedback tracking
    }
}
```

### 2. AI Integration Module (src/ai/)

#### ollama_client.rs - Ollama HTTP Client
```rust
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::context::ContextData;

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model_name: String,
}

impl OllamaClient {
    pub fn new(settings: &Settings) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            base_url: settings.ollama_url.clone(),
            model_name: settings.model_name.clone(),
        })
    }
    
    pub async fn generate_suggestions(
        &self,
        prompt: &str,
        context: &ContextData,
    ) -> Result<Vec<Suggestion>> {
        // Build enhanced prompt with context
        let enhanced_prompt = self.build_prompt(prompt, context)?;
        
        // Make HTTP request to Ollama
        let request = OllamaRequest {
            model: self.model_name.clone(),
            prompt: enhanced_prompt,
            stream: false,
            options: OllamaOptions {
                temperature: 0.0,
                top_p: 0.9,
                max_tokens: 150,
            },
        };
        
        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;
        
        // Parse response into structured suggestions
        self.parse_response(&response.response)
    }
    
    pub fn health_check(&self) -> Result<()> {
        // Check if Ollama is running and model is available
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let response = self.client
                .get(&format!("{}/api/tags", self.base_url))
                .send()
                .await?;
            
            if response.status().is_success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Ollama not accessible"))
            }
        })
    }
    
    fn build_prompt(&self, user_prompt: &str, context: &ContextData) -> Result<String> {
        // Combine user prompt with system context, environment info,
        // recent commands, and PHLOEM.md patterns for enhanced suggestions
    }
    
    fn parse_response(&self, response: &str) -> Result<Vec<Suggestion>> {
        // Parse Ollama response into structured command suggestions
        // with confidence scores and explanations
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}
```

### 3. Context Module (src/context/)

#### manager.rs - Context Management
```rust
use anyhow::Result;
use std::collections::HashMap;
use crate::context::{CacheManager, StorageManager};

pub struct ContextManager {
    pub cache: CacheManager,
    storage: StorageManager,
    env_detector: EnvironmentDetector,
}

impl ContextManager {
    pub fn new(settings: &Settings) -> Result<Self> {
        let storage = StorageManager::new()?;
        let cache_path = storage.get_phloem_dir().join("cache/suggestions.db");
        let cache = CacheManager::new(&cache_path)?;
        let env_detector = EnvironmentDetector::new();

        Ok(Self { cache, storage, env_detector })
    }
    
    pub fn get_cached_suggestion(&self, prompt: &str) -> Result<Option<Suggestion>> {
        // Check SQLite cache for high-confidence suggestions
        // Only return cached results with:
        // - use_count >= 5
        // - success_rate > 0.7
        // - created_at > datetime('now', '-7 days')
        self.cache.get_suggestion(prompt)
    }
    
    pub fn get_relevant_context(&self, prompt: &str) -> Result<ContextData> {
        // Load PHLOEM.md content
        let context_content = self.storage.read_context_file()?;
        
        // Get environment information
        let environment = self.cache.get_environment()?;
        
        // Get recent successful commands from history
        let recent_commands = self.cache.get_recent_commands(10)?;
        
        // Integrate shell history for richer context
        if let Ok(shell_history) = self.cache.get_shell_history() {
            // Merge relevant shell commands with phloem history
        }
        
        Ok(ContextData {
            content: context_content,
            environment,
            recent_commands,
            prompt_category: self.categorize_prompt(prompt),
        })
    }
    
    pub fn record_command_execution(
        &mut self,
        command: &str,
        prompt: &str,
        success: bool,
        exit_code: Option<i32>,
    ) -> Result<()> {
        // Record command execution for learning
        self.cache.record_command_execution(command, prompt, success, exit_code)?;
        
        // Update PHLOEM.md with successful patterns
        if success {
            self.update_successful_command_pattern(prompt, command)?;
        }
        
        Ok(())
    }
}
```

#### cache.rs - SQLite Cache Operations
```rust
use rusqlite::{params, Connection};
use anyhow::Result;

pub struct CacheManager {
    connection: Connection,
}

impl CacheManager {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let connection = Connection::open(db_path)?;
        
        // Initialize schema from sql/schema.sql
        connection.execute_batch(include_str!("../../sql/schema.sql"))?;
        
        Ok(Self { connection })
    }
    
    pub fn get_suggestion(&self, prompt: &str) -> Result<Option<Suggestion>> {
        let prompt_hash = self.hash_prompt(prompt);
        
        // Only return suggestions that have proven successful
        let mut stmt = self.connection.prepare(
            "SELECT suggestion, explanation, confidence FROM suggestions 
             WHERE prompt_hash = ?1 
             AND created_at > datetime('now', '-7 days')
             AND use_count >= 5
             AND success_rate > 0.7
             ORDER BY (success_rate * 0.6 + confidence * 0.4) DESC 
             LIMIT 1"
        )?;
        
        // Implementation returns high-confidence cached suggestions
    }
    
    pub fn record_suggestion_usage(
        &mut self,
        prompt: &str,
        command: &str,
        success: bool,
    ) -> Result<()> {
        // Update usage statistics and success rates
        // This feeds into the caching decision algorithm
    }
}
```

## Main Entry Point (src/main.rs)

```rust
use anyhow::Result;
use clap::Parser;
use phloem::{Cli, CommandHandler, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
        .init();

    let cli = Cli::parse();

    // Handle version early
    if matches!(cli.command, Some(Commands::Version)) {
        let version_info = format!(
            "phloem {}\\nRust version: {}\\nPlatform: {}-{}",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_RUST_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        println!("{}", version_info);
        return Ok(());
    }

    // Initialize command handler
    let mut handler = CommandHandler::new()?;

    match cli.command {
        Some(command) => {
            // Handle subcommands (init, config, clear, doctor, etc.)
            match handler.handle_command(command).await {
                Ok(output) => println!("{}", output),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            if let Some(ref prompt) = cli.prompt {
                // Handle prompt for command generation
                let options = (&cli).into();
                let suggestions = handler.handle_prompt(prompt, options).await?;
                
                // Display interactive suggestions
                match handler.format_suggestions(suggestions, cli.explain, prompt).await {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Show help message
                println!("Phloem - Secure, fast command suggestions using local models");
                // ... help text
            }
        }
    }

    Ok(())
}
```

## Build Configuration (Cargo.toml)

```toml
[package]
name = "phloem"
version = "0.1.0"
edition = "2021"
description = "Secure, fast command suggestions using local models"
license = "MIT"
repository = "https://github.com/phloem-sh/phloem"
keywords = ["cli", "ai", "commands", "productivity"]
categories = ["command-line-utilities"]

[[bin]]
name = "phloem"
path = "src/main.rs"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
dirs = "5.0"
rusqlite = { version = "0.30", features = ["bundled"] }
console = "0.15"
indicatif = "0.17"
dialoguer = "0.11"
crossterm = "0.27"
arboard = "3.2"
log = "0.4"
env_logger = "0.10"
which = "4.0"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
toml = "0.8"
regex = "1.0"
reqwest = { version = "0.11", features = ["json"] }
url = "2.0"

[dev-dependencies]
tempfile = "3.0"
tokio-test = "0.4"
assert_cmd = "2.0"
predicates = "3.0"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## Key Implementation Features

### 1. **High Performance**
- **Sub-100ms Cache Lookups**: SQLite-backed intelligent caching
- **Async HTTP Client**: Non-blocking Ollama communication
- **Optimized Builds**: LTO, single codegen unit, stripped binaries

### 2. **Smart Caching Strategy**
- **Proven Success Only**: Cache entries require 5+ uses with >70% success rate
- **Temporal Awareness**: Recent suggestions weighted higher
- **Context Sensitive**: Different cache keys for different environments

### 3. **Interactive User Experience**
- **Keyboard Navigation**: Arrow keys, Enter, Tab, Escape shortcuts
- **Real-time Feedback**: Immediate command execution and success tracking
- **Clipboard Integration**: Easy copy-paste workflow

### 4. **Robust Error Handling**
- **Graceful Degradation**: Works offline with cached suggestions
- **Comprehensive Logging**: Debug information without user spam
- **Health Checks**: Automatic Ollama connectivity verification

### 5. **Privacy & Security**
- **Local-Only Processing**: All data stays on user's machine
- **Command Validation**: Built-in dangerous command filtering
- **No Telemetry**: Zero external data transmission

This Rust implementation provides a fast, secure, and user-friendly CLI tool that leverages local AI models through Ollama for intelligent command suggestions while maintaining complete privacy and offline functionality.