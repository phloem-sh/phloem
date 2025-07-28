use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub general: GeneralConfig,
    pub model: ModelConfig,
    pub cache: CacheConfig,
    pub output: OutputConfig,
    pub privacy: PrivacyConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub max_context_size_kb: usize,
    pub recent_commands_limit: usize,
    pub learning_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub model_path: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheConfig {
    pub max_cache_entries: usize,
    pub cache_ttl_hours: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConfig {
    pub show_explanations: bool,
    pub use_colors: bool,
    pub max_suggestions: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivacyConfig {
    pub collect_usage_stats: bool,
    pub share_anonymous_data: bool,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path_static()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let settings: Settings = toml::from_str(&content)?;
            Ok(settings)
        } else {
            // Return default settings if config doesn't exist
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path_static()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;

        Ok(())
    }

    pub fn get_config_path(&self) -> Result<PathBuf> {
        Self::get_config_path_static()
    }

    fn get_config_path_static() -> Result<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

        Ok(home_dir.join(".phloem").join("config.toml"))
    }
}

impl Default for Settings {
    fn default() -> Self {
        let home_dir = dirs::home_dir()
            .map(|h| {
                h.join(".phloem")
                    .join("models")
                    .join("gemma-3n")
                    .display()
                    .to_string()
            })
            .unwrap_or_else(|| "~/.phloem/models/gemma-3n".to_string());

        Self {
            general: GeneralConfig {
                max_context_size_kb: 50,
                recent_commands_limit: 100,
                learning_enabled: true,
            },
            model: ModelConfig {
                model_path: home_dir,
                max_tokens: 100,
                temperature: 0.0,
            },
            cache: CacheConfig {
                max_cache_entries: 1000,
                cache_ttl_hours: 24,
            },
            output: OutputConfig {
                show_explanations: true,
                use_colors: true,
                max_suggestions: 3,
            },
            privacy: PrivacyConfig {
                collect_usage_stats: false,
                share_anonymous_data: false,
            },
        }
    }
}
