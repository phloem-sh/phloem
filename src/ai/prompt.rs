use crate::context::ContextData;

pub struct PromptBuilder;

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build_enhanced_prompt(&self, user_prompt: &str, _context: &ContextData) -> String {
        // This is handled by the Python layer, but we can do some preprocessing here
        format!("User request: {user_prompt}")
    }
}
