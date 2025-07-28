use regex::Regex;
use std::collections::HashSet;

pub struct CommandValidator;

impl CommandValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn is_safe_command(&self, command: &str) -> bool {
        let dangerous_patterns = self.get_dangerous_patterns();

        for pattern in dangerous_patterns {
            if pattern.is_match(command) {
                return false;
            }
        }

        true
    }

    pub fn is_valid_syntax(&self, command: &str) -> bool {
        let trimmed = command.trim();

        // Basic checks
        if trimmed.is_empty() {
            return false;
        }

        // Check for balanced quotes
        if !self.has_balanced_quotes(trimmed) {
            return false;
        }

        // Check for balanced parentheses
        if !self.has_balanced_parentheses(trimmed) {
            return false;
        }

        // Check if it looks like a command (starts with alphanumeric or slash)
        if !trimmed.chars().next().unwrap_or(' ').is_alphanumeric() && !trimmed.starts_with('/') {
            return false;
        }

        true
    }

    pub fn sanitize_command(&self, command: &str) -> String {
        // Remove potentially dangerous characters and sequences
        let mut sanitized = command.to_string();

        // Remove null bytes
        sanitized = sanitized.replace('\0', "");

        // Remove excessive whitespace
        sanitized = sanitized.trim().to_string();

        // Limit length
        if sanitized.len() > 1000 {
            sanitized.truncate(1000);
        }

        sanitized
    }

    pub fn extract_command_name(&self, command: &str) -> Option<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_part) = parts.first() {
            // Handle cases like "sudo command" or pipes
            if *first_part == "sudo" && parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                Some(first_part.to_string())
            }
        } else {
            None
        }
    }

    pub fn is_destructive_command(&self, command: &str) -> bool {
        let destructive_commands = self.get_destructive_commands();

        if let Some(cmd_name) = self.extract_command_name(command) {
            destructive_commands.contains(&cmd_name.as_str())
        } else {
            false
        }
    }

    fn get_dangerous_patterns(&self) -> Vec<Regex> {
        let patterns = vec![
            r"rm\s+-rf\s+/",        // rm -rf /
            r"rm\s+-rf\s+\*",       // rm -rf *
            r">\s*/dev/sd[a-z]",    // Write to raw disk
            r"dd.*of=/dev/sd[a-z]", // DD to disk
            r"mkfs\.",              // Format filesystem
            r"fdisk\s+/dev/",       // Disk partitioning
            r"parted\s+/dev/",      // Disk partitioning
            r":\(\)\{.*\}\;",       // Fork bomb pattern
            r"curl.*\|\s*bash",     // Dangerous curl | bash
            r"wget.*\|\s*bash",     // Dangerous wget | bash
            r"chmod\s+777\s+/",     // Dangerous chmod on root
            r"chown.*:.*\s+/",      // Dangerous chown on root
        ];

        patterns
            .into_iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect()
    }

    fn get_destructive_commands(&self) -> HashSet<&'static str> {
        [
            "rm", "rmdir", "dd", "mkfs", "fdisk", "parted", "format", "del", "erase", "shred",
            "wipe", "halt", "shutdown", "reboot", "poweroff",
        ]
        .iter()
        .cloned()
        .collect()
    }

    fn has_balanced_quotes(&self, text: &str) -> bool {
        let mut single_quotes = 0;
        let mut double_quotes = 0;
        let mut escaped = false;

        for ch in text.chars() {
            match ch {
                '\\' if !escaped => escaped = true,
                '\'' if !escaped => single_quotes += 1,
                '"' if !escaped => double_quotes += 1,
                _ => escaped = false,
            }
        }

        single_quotes % 2 == 0 && double_quotes % 2 == 0
    }

    fn has_balanced_parentheses(&self, text: &str) -> bool {
        let mut count = 0;
        let mut escaped = false;
        let mut in_quotes = false;
        let mut quote_char = ' ';

        for ch in text.chars() {
            match ch {
                '\\' if !escaped => escaped = true,
                '\'' | '"' if !escaped && !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                }
                c if !escaped && in_quotes && c == quote_char => {
                    in_quotes = false;
                }
                '(' if !escaped && !in_quotes => count += 1,
                ')' if !escaped && !in_quotes => count -= 1,
                _ => escaped = false,
            }

            if count < 0 {
                return false;
            }
        }

        count == 0
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}
