use crate::cli::Suggestion;
use crate::context::ContextManager;
use arboard::Clipboard;
use console::{style, Color};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum SelectAction {
    Execute(usize),
    Output(usize),
    Followup(usize),
    Cancel,
}

#[derive(Debug)]
pub enum FormatResult {
    Executed(String),
    Output(String),
    FollowupRequested,
    Static(String),
}

pub struct OutputFormatter {
    use_colors: bool,
}

pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let message = message.to_string();

        let handle = thread::spawn(move || {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut frame_index = 0;

            while running_clone.load(Ordering::Relaxed) {
                eprint!("\r{} {}", frames[frame_index], message);
                io::stderr().flush().unwrap();
                frame_index = (frame_index + 1) % frames.len();
                thread::sleep(Duration::from_millis(100));
            }

            // Clear the spinner line
            eprint!("\r{}\r", " ".repeat(message.len() + 3));
            io::stderr().flush().unwrap();
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    pub fn stop(mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

impl OutputFormatter {
    pub fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }

    pub fn format_suggestions(
        &self,
        suggestions: &[Suggestion],
        show_explanations: bool,
        original_prompt: &str,
        context: &mut ContextManager,
    ) -> FormatResult {
        if suggestions.is_empty() {
            return FormatResult::Static(self.style_text("No suggestions found.", Color::Yellow));
        }

        self.interactive_select(suggestions, show_explanations, original_prompt, context)
    }

    fn interactive_select(
        &self,
        suggestions: &[Suggestion],
        show_explanations: bool,
        original_prompt: &str,
        context: &mut ContextManager,
    ) -> FormatResult {
        let items: Vec<String> = suggestions
            .iter()
            .map(|s| {
                if show_explanations && s.explanation.is_some() {
                    format!("{} - {}", s.command, s.explanation.as_ref().unwrap())
                } else {
                    s.command.clone()
                }
            })
            .collect();

        match self.custom_select(&items) {
            Ok(SelectAction::Execute(index)) => {
                let selected_command = &suggestions[index].command;

                // Ensure we're back to normal terminal mode before printing
                io::stdout().flush().unwrap();
                eprintln!("{selected_command}");

                let mut cmd = if cfg!(target_os = "windows") {
                    let mut cmd = Command::new("cmd");
                    cmd.args(["/C", selected_command]);
                    cmd
                } else {
                    let mut cmd = Command::new("sh");
                    cmd.args(["-c", selected_command]);
                    cmd
                };

                match cmd.status() {
                    Ok(status) => {
                        let success = status.success();

                        // Record feedback for learning
                        if let Err(e) = context.record_suggestion_feedback(
                            original_prompt,
                            selected_command,
                            success,
                        ) {
                            log::warn!("Failed to record suggestion feedback: {e}");
                        }

                        if success {
                            FormatResult::Executed(String::new())
                        } else {
                            FormatResult::Executed(self.format_error(&format!(
                                "Command exited with code: {:?}",
                                status.code()
                            )))
                        }
                    }
                    Err(e) => {
                        // Record execution failure
                        if let Err(err) = context.record_suggestion_feedback(
                            original_prompt,
                            selected_command,
                            false,
                        ) {
                            log::warn!("Failed to record suggestion feedback: {err}");
                        }
                        FormatResult::Executed(
                            self.format_error(&format!("Failed to execute command: {e}")),
                        )
                    }
                }
            }
            Ok(SelectAction::Output(index)) => {
                let selected_command = &suggestions[index].command;

                // Copy to clipboard and show instructions
                match Clipboard::new() {
                    Ok(mut clipboard) => {
                        if clipboard.set_text(selected_command).is_ok() {
                            eprintln!("Command copied to clipboard: {selected_command}");
                            eprintln!("Press Cmd+V (Mac) or Ctrl+V to paste at your prompt");
                        } else {
                            eprintln!("{selected_command}");
                        }
                    }
                    Err(_) => {
                        eprintln!("{selected_command}");
                    }
                }

                FormatResult::Output(String::new())
            }
            Ok(SelectAction::Followup(_index)) => FormatResult::FollowupRequested,
            Ok(SelectAction::Cancel) => {
                FormatResult::Static(self.format_suggestions_static(suggestions, show_explanations))
            }
            Err(_) => {
                FormatResult::Static(self.format_suggestions_static(suggestions, show_explanations))
            }
        }
    }

    // ========================================================================
    // Interactive Selection
    // ========================================================================

    /// Custom selection interface with keyboard navigation
    fn custom_select(&self, items: &[String]) -> Result<SelectAction, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let mut selected = 0;

        let result = self.selection_loop(&mut stdout, items, &mut selected);

        disable_raw_mode()?;
        execute!(stdout, LeaveAlternateScreen)?;
        result
    }

    /// Main selection loop handling user input
    fn selection_loop(
        &self,
        stdout: &mut io::Stdout,
        items: &[String],
        selected: &mut usize,
    ) -> Result<SelectAction, io::Error> {
        loop {
            self.render_menu(stdout, items, *selected)?;

            if let Event::Key(key_event) = event::read()? {
                match self.handle_key_input(key_event.code, selected, items.len()) {
                    Some(action) => return Ok(action),
                    None => continue,
                }
            }
        }
    }

    /// Renders the selection menu
    fn render_menu(
        &self,
        stdout: &mut io::Stdout,
        items: &[String],
        selected: usize,
    ) -> Result<(), io::Error> {
        execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;

        println!("Select command (Enter=run, Tab=output, Esc=follow-up, Esc Esc=exit):\r");
        println!("\r");

        for (i, item) in items.iter().enumerate() {
            if i == selected {
                println!("▶ {}\r", self.style_text(item, Color::Green));
            } else {
                println!("  {item}\r");
            }
        }

        stdout.flush()
    }

    /// Handles keyboard input and returns action if any
    fn handle_key_input(
        &self,
        key_code: KeyCode,
        selected: &mut usize,
        items_len: usize,
    ) -> Option<SelectAction> {
        match key_code {
            KeyCode::Up => {
                *selected = selected.saturating_sub(1);
                None
            }
            KeyCode::Down => {
                if *selected < items_len - 1 {
                    *selected += 1;
                }
                None
            }
            KeyCode::Enter => Some(SelectAction::Execute(*selected)),
            KeyCode::Tab => Some(SelectAction::Output(*selected)),
            KeyCode::Char('f') | KeyCode::Char('F') => Some(SelectAction::Followup(*selected)),
            KeyCode::Esc => self.handle_escape_key(*selected),
            _ => None,
        }
    }

    /// Handles escape key with double-escape detection
    fn handle_escape_key(&self, selected: usize) -> Option<SelectAction> {
        let timeout = Duration::from_millis(300);

        if let Ok(true) = event::poll(timeout) {
            if let Ok(Event::Key(second_key)) = event::read() {
                if matches!(second_key.code, KeyCode::Esc) {
                    return Some(SelectAction::Cancel);
                }
                // Different key after escape - treat as single escape
                return Some(SelectAction::Followup(selected));
            }
        }

        // Single escape (no second key within timeout)
        Some(SelectAction::Followup(selected))
    }

    fn format_suggestions_static(
        &self,
        suggestions: &[Suggestion],
        show_explanations: bool,
    ) -> String {
        let mut output = String::new();

        for (i, suggestion) in suggestions.iter().enumerate() {
            // Command number and text
            let number = format!("{}. ", i + 1);
            output.push_str(&self.style_text(&number, Color::Cyan));
            output.push_str(&self.style_text(&suggestion.command, Color::Green));
            output.push('\n');

            // Explanation if available and requested
            if show_explanations {
                if let Some(explanation) = &suggestion.explanation {
                    let indented = format!("   {explanation}");
                    output.push_str(&self.style_text(&indented, Color::White));
                    output.push('\n');
                }
            }

            // Confidence (only in verbose mode)
            if suggestion.confidence > 0.0 {
                let confidence = format!("   (confidence: {:.1}%)", suggestion.confidence * 100.0);
                output.push_str(&self.style_text(&confidence, Color::Blue));
                output.push('\n');
            }

            if i < suggestions.len() - 1 {
                output.push('\n');
            }
        }

        output
    }

    pub fn format_error(&self, message: &str) -> String {
        format!("{} {}", self.style_text("Error:", Color::Red), message)
    }

    pub fn format_success(&self, message: &str) -> String {
        format!("{} {}", self.style_text("✓", Color::Green), message)
    }

    pub fn format_warning(&self, message: &str) -> String {
        format!("{} {}", self.style_text("⚠", Color::Yellow), message)
    }

    pub fn format_info(&self, message: &str) -> String {
        format!("{} {}", self.style_text("ℹ", Color::Blue), message)
    }

    fn style_text(&self, text: &str, color: Color) -> String {
        if self.use_colors {
            style(text).fg(color).to_string()
        } else {
            text.to_string()
        }
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new(true)
    }
}
