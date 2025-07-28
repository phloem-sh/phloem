use crate::cli::Suggestion;

pub struct ResponseParser;

impl Default for ResponseParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseParser {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_suggestions(&self, suggestions: &[Suggestion]) -> Vec<Suggestion> {
        // Additional validation on the Rust side if needed
        suggestions
            .iter()
            .filter(|s| !s.command.is_empty())
            .cloned()
            .collect()
    }
}
