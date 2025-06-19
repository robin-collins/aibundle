//!
//! # AIBundle Application
//!
//! AIBundle is a modular code analysis and selection tool for Rust projects, supporting both CLI and TUI modes.
//!
//! ## Purpose
//! This crate provides a unified interface for analyzing, selecting, and exporting code bundles from large codebases. It is designed for developers, researchers, and automation tools requiring precise code extraction, dependency analysis, and project structure visualization.
//!
//! ## Organization
//! - **cli**: Command-line interface logic and argument parsing
//! - **clipboard**: Clipboard integration for copying results
//! - **config**: Configuration loading and management
//! - **fs**: Filesystem traversal and filtering
//! - **models**: Core data structures and configuration types
//! - **output**: Output formatting and serialization
//! - **tui**: Terminal user interface components
//! - **utils**: Utility functions and helpers
//!
//! ## Usage
//! Run the binary with CLI arguments for batch operations, or without arguments for the interactive TUI.
//!
//! ## Examples
//! ```sh
//! # Run in TUI mode
//! aibundle
//!
//! # Run in CLI mode
//! aibundle --files "src/**/*.rs" --format json --output-file out.json
//! ```

mod cli;
mod clipboard;
mod config;
mod fs;
mod models;
mod output;
mod tui;
mod utils;

use crate::models::app_config::{FullConfig, ModeConfig};
use clap::Parser;
use cli::{run_cli_mode, CliModeOptions, CliOptions};
use crossterm::terminal::enable_raw_mode;
use crossterm::{execute, cursor};
use std::io::{self, Write};
use std::path::PathBuf;
use crate::tui::app::{AppRunResult};

/// Main entry point for the AIBundle application.
///
/// Parses CLI arguments, loads configuration, and launches either the CLI or TUI mode.
///
/// # Arguments
///
/// * None (arguments are parsed from the command line)
///
/// # Returns
///
/// * `io::Result<()>` - Returns `Ok(())` on success, or an error if initialization or execution fails.
///
/// # Panics
///
/// * This function does not explicitly panic, but may propagate panics from dependencies.
///
/// # Errors
///
/// * Returns an `io::Error` if configuration loading, file operations, or terminal setup fails.
///
/// # Examples
///
/// ```no_run
/// // Run from the command line:
/// // $ aibundle --files "src/**/*.rs" --format json --output-file out.json
/// ```
#[tokio::main]
async fn main() -> io::Result<()> {
    let cli_args = CliOptions::parse();

    // Use the positional SOURCE_DIR if supplied; otherwise, fall back to --source-dir.
    // Memory optimization: Use as_ref() to avoid unnecessary clone
    let effective_source_dir = cli_args
        .source_dir_pos
        .as_ref()
        .unwrap_or(&cli_args.source_dir)
        .clone(); // Only clone once at the end

    // Load existing config from the user's home directory
    let full_config = config::load_config().await?;

    // Determine CLI mode if any of these flags are provided.
    let use_cli = cli_args.files.is_some()
        || cli_args.output_file.is_some()
        || cli_args.output_console
        || cli_args.save_config;

    if use_cli {
        // If saving config is requested, save a default config file (with both [cli] and [tui] sections)
        if cli_args.save_config {
            // Define our default ignored directories as a Vec<String>
            let default_ignore: Vec<String> = models::constants::DEFAULT_IGNORED_DIRS
                .iter()
                .map(|s| s.to_string())
                .collect();

            let cli_config = ModeConfig {
                files: cli_args.files.as_ref().cloned().or(Some("*".to_string())),
                format: Some(cli_args.format.clone()), // Keep clone for owned field
                out: cli_args
                    .output_file
                    .as_ref()
                    .cloned()
                    .or(Some("".to_string())),
                gitignore: Some(cli_args.gitignore),
                ignore: if cli_args.ignore.len() == 1 && cli_args.ignore[0] == "default" {
                    Some(default_ignore.clone()) // Keep clone for owned field
                } else {
                    Some(cli_args.ignore.clone()) // Keep clone for owned field
                },
                line_numbers: Some(cli_args.line_numbers),
                recursive: Some(cli_args.recursive),
                source_dir: Some(cli_args.source_dir.clone()), // Keep clone for owned field
                selection_limit: Some(models::DEFAULT_SELECTION_LIMIT),
            };
            // For TUI, use the same defaults as CLI
            let tui_config = cli_config.clone();
            let config_to_save = FullConfig {
                cli: Some(cli_config),
                tui: Some(tui_config),
            };
            let toml_str = toml::to_string_pretty(&config_to_save).map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("TOML serialize error: {e}"))
            })?;
            let config_path = config::config_file_path()?;
            std::fs::write(&config_path, toml_str)?;
            println!(
                "Configuration saved successfully to {}.",
                config_path.display()
            );

            // If --save-config was provided without any other CLI options, exit early.
            if cli_args.files.is_none()
                && cli_args.search.is_none()
                && cli_args.output_file.is_none()
                && !cli_args.output_console
            {
                return Ok(());
            }
        }

        // Merge command-line values with the [cli] defaults (command-line wins)
        // Memory optimization: Use references and avoid unnecessary clones
        let cli_conf = full_config.cli.unwrap_or_default();
        let files = cli_args.files.as_ref().cloned().or(cli_conf.files);
        let format = if !cli_args.format.is_empty() {
            &cli_args.format
        } else {
            cli_conf.format.as_deref().unwrap_or("llm")
        };
        let source_dir = &effective_source_dir;
        let gitignore = cli_args.gitignore || cli_conf.gitignore.unwrap_or(true);
        let line_numbers = cli_args.line_numbers || cli_conf.line_numbers.unwrap_or(false);
        let recursive = cli_args.recursive || cli_conf.recursive.unwrap_or(false);
        let ignore = if !cli_args.ignore.is_empty() {
            &cli_args.ignore
        } else {
            cli_conf.ignore.as_deref().unwrap_or(&[])
        };

        run_cli_mode(CliModeOptions {
            files_pattern: files,
            source_dir: source_dir.clone(), // Only clone when passing to owned field
            format: format.to_string(),     // Convert &str to String only when needed
            gitignore,
            line_numbers,
            recursive,
            ignore_list: ignore.to_vec(), // Convert slice to Vec only when needed
            output_file: cli_args.output_file,
            output_console: cli_args.output_console,
        }).await
    } else {
        // Run in TUI mode: start in the effective source directory.
        let default_config = models::AppConfig {
            default_format: None,
            default_gitignore: None,
            default_ignore: None,
            default_line_numbers: None,
            default_recursive: None,
            selection_limit: Some(crate::models::DEFAULT_SELECTION_LIMIT),
        };

        let ignore_config = crate::models::IgnoreConfig::default();

        let mut app = tui::App::new(
            default_config,
            PathBuf::from(&effective_source_dir), // Use reference to avoid clone
            ignore_config,
        )?;

        if let Some(tui_conf) = full_config.tui {
            if let Some(format) = tui_conf.format {
                app.state.output_format = match format.to_lowercase().as_str() {
                    "markdown" => models::OutputFormat::Markdown,
                    "json" => models::OutputFormat::Json,
                    "llm" => models::OutputFormat::Llm,
                    _ => models::OutputFormat::Xml,
                };
            }
            if let Some(git) = tui_conf.gitignore {
                app.state.ignore_config.use_gitignore = git;
            }
            if let Some(ignore) = tui_conf.ignore {
                app.state.ignore_config.extra_ignore_patterns = ignore;
            }
            if let Some(ln) = tui_conf.line_numbers {
                app.state.show_line_numbers = ln;
            }
            // Only override the current directory from saved config when no explicit directory was provided.
            if effective_source_dir == "." {
                if let Some(src) = tui_conf.source_dir {
                    app.state.current_dir = PathBuf::from(src);
                }
            }
            if let Some(limit) = tui_conf.selection_limit {
                app.state.selection_limit = limit;
            }
            // Set the recursive flag from the saved TUI config, if provided.
            if let Some(recursive) = tui_conf.recursive {
                app.state.recursive = recursive;
            }
        }

        enable_raw_mode()?;
        let run_result = app.run()?;

        // The app.run() function now disables raw mode internally right after leaving alternate screen
        // But as a failsafe, we'll also try to reset the terminal with a system command
        if cfg!(unix) {
            let status = std::process::Command::new("stty")
                .arg("sane")
                .status();
            // We'll ignore any errors here, it's just an extra precaution
            let _ = status;
        }

        // No need for additional disable_raw_mode() as it's now done in app.run()

        // Ensure cursor is shown as a final safeguard
        let mut stdout = io::stdout();
        let _ = execute!(stdout, cursor::Show);
        let _ = stdout.flush();

        // Print results
        match run_result {
            AppRunResult::CopyBlockedByLimit => {
                println!("\nWarning: Selection limit was exceeded. No items were copied.");
            }
            AppRunResult::Copied(data) => {
                println!("\nCopied to clipboard:");
                println!("Files copied: {}", data.files);
                println!("Folders copied: {}", data.folders);
                println!("Total lines: {}", data.line_count);
                println!(
                    "Total size: {}",
                    crate::utils::human_readable_size(data.byte_size)
                );
                println!(); // Ensure a final newline for the prompt
            }
            AppRunResult::NoAction => {
                // No action, so no output
            }
        }

        Ok(())
    }
}

// TODO: Add support for additional CLI subcommands or modes.
