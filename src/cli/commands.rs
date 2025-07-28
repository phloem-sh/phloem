use anyhow::Result;
use log::{debug, info, warn};
use std::io;
use std::path::PathBuf;

use crate::ai::OllamaClient;
use crate::cli::{Commands, FormatResult, OutputFormatter, PromptOptions, Spinner};
use crate::config::Settings;
use crate::context::ContextManager;

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub command: String,
    pub explanation: Option<String>,
    pub confidence: f32,
}

pub struct CommandHandler {
    context: ContextManager,
    ai_client: OllamaClient,
    settings: Settings,
    formatter: OutputFormatter,
}

impl CommandHandler {
    pub fn new() -> Result<Self> {
        let settings = Settings::load()?;
        let context = ContextManager::new(&settings)?;
        let ai_client = OllamaClient::new(&settings)?;
        let formatter = OutputFormatter::new(settings.output.use_colors);

        Ok(Self {
            context,
            ai_client,
            settings,
            formatter,
        })
    }

    pub async fn handle_prompt(
        &mut self,
        prompt: &str,
        options: PromptOptions,
    ) -> Result<Vec<Suggestion>> {
        debug!("Processing prompt: {prompt}");

        // Check cache first unless explicitly disabled
        if !options.no_cache {
            if let Ok(Some(cached)) = self.context.get_cached_suggestion(prompt) {
                info!("Found cached suggestion for prompt");
                return Ok(vec![cached]);
            }
        }

        // Load context for prompt enhancement
        let context_data = self.context.get_relevant_context(prompt)?;
        debug!(
            "Loaded context data with {} recent commands",
            context_data.recent_commands.len()
        );

        // Show spinner while generating suggestions
        let spinner = Spinner::new("Generating suggestions...");

        // Generate suggestions via AI
        let suggestions = self
            .ai_client
            .generate_suggestions(prompt, &context_data, options.max_suggestions)
            .await?;

        spinner.stop();
        info!("Generated {} suggestions", suggestions.len());

        // Cache successful results
        for suggestion in &suggestions {
            if let Err(e) = self.context.cache_suggestion(prompt, suggestion) {
                warn!("Failed to cache suggestion: {e}");
            }
        }

        Ok(suggestions)
    }

    pub async fn handle_command(&mut self, command: Commands) -> Result<String> {
        match command {
            Commands::Init => self.handle_init().await,
            Commands::Update { model, binary } => self.handle_update(model, binary),
            Commands::Config => self.handle_config(),
            Commands::Clear { cache, context } => self.handle_clear(cache, context),
            Commands::Doctor => self.handle_doctor().await,
            Commands::Version => self.handle_version(),
        }
    }

    async fn handle_init(&mut self) -> Result<String> {
        info!("Initializing Phloem");

        let spinner = Spinner::new("Initializing phloem...");

        // Initialize ~/.phloem directory
        self.context.initialize_directory()?;

        // Check Ollama service
        if let Err(e) = self.ai_client.verify_connection().await {
            spinner.stop();
            return Ok(self.formatter.format_warning(&format!(
                "Ollama service not available: {e}. Make sure Ollama is installed and running."
            )));
        }

        spinner.stop();
        Ok(self
            .formatter
            .format_success("Phloem initialized successfully"))
    }

    fn handle_update(&mut self, model: bool, binary: bool) -> Result<String> {
        if !model && !binary {
            return Ok(self
                .formatter
                .format_info("Specify --model or --binary to update"));
        }

        let mut messages = Vec::new();

        if model {
            messages.push("Model update not yet implemented");
        }

        if binary {
            messages.push("Binary update not yet implemented");
        }

        Ok(messages.join("\n"))
    }

    fn handle_config(&self) -> Result<String> {
        let mut config_info = format!(
            "Phloem Configuration:\n\
            - Config file: {:?}\n\
            - Context file: {:?}\n\
            - Cache database: {:?}\n\
            - Model path: {:?}\n\
            - Max suggestions: {}\n\
            - Use colors: {}\n\n",
            self.settings.get_config_path(),
            self.context.get_context_file_path(),
            self.context.get_cache_path(),
            self.settings.model.model_path,
            self.settings.output.max_suggestions,
            self.settings.output.use_colors
        );

        // Add cache statistics
        if let Ok(stats) = self.context.cache.get_cache_stats() {
            config_info.push_str(&stats);
        }

        Ok(config_info)
    }

    fn handle_clear(&mut self, cache: bool, context: bool) -> Result<String> {
        let mut messages = Vec::new();

        if cache {
            self.context.clear_cache()?;
            messages.push(self.formatter.format_success("Cache cleared"));
        }

        if context {
            self.context.clear_context()?;
            messages.push(self.formatter.format_success("Context cleared"));
        }

        if !cache && !context {
            messages.push(
                self.formatter
                    .format_info("Specify --cache or --context to clear"),
            );
        }

        Ok(messages.join("\n"))
    }

    async fn handle_doctor(&self) -> Result<String> {
        let spinner = Spinner::new("Running diagnostics...");
        let mut diagnostics = Vec::new();

        // Check directories
        let phloem_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".phloem");

        if phloem_dir.exists() {
            diagnostics.push("✓ ~/.phloem directory exists".to_string());
        } else {
            diagnostics.push("✗ ~/.phloem directory missing (run: phloem init)".to_string());
        }

        // Check Ollama connection
        match self.ai_client.verify_connection().await {
            Ok(_) => diagnostics.push("✓ Ollama service running".to_string()),
            Err(e) => diagnostics.push(format!("✗ Ollama service: {e}")),
        }

        // Check database
        if self.context.get_cache_path().exists() {
            diagnostics.push("✓ Cache database exists".to_string());
        } else {
            diagnostics.push("✗ Cache database missing".to_string());
        }

        // Check model
        let model_path = PathBuf::from(&self.settings.model.model_path);
        if model_path.exists() {
            diagnostics.push("✓ Model files found".to_string());
        } else {
            diagnostics.push("✗ Model files missing (run installation script)".to_string());
        }

        spinner.stop();
        Ok(format!("Phloem Health Check:\n{}", diagnostics.join("\n")))
    }

    fn handle_version(&self) -> Result<String> {
        Ok(format!(
            "phloem {}\nRust version: {}\nPlatform: {}",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_RUST_VERSION"),
            std::env::consts::OS
        ))
    }

    pub async fn format_suggestions(
        &mut self,
        mut suggestions: Vec<Suggestion>,
        show_explanations: bool,
        original_prompt: &str,
    ) -> Result<String> {
        loop {
            match self.formatter.format_suggestions(
                &suggestions,
                show_explanations,
                original_prompt,
                &mut self.context,
            ) {
                FormatResult::Executed(output) => return Ok(output),
                FormatResult::Output(output) => return Ok(output),
                FormatResult::Static(output) => return Ok(output),
                FormatResult::FollowupRequested => {
                    // Ask user for modification request
                    println!("What would you like to modify about the command?");
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let modification_request = input.trim();

                    if modification_request.is_empty() {
                        continue;
                    }

                    // Create follow-up prompt (much cleaner)
                    let followup_prompt =
                        format!("{original_prompt} ({})", modification_request.trim());

                    // Get new suggestions
                    let options = PromptOptions {
                        max_suggestions: 3,
                        no_cache: true,
                        explain: false,
                        verbose: false,
                    };

                    match self.handle_prompt(&followup_prompt, options).await {
                        Ok(new_suggestions) => {
                            // Replace suggestions and continue the loop
                            suggestions = new_suggestions;
                            continue;
                        }
                        Err(e) => {
                            return Ok(self.format_error(&format!(
                                "Failed to get follow-up suggestions: {e}"
                            )));
                        }
                    }
                }
            }
        }
    }

    pub fn format_error(&self, message: &str) -> String {
        self.formatter.format_error(message)
    }
}
