use rusqlite::{params, Connection};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
// use chrono::Utc; // Will be used when we add timestamp functionality
use anyhow::Result;

use crate::cli::Suggestion;

pub struct CacheManager {
    connection: Connection,
}

impl CacheManager {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let connection = Connection::open(db_path)?;

        // Initialize schema
        connection.execute_batch(include_str!("../../sql/schema.sql"))?;

        // Run migrations for existing databases
        Self::migrate_database(&connection)?;

        Ok(Self { connection })
    }

    fn migrate_database(connection: &Connection) -> Result<()> {
        // Check if we need to add new columns to existing suggestions table
        let mut stmt = connection.prepare("PRAGMA table_info(suggestions)")?;
        let rows = stmt.query_map([], |row| {
            row.get::<_, String>(1) // column name
        })?;

        let mut has_success_count = false;
        let mut has_success_rate = false;

        for row in rows {
            match row? {
                name if name == "success_count" => has_success_count = true,
                name if name == "success_rate" => has_success_rate = true,
                _ => {}
            }
        }

        // Add missing columns
        if !has_success_count {
            connection.execute(
                "ALTER TABLE suggestions ADD COLUMN success_count INTEGER DEFAULT 0",
                [],
            )?;
        }
        if !has_success_rate {
            connection.execute(
                "ALTER TABLE suggestions ADD COLUMN success_rate REAL DEFAULT 0.5",
                [],
            )?;
        }

        Ok(())
    }

    pub fn get_suggestion(&self, prompt: &str) -> Result<Option<Suggestion>> {
        let prompt_hash = self.hash_prompt(prompt);

        let mut stmt = self.connection.prepare(
            "SELECT suggestion, explanation, confidence, use_count, success_rate FROM suggestions 
             WHERE prompt_hash = ?1 
             AND created_at > datetime('now', '-7 days')
             AND use_count >= 5
             AND success_rate > 0.7
             ORDER BY (success_rate * 0.6 + confidence * 0.4) DESC 
             LIMIT 1",
        )?;

        let result = stmt.query_row([prompt_hash.clone()], |row| {
            Ok(Suggestion {
                command: row.get(0)?,
                explanation: row.get(1)?,
                confidence: row.get(2)?,
            })
        });

        match result {
            Ok(suggestion) => {
                // Update last_used timestamp and use_count
                self.update_suggestion_usage(&prompt_hash)?;
                Ok(Some(suggestion))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn cache_suggestion(&mut self, prompt: &str, suggestion: &Suggestion) -> Result<()> {
        let prompt_hash = self.hash_prompt(prompt);

        // Check if this suggestion already exists
        let existing = self.connection.query_row(
            "SELECT id, use_count, success_count FROM suggestions WHERE prompt_hash = ?1 AND suggestion = ?2",
            params![prompt_hash, suggestion.command],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))
        );

        match existing {
            Ok((id, use_count, success_count)) => {
                // Update existing suggestion
                let success_rate = if use_count > 0 {
                    success_count as f32 / use_count as f32
                } else {
                    0.5
                };

                self.connection.execute(
                    "UPDATE suggestions SET last_used = datetime('now'), confidence = ?1, success_rate = ?2 WHERE id = ?3",
                    params![suggestion.confidence, success_rate, id],
                )?;
            }
            Err(_) => {
                // Insert new suggestion with conservative defaults
                self.connection.execute(
                    "INSERT INTO suggestions 
                     (prompt_hash, prompt, suggestion, explanation, confidence, created_at, last_used, use_count, success_count, success_rate) 
                     VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'), 0, 0, 0.5)",
                    params![
                        prompt_hash,
                        prompt,
                        suggestion.command,
                        suggestion.explanation,
                        suggestion.confidence,
                    ],
                )?;
            }
        }

        Ok(())
    }

    pub fn record_command_execution(
        &mut self,
        command: &str,
        prompt: &str,
        success: bool,
        exit_code: Option<i32>,
    ) -> Result<()> {
        let context_snapshot = self.get_current_environment_snapshot()?;

        self.connection.execute(
            "INSERT INTO history (command, prompt, success, exit_code, context_snapshot) 
             VALUES (?, ?, ?, ?, ?)",
            params![command, prompt, success, exit_code, context_snapshot,],
        )?;

        Ok(())
    }

    pub fn get_recent_commands(&self, limit: usize) -> Result<Vec<String>> {
        let mut stmt = self.connection.prepare(
            "SELECT command FROM history 
             WHERE success = TRUE 
             ORDER BY executed_at DESC 
             LIMIT ?1",
        )?;

        let rows = stmt.query_map([limit], |row| row.get::<_, String>(0))?;

        let mut commands = Vec::new();
        for command in rows {
            commands.push(command?);
        }

        Ok(commands)
    }

    pub fn update_environment(&mut self, key: &str, value: &str) -> Result<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO environment (key, value, updated_at) 
             VALUES (?, ?, datetime('now'))",
            params![key, value],
        )?;

        Ok(())
    }

    pub fn get_environment(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut stmt = self
            .connection
            .prepare("SELECT key, value FROM environment")?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut env = std::collections::HashMap::new();
        for row in rows {
            let (key, value) = row?;
            env.insert(key, value);
        }

        Ok(env)
    }

    pub fn clear_cache(&mut self) -> Result<()> {
        self.connection.execute("DELETE FROM suggestions", [])?;
        self.connection.execute("DELETE FROM history", [])?;
        Ok(())
    }

    pub fn get_cache_stats(&self) -> Result<String> {
        let mut stats = String::new();

        // Total suggestions
        let total: i64 =
            self.connection
                .query_row("SELECT COUNT(*) FROM suggestions", [], |row| row.get(0))?;

        // Cached suggestions (ready for reuse)
        let cached: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM suggestions WHERE use_count >= 5 AND success_rate > 0.7",
            [],
            |row| row.get(0),
        )?;

        // Success rate stats
        let (avg_success_rate, high_success): (f64, i64) = self.connection.query_row(
            "SELECT AVG(success_rate), COUNT(*) FROM suggestions WHERE success_rate > 0.8",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        stats.push_str("Cache Statistics:\n");
        stats.push_str(&format!("- Total suggestions: {total}\n"));
        stats.push_str(&format!(
            "- Ready for reuse: {} ({:.1}%)\n",
            cached,
            if total > 0 {
                cached as f64 / total as f64 * 100.0
            } else {
                0.0
            }
        ));
        stats.push_str(&format!(
            "- Average success rate: {:.1}%\n",
            avg_success_rate * 100.0
        ));
        stats.push_str(&format!("- High success (>80%): {high_success}\n"));

        Ok(stats)
    }

    pub fn prune_old_data(&mut self, days: i32) -> Result<()> {
        // Remove old suggestions
        self.connection.execute(
            "DELETE FROM suggestions WHERE created_at < datetime('now', '-' || ?1 || ' days')",
            [days],
        )?;

        // Remove old history
        self.connection.execute(
            "DELETE FROM history WHERE executed_at < datetime('now', '-' || ?1 || ' days')",
            [days],
        )?;

        Ok(())
    }

    fn hash_prompt(&self, prompt: &str) -> String {
        let mut hasher = DefaultHasher::new();
        prompt.to_lowercase().trim().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn update_suggestion_usage(&self, prompt_hash: &str) -> Result<()> {
        self.connection.execute(
            "UPDATE suggestions 
             SET last_used = datetime('now'), use_count = use_count + 1 
             WHERE prompt_hash = ?1",
            [prompt_hash],
        )?;

        Ok(())
    }

    pub fn record_suggestion_usage(
        &mut self,
        prompt: &str,
        command: &str,
        success: bool,
    ) -> Result<()> {
        let prompt_hash = self.hash_prompt(prompt);

        // Update the suggestion's usage statistics
        let mut stmt = self.connection.prepare(
            "UPDATE suggestions 
             SET use_count = use_count + 1,
                 success_count = success_count + CASE WHEN ?3 THEN 1 ELSE 0 END,
                 success_rate = CAST(success_count + CASE WHEN ?3 THEN 1 ELSE 0 END AS FLOAT) / (use_count + 1),
                 last_used = datetime('now')
             WHERE prompt_hash = ?1 AND suggestion = ?2"
        )?;

        stmt.execute(params![prompt_hash, command, success])?;
        Ok(())
    }

    pub fn get_shell_history(&self) -> Result<Vec<String>> {
        let home = std::env::var("HOME")?;
        let shell = std::env::var("SHELL").unwrap_or_default();

        let history_file = if shell.contains("zsh") {
            format!("{home}/.zsh_history")
        } else if shell.contains("bash") {
            format!("{home}/.bash_history")
        } else {
            return Ok(Vec::new());
        };

        let history_path = std::path::Path::new(&history_file);
        if !history_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(history_path)?;
        let mut commands: Vec<String> = content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }

                // Handle zsh history format (: timestamp:duration;command)
                if line.starts_with(':') {
                    if let Some(semicolon_pos) = line.find(';') {
                        return Some(line[semicolon_pos + 1..].to_string());
                    }
                }

                Some(line.to_string())
            })
            .collect();

        // Get last 100 commands and reverse to get most recent first
        commands.reverse();
        commands.truncate(100);

        Ok(commands)
    }

    fn get_current_environment_snapshot(&self) -> Result<String> {
        let env = self.get_environment()?;
        Ok(serde_json::to_string(&env)?)
    }
}
