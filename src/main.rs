use anyhow::Result;
use clap::Parser;
use log::error;

use phloem::{Cli, CommandHandler, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging - only show errors
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
        .init();

    let cli = Cli::parse();

    // Handle version early
    if matches!(cli.command, Some(Commands::Version)) {
        let version_info = format!(
            "phloem {}\nRust version: {}\nPlatform: {}-{}",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_RUST_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        println!("{version_info}");
        return Ok(());
    }

    // Initialize command handler
    let mut handler = match CommandHandler::new() {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to initialize Phloem: {e}");
            eprintln!("Error: Failed to initialize Phloem: {e}");
            eprintln!("Try running 'phloem init' first.");
            std::process::exit(1);
        }
    };

    match cli.command {
        Some(command) => {
            // Handle subcommands
            match handler.handle_command(command).await {
                Ok(output) => println!("{output}"),
                Err(e) => {
                    error!("Command failed: {e}");
                    let error_msg = handler.format_error(&e.to_string());
                    eprintln!("{error_msg}");
                    std::process::exit(1);
                }
            }
        }
        None => {
            if let Some(ref prompt) = cli.prompt {
                // Handle prompt for command generation

                let options = (&cli).into();

                match handler.handle_prompt(prompt, options).await {
                    Ok(suggestions) => {
                        if suggestions.is_empty() {
                            println!(
                                "{}",
                                handler.format_error(
                                    "No suggestions found. Try rephrasing your prompt."
                                )
                            );
                        } else {
                            match handler
                                .format_suggestions(suggestions, cli.explain, prompt)
                                .await
                            {
                                Ok(output) => {
                                    if !output.is_empty() {
                                        println!("{output}");
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to format suggestions: {e}");
                                    eprintln!(
                                        "{}",
                                        handler.format_error(&format!(
                                            "Failed to format suggestions: {e}"
                                        ))
                                    );
                                    std::process::exit(1);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to generate suggestions: {e}");
                        let error_msg = handler.format_error(&format!(
                            "Failed to generate suggestions: {e}. Check that the ML service is properly configured."
                        ));
                        eprintln!("{error_msg}");
                        std::process::exit(1);
                    }
                }
            } else {
                // No prompt provided, show help
                let help = r#"Phloem - Secure, fast command suggestions using local models

Usage:
  phloem [OPTIONS] <PROMPT>
  phloem [COMMAND]

Examples:
  phloem "list running containers"
  phloem "find large files in current directory"
  phloem --explain "git commit with message"

Commands:
  init      Initialize phloem setup
  update    Update model or binary  
  config    Show configuration
  clear     Clear cache and context
  doctor    Run diagnostics
  help      Show this help message

Options:
  -e, --explain       Show detailed explanations
  -n, --suggestions   Number of suggestions to show [default: 3]
      --no-cache      Skip cache and force fresh inference
  -v, --verbose       Verbose output
  -h, --help          Print help

For more information, visit: https://phloem.sh
"#;
                println!("{help}");
            }
        }
    }

    Ok(())
}
