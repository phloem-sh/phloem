use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "phloem")]
#[command(about = "Secure, fast command suggestions using local models")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(long_about = None)]
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
        /// Update the ML model
        #[arg(long)]
        model: bool,
        /// Update the binary
        #[arg(long)]
        binary: bool,
    },
    /// Show configuration
    Config,
    /// Clear cache and context
    Clear {
        /// Clear command cache
        #[arg(long)]
        cache: bool,
        /// Clear learning context
        #[arg(long)]
        context: bool,
    },
    /// Run diagnostics
    Doctor,
    /// Show version information
    Version,
}

#[derive(Debug, Clone)]
pub struct PromptOptions {
    pub no_cache: bool,
    pub explain: bool,
    pub max_suggestions: usize,
    pub verbose: bool,
}

impl From<&Cli> for PromptOptions {
    fn from(cli: &Cli) -> Self {
        Self {
            no_cache: cli.no_cache,
            explain: cli.explain,
            max_suggestions: cli.suggestions,
            verbose: cli.verbose,
        }
    }
}
