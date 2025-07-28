use crate::config::Settings;

pub struct DefaultConfig;

impl DefaultConfig {
    pub fn create_default_config_file() -> String {
        r#"[general]
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
"#
        .to_string()
    }

    pub fn get_default_settings() -> Settings {
        Settings::default()
    }
}
