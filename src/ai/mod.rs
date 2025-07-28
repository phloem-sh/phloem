pub mod ollama_client;
pub mod prompt;
pub mod response;

pub use ollama_client::OllamaClient;
pub use prompt::PromptBuilder;
pub use response::ResponseParser;
