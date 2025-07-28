// External dependencies
use anyhow::{Context, Result};
use log::{debug, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

// Internal dependencies
use crate::cli::Suggestion;
use crate::config::Settings;
use crate::context::ContextData;

// ============================================================================
// JSON Response Structures
// ============================================================================

#[derive(Debug, Deserialize)]
struct CommandSuggestion {
    command: String,
    explanation: String,
}

#[derive(Debug, Deserialize)]
struct CommandsResponse {
    commands: Vec<CommandSuggestion>,
}

// ============================================================================
// Ollama API Structures
// ============================================================================

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: Option<String>,
    options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

pub struct OllamaClient {
    client: Client,
    base_url: Url,
    model_name: String,
}

// ============================================================================
// Client Implementation
// ============================================================================

impl OllamaClient {
    /// Creates a new OllamaClient instance with default configuration
    pub fn new(_settings: &Settings) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let base_url = Url::parse("http://localhost:11434").context("Invalid Ollama base URL")?;
        let model_name = "gemma3n:e2b".to_string();

        Ok(Self {
            client,
            base_url,
            model_name,
        })
    }

    // ========================================================================
    // Connection and Model Management
    // ========================================================================

    /// Verifies connection to the Ollama service
    pub async fn verify_connection(&self) -> Result<()> {
        debug!("Verifying Ollama connection");

        let url = self
            .base_url
            .join("/api/version")
            .context("Failed to build version URL")?;

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to connect to Ollama service")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Ollama service returned error: {}",
                response.status()
            ));
        }

        info!("Ollama connection verified");
        Ok(())
    }

    /// Lists all available models from the Ollama service
    pub async fn list_models(&self) -> Result<Vec<String>> {
        debug!("Listing available models");

        let url = self
            .base_url
            .join("/api/tags")
            .context("Failed to build tags URL")?;

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to list models")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to list models: {}",
                response.status()
            ));
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .context("Failed to parse models response")?;

        let model_names: Vec<String> = models_response.models.into_iter().map(|m| m.name).collect();

        debug!("Found {} models", model_names.len());
        Ok(model_names)
    }

    /// Ensures the configured model is available, pulling it if necessary
    pub async fn ensure_model_available(&self) -> Result<()> {
        debug!("Ensuring model {} is available", self.model_name);

        let models = self.list_models().await?;

        if models.contains(&self.model_name) {
            info!("Model {} already available", self.model_name);
            return Ok(());
        }

        info!("Model {} not found, pulling...", self.model_name);
        self.pull_model().await
    }

    /// Pulls the specified model from Ollama
    async fn pull_model(&self) -> Result<()> {
        let url = self
            .base_url
            .join("/api/pull")
            .context("Failed to build pull URL")?;

        let request_body = serde_json::json!({
            "name": self.model_name
        });

        info!(
            "Pulling model {}, this may take a while...",
            self.model_name
        );

        let response = self
            .client
            .post(url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to start model pull")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to pull model: {}",
                response.status()
            ));
        }

        // Note: In production, we'd stream the response and show progress
        info!("Model {} pulled successfully", self.model_name);
        Ok(())
    }

    // ========================================================================
    // Suggestion Generation
    // ========================================================================

    /// Generates command suggestions based on user prompt and context
    pub async fn generate_suggestions(
        &self,
        prompt: &str,
        context: &ContextData,
        max_suggestions: usize,
    ) -> Result<Vec<Suggestion>> {
        debug!("Generating suggestions for prompt: {prompt}");

        let enhanced_prompt = self.build_enhanced_prompt(prompt, context);
        let response = self.generate_text(&enhanced_prompt).await?;
        let suggestions = self.parse_response(&response, max_suggestions);

        info!("Generated {} suggestions", suggestions.len());
        Ok(suggestions)
    }

    async fn generate_text(&self, prompt: &str) -> Result<String> {
        let url = self
            .base_url
            .join("/api/generate")
            .context("Failed to build generate URL")?;

        let mut options = HashMap::new();
        options.insert("temperature".to_string(), serde_json::Value::from(0.0));
        options.insert("top_k".to_string(), serde_json::Value::from(40));
        options.insert("top_p".to_string(), serde_json::Value::from(0.9));
        options.insert("num_predict".to_string(), serde_json::Value::from(200));

        let request = OllamaGenerateRequest {
            model: self.model_name.clone(),
            prompt: prompt.to_string(),
            stream: false,
            format: Some("json".to_string()),
            options,
        };

        debug!("Sending request to Ollama, prompt length: {}", prompt.len());

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .context("Failed to send generate request")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Generate request failed: {}",
                response.status()
            ));
        }

        let generate_response: OllamaGenerateResponse = response
            .json()
            .await
            .context("Failed to parse generate response")?;

        if !generate_response.done {
            warn!("Generation was not completed");
        }

        debug!(
            "Generated response length: {}",
            generate_response.response.len()
        );
        Ok(generate_response.response)
    }

    fn build_enhanced_prompt(&self, user_prompt: &str, context: &ContextData) -> String {
        let environment = &context.environment;
        let recent_commands = &context.recent_commands;
        let context_content = &context.content;

        let available_tools = &environment
            .get("available_tools")
            .map_or("basic".to_string(), |v| {
                v.split(',').take(20).collect::<Vec<_>>().join(", ")
            });

        let mut prompt = format!(
            r#"Generate ONLY valid shell commands for: {}

OS: {} | Shell: {} 
AVAILABLE EXECUTABLES: {}
Recent Commands: {}

CRITICAL - Commands MUST:
1. Use ONLY executables listed above that exist in PATH
2. Start with a real command name, not pseudo-commands
3. Use proper shell syntax
4. Be directly runnable

IMPORTANT: If "lazygit" is in available executables, suggest "lazygit" not installation commands.

"#,
            user_prompt,
            environment.get("os").map_or("unknown", |v| v.as_str()),
            environment.get("shell").map_or("unknown", |v| v.as_str()),
            available_tools,
            recent_commands
                .iter()
                .take(2)
                .map(|cmd| cmd.split_whitespace().next().unwrap_or(""))
                .collect::<Vec<_>>()
                .join(",")
        );

        // Add learned context from PHLOEM.md if available
        if !context_content.is_empty() {
            prompt.push_str("\nLEARNED PATTERNS (use for reference):\n");
            prompt.push_str(
                &context_content
                    .lines()
                    .filter(|line| line.contains("→") || line.contains("✓"))
                    .take(10) // Limit to prevent prompt explosion
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            prompt.push('\n');
        }

        prompt.push_str(
            r#"
RESPONSE FORMAT - Return JSON exactly like this:
{
  "commands": [
    {"command": "actual_executable_command", "explanation": "brief description"},
    {"command": "another_command", "explanation": "brief description"}
  ]
}

Generate maximum 3 commands in this JSON format:"#,
        );

        prompt
    }

    fn parse_response(&self, response: &str, max_suggestions: usize) -> Vec<Suggestion> {
        debug!("Parsing JSON response: {response}");

        // Try to parse as JSON first
        match serde_json::from_str::<CommandsResponse>(response) {
            Ok(commands_response) => {
                let mut suggestions = Vec::new();

                for cmd_suggestion in commands_response.commands.into_iter().take(max_suggestions) {
                    if self.is_valid_command(&cmd_suggestion.command) {
                        suggestions.push(Suggestion {
                            command: cmd_suggestion.command,
                            explanation: Some(cmd_suggestion.explanation),
                            confidence: 0.8,
                        });
                    } else {
                        debug!("Invalid command rejected: {}", cmd_suggestion.command);
                    }
                }

                if !suggestions.is_empty() {
                    return suggestions;
                }
            }
            Err(e) => {
                debug!("JSON parsing failed: {e}, trying fallback");
            }
        }

        // Fallback: try to extract commands from text response
        self.extract_commands_fallback(response, max_suggestions)
    }

    fn extract_commands_fallback(&self, response: &str, max_suggestions: usize) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        for line in response.lines() {
            let line = line.trim();

            // Skip empty lines and obvious non-commands
            if line.is_empty() || line.starts_with('#') || line.len() > 200 {
                continue;
            }

            // Look for lines that look like commands
            if self.looks_like_command(line) && self.is_valid_command(line) {
                suggestions.push(Suggestion {
                    command: line.to_string(),
                    explanation: None,
                    confidence: 0.6,
                });

                if suggestions.len() >= max_suggestions {
                    break;
                }
            }
        }

        suggestions
    }

    fn looks_like_command(&self, line: &str) -> bool {
        // Simple heuristics to identify command-like lines
        let starts_with_command = line
            .split_whitespace()
            .next()
            .map(|first_word| {
                // Common command prefixes
                matches!(
                    first_word,
                    "ls" | "cd"
                        | "grep"
                        | "find"
                        | "docker"
                        | "kubectl"
                        | "git"
                        | "curl"
                        | "wget"
                        | "ssh"
                        | "sudo"
                        | "cp"
                        | "mv"
                        | "rm"
                        | "cat"
                        | "tail"
                        | "head"
                        | "ps"
                        | "kill"
                        | "top"
                        | "df"
                        | "du"
                        | "tar"
                        | "zip"
                        | "unzip"
                )
            })
            .unwrap_or(false);

        starts_with_command || line.contains("--") || line.contains("|")
    }

    fn is_valid_command(&self, command: &str) -> bool {
        // Basic safety checks
        let dangerous_patterns = ["rm -rf /", "rm -rf *", "dd if=", "mkfs", "fdisk", "> /dev/"];

        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                return false;
            }
        }

        // Must not be empty and not too long
        if command.is_empty() || command.len() > 500 {
            return false;
        }

        // Extract the first word (the executable name)
        let first_word = command.split_whitespace().next().unwrap_or("").trim();

        // Skip shell operators and redirections
        if first_word.is_empty() || first_word.starts_with('#') {
            return false;
        }

        // Check if it's executable using 'which' command
        if let Ok(output) = std::process::Command::new("which").arg(first_word).output() {
            if output.status.success() {
                return true;
            }
        }

        // Allow shell built-ins and paths
        if first_word.contains('/')
            || first_word == "cd"
            || first_word == "echo"
            || first_word == "pwd"
        {
            return true;
        }

        // Reject commands that look like pseudo-commands or APIs
        let pseudo_patterns = [" query ", " api ", " endpoint ", " service "];
        for pattern in &pseudo_patterns {
            if command.to_lowercase().contains(pattern) {
                return false;
            }
        }

        // Log unknown commands for debugging
        log::debug!("Command '{first_word}' not found in PATH");
        false
    }
}
