use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::cli::Suggestion;
use crate::config::Settings;
use crate::context::{CacheManager, StorageManager};
use crate::utils::environment::EnvironmentDetector;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextData {
    pub content: String,
    pub environment: HashMap<String, String>,
    pub recent_commands: Vec<String>,
    pub prompt_category: String,
}

pub struct ContextManager {
    pub cache: CacheManager,
    storage: StorageManager,
    env_detector: EnvironmentDetector,
}

impl ContextManager {
    pub fn new(_settings: &Settings) -> Result<Self> {
        let storage = StorageManager::new()?;
        let cache_path = storage
            .get_phloem_dir()
            .join("cache")
            .join("suggestions.db");
        let cache = CacheManager::new(&cache_path)?;
        let env_detector = EnvironmentDetector::new();

        Ok(Self {
            cache,
            storage,
            env_detector,
        })
    }

    pub fn initialize_directory(&mut self) -> Result<()> {
        info!("Initializing Commandy directory structure");
        self.storage.initialize_directory()?;

        // Detect and store initial environment
        let env_info = self.env_detector.detect_environment()?;
        self.update_environment_info(&env_info)?;

        Ok(())
    }

    pub fn get_cached_suggestion(&self, prompt: &str) -> Result<Option<Suggestion>> {
        debug!("Checking cache for prompt: {prompt}");
        self.cache.get_suggestion(prompt)
    }

    pub fn cache_suggestion(&mut self, prompt: &str, suggestion: &Suggestion) -> Result<()> {
        debug!("Caching suggestion for prompt: {prompt}");
        self.cache.cache_suggestion(prompt, suggestion)?;

        // Also update context learning
        self.update_context_learning(prompt, suggestion)?;

        Ok(())
    }

    pub fn get_relevant_context(&self, prompt: &str) -> Result<ContextData> {
        debug!("Loading relevant context for prompt: {prompt}");

        // Read context file
        let context_content = self.storage.read_context_file()?;

        // Get environment information
        let environment = self.cache.get_environment()?;

        // Get recent successful commands from commandy history
        let mut recent_commands = self.cache.get_recent_commands(10)?;

        // Integrate shell history for richer context
        if let Ok(shell_history) = self.cache.get_shell_history() {
            // Add relevant shell commands to context
            let relevant_shell_commands: Vec<String> = shell_history
                .into_iter()
                .take(20) // Get more shell history
                .filter(|cmd| self.is_command_relevant(cmd, prompt))
                .collect();

            // Merge and deduplicate
            recent_commands.extend(relevant_shell_commands);
            recent_commands.sort();
            recent_commands.dedup();
        }

        // Categorize the prompt
        let prompt_category = self.categorize_prompt(prompt);

        Ok(ContextData {
            content: context_content,
            environment,
            recent_commands,
            prompt_category,
        })
    }

    pub fn record_command_execution(
        &mut self,
        command: &str,
        prompt: &str,
        success: bool,
        exit_code: Option<i32>,
    ) -> Result<()> {
        debug!("Recording command execution: {command} (success: {success})");

        // Record in history table
        self.cache
            .record_command_execution(command, prompt, success, exit_code)?;

        // Update suggestion success metrics
        if let Err(e) = self.cache.record_suggestion_usage(prompt, command, success) {
            warn!("Failed to update suggestion usage metrics: {e}");
        }

        if success {
            self.update_successful_command_pattern(prompt, command)?;
        }

        Ok(())
    }

    pub fn record_suggestion_feedback(
        &mut self,
        prompt: &str,
        command: &str,
        success: bool,
    ) -> Result<()> {
        debug!("Recording suggestion feedback: {prompt} -> {command} (success: {success})");

        // If successful, learn about the command pattern
        if success {
            self.learn_successful_command(prompt, command)?;
        }

        self.cache.record_suggestion_usage(prompt, command, success)
    }

    fn learn_successful_command(&self, prompt: &str, command: &str) -> Result<()> {
        // Extract the executable name
        let executable = command.split_whitespace().next().unwrap_or("").trim();

        // Skip common commands that don't need learning
        let skip_learning = ["ls", "cd", "pwd", "echo", "cat", "grep"];
        if skip_learning.contains(&executable) {
            return Ok(());
        }

        let category = self.categorize_prompt(prompt);

        // Update COMMANDY.md with learned command pattern
        let learning_content = format!(
            "✓ Validated executable: `{executable}`\n\
            Context: \"{prompt}\"\n\
            Full command: `{command}`"
        );

        self.storage
            .append_to_context(&category, &learning_content)?;

        Ok(())
    }

    pub fn clear_cache(&mut self) -> Result<()> {
        info!("Clearing command cache");
        self.cache.clear_cache()
    }

    pub fn clear_context(&self) -> Result<()> {
        info!("Clearing learning context");
        self.storage.clear_context()
    }

    pub fn get_context_file_path(&self) -> &PathBuf {
        self.storage.get_context_file_path()
    }

    pub fn get_cache_path(&self) -> PathBuf {
        self.storage
            .get_phloem_dir()
            .join("cache")
            .join("suggestions.db")
    }

    fn update_environment_info(&mut self, env_info: &HashMap<String, String>) -> Result<()> {
        for (key, value) in env_info {
            if let Err(e) = self.cache.update_environment(key, value) {
                warn!("Failed to update environment info for {key}: {e}");
            }
        }
        Ok(())
    }

    fn categorize_prompt(&self, prompt: &str) -> String {
        let prompt_lower = prompt.to_lowercase();

        // Simple categorization based on keywords
        if prompt_lower.contains("docker") || prompt_lower.contains("container") {
            "Docker".to_string()
        } else if prompt_lower.contains("kubectl")
            || prompt_lower.contains("pod")
            || prompt_lower.contains("kubernetes")
        {
            "Kubernetes".to_string()
        } else if prompt_lower.contains("git")
            || prompt_lower.contains("commit")
            || prompt_lower.contains("branch")
        {
            "Git".to_string()
        } else if prompt_lower.contains("file")
            || prompt_lower.contains("find")
            || prompt_lower.contains("ls")
        {
            "File Management".to_string()
        } else if prompt_lower.contains("process")
            || prompt_lower.contains("kill")
            || prompt_lower.contains("ps")
        {
            "Process Management".to_string()
        } else {
            "General".to_string()
        }
    }

    fn update_context_learning(&self, prompt: &str, suggestion: &Suggestion) -> Result<()> {
        let category = self.categorize_prompt(prompt);

        let learning_content = format!(
            "User prompt: \"{}\"\n→ Suggested: `{}`\n{}",
            prompt,
            suggestion.command,
            suggestion
                .explanation
                .as_ref()
                .map(|e| format!("Explanation: {e}"))
                .unwrap_or_default()
        );

        self.storage
            .append_to_context(&category, &learning_content)?;

        Ok(())
    }

    fn update_successful_command_pattern(&self, prompt: &str, command: &str) -> Result<()> {
        let category = self.categorize_prompt(prompt);

        let success_content = format!("✓ Successful execution:\n\"{prompt}\" → `{command}`");

        self.storage
            .append_to_context(&category, &success_content)?;

        Ok(())
    }

    fn is_command_relevant(&self, command: &str, prompt: &str) -> bool {
        let prompt_lower = prompt.to_lowercase();
        let command_lower = command.to_lowercase();

        // Skip very common/basic commands that don't add much context
        let basic_commands = ["ls", "cd", "pwd", "clear", "exit", "history"];
        if basic_commands
            .iter()
            .any(|&basic| command_lower.starts_with(basic))
        {
            return false;
        }

        // Include commands that share keywords with the prompt
        let prompt_words: Vec<&str> = prompt_lower.split_whitespace().collect();
        let command_words: Vec<&str> = command_lower.split_whitespace().collect();

        // Check for common keywords
        for prompt_word in &prompt_words {
            if prompt_word.len() > 3 {
                // Skip short words
                for command_word in &command_words {
                    if command_word.contains(prompt_word) || prompt_word.contains(command_word) {
                        return true;
                    }
                }
            }
        }

        // Include commands from the same category
        let prompt_category = self.categorize_prompt(prompt);
        let command_category = self.categorize_prompt(command);

        if prompt_category != "General" && prompt_category == command_category {
            return true;
        }

        false
    }
}
