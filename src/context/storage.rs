use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

pub struct StorageManager {
    phloem_dir: PathBuf,
    context_file: PathBuf,
}

impl StorageManager {
    pub fn new() -> Result<Self> {
        let phloem_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".phloem");

        let context_file = phloem_dir.join("PHLOEM.md");

        Ok(Self {
            phloem_dir,
            context_file,
        })
    }

    pub fn initialize_directory(&self) -> Result<()> {
        // Create main directory
        fs::create_dir_all(&self.phloem_dir)?;

        // Create subdirectories
        let subdirs = ["cache", "models", "logs", "backups"];
        for subdir in &subdirs {
            fs::create_dir_all(self.phloem_dir.join(subdir))?;
        }

        // Initialize PHLOEM.md if it doesn't exist
        if !self.context_file.exists() {
            self.create_initial_context_file()?;
        }

        // Create default config if it doesn't exist
        let config_file = self.phloem_dir.join("config.toml");
        if !config_file.exists() {
            self.create_default_config()?;
        }

        Ok(())
    }

    pub fn read_context_file(&self) -> Result<String> {
        if !self.context_file.exists() {
            return Ok(String::new());
        }

        let content = fs::read_to_string(&self.context_file)?;
        Ok(content)
    }

    pub fn append_to_context(&self, section: &str, content: &str) -> Result<()> {
        let current_content = self.read_context_file()?;

        // Find the section or create it
        let updated_content = if current_content.contains(&format!("### {section}")) {
            self.update_existing_section(&current_content, section, content)
        } else {
            self.add_new_section(&current_content, section, content)
        };

        // Backup the current file
        self.backup_context_file()?;

        // Write updated content
        fs::write(&self.context_file, updated_content)?;

        Ok(())
    }

    pub fn clear_context(&self) -> Result<()> {
        self.backup_context_file()?;
        self.create_initial_context_file()?;
        Ok(())
    }

    pub fn get_context_file_path(&self) -> &PathBuf {
        &self.context_file
    }

    pub fn get_phloem_dir(&self) -> &PathBuf {
        &self.phloem_dir
    }

    fn create_initial_context_file(&self) -> Result<()> {
        let initial_content = format!(
            r#"<!-- PHLOEM_VERSION: 1.0 -->
<!-- LAST_UPDATED: {} -->
# Phloem Context

## User Profile
- **OS**: {}
- **Shell**: {}
- **Terminal**: {}
- **Preferred Style**: Concise commands with explanations

## Environment
- **Detected Tools**: 
- **Container Runtime**: 
- **Cloud Provider**: 
- **Current Context**: 

## Command Patterns

### Git
Last updated: {}
User prefers:
- Descriptive commit messages
- Feature branch workflow

### Docker
Last updated: {}
User prefers:
- Interactive mode for debugging
- Volume mounts for development

### Kubernetes
Last updated: {}
User prefers:
- Full resource names over abbreviations
- Explicit namespace specification

## Recent Context
- Working directory: {}
- Current project: Unknown

## Learning Notes
- User interaction patterns will be recorded here
- Command preferences will be learned over time
"#,
            Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
            std::env::consts::OS,
            std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string()),
            std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string()),
            Utc::now().format("%Y-%m-%d"),
            Utc::now().format("%Y-%m-%d"),
            Utc::now().format("%Y-%m-%d"),
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
        );

        fs::write(&self.context_file, initial_content)?;
        Ok(())
    }

    fn create_default_config(&self) -> Result<()> {
        let config_content = r#"[general]
max_context_size_kb = 50
recent_commands_limit = 100
learning_enabled = true

[model]
model_path = "~/.phloem/models/gemma-3n"
max_tokens = 100
temperature = 0.0

[cache]
max_cache_entries = 1000
cache_ttl_hours = 24

[output]
show_explanations = true
use_colors = true
max_suggestions = 3

[privacy]
collect_usage_stats = false
share_anonymous_data = false
"#;

        let config_path = self.phloem_dir.join("config.toml");
        fs::write(config_path, config_content)?;
        Ok(())
    }

    fn backup_context_file(&self) -> Result<()> {
        if !self.context_file.exists() {
            return Ok(());
        }

        let backup_dir = self.phloem_dir.join("backups");
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = backup_dir.join(format!("PHLOEM_{timestamp}.md"));

        fs::copy(&self.context_file, backup_path)?;

        // Keep only the 5 most recent backups
        self.cleanup_old_backups()?;

        Ok(())
    }

    fn cleanup_old_backups(&self) -> Result<()> {
        let backup_dir = self.phloem_dir.join("backups");
        let mut backups: Vec<_> = fs::read_dir(backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("PHLOEM_"))
            .collect();

        // Sort by modification time (newest first)
        backups.sort_by_key(|entry| {
            entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        backups.reverse();

        // Remove all but the 5 most recent
        for backup in backups.iter().skip(5) {
            if let Err(e) = fs::remove_file(backup.path()) {
                log::warn!("Failed to remove old backup: {e}");
            }
        }

        Ok(())
    }

    fn update_existing_section(&self, content: &str, section: &str, new_content: &str) -> String {
        let section_header = format!("### {section}");
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_section = false;
        let mut section_found = false;

        for line in lines {
            if line.starts_with("### ") {
                if in_section {
                    // End of our section, add new content
                    let timestamp = format!("Last updated: {}", Utc::now().format("%Y-%m-%d"));
                    result.push(timestamp);
                    result.push(new_content.to_string());
                    result.push(String::new());
                }
                in_section = line == section_header;
                if in_section {
                    section_found = true;
                }
                result.push(line.to_string());
            } else if in_section && line.starts_with("Last updated:") {
                // Skip old timestamp, we'll add a new one
                continue;
            } else if !in_section {
                result.push(line.to_string());
            }
        }

        // If we were in the section at the end of file
        if in_section && section_found {
            let timestamp = format!("Last updated: {}", Utc::now().format("%Y-%m-%d"));
            result.push(timestamp);
            result.push(new_content.to_string());
        }

        result.join("\n")
    }

    fn add_new_section(&self, content: &str, section: &str, new_content: &str) -> String {
        let section_content = format!(
            "\n### {}\nLast updated: {}\n{}\n",
            section,
            Utc::now().format("%Y-%m-%d"),
            new_content
        );

        // Add to the Command Patterns section
        if let Some(pos) = content.find("## Recent Context") {
            let mut result = content.to_string();
            result.insert_str(pos, &section_content);
            result
        } else {
            format!("{content}{section_content}")
        }
    }
}
