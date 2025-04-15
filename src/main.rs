//!
//! # AIBundle Main Entry Point
//!
//! This is the main entry point for the AIBundle application. It parses CLI arguments, loads configuration, and launches either the CLI or TUI mode.
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
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use std::path::PathBuf;

/// Main entry point. Parses CLI args, loads config, and launches CLI or TUI mode.
fn main() -> io::Result<()> {
    let cli_args = CliOptions::parse();

    // Use the positional SOURCE_DIR if supplied; otherwise, fall back to --source-dir.
    let effective_source_dir = cli_args
        .source_dir_pos
        .unwrap_or(cli_args.source_dir.clone());

    // Load existing config from the user's home directory
    let full_config = config::load_config()?;

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
                files: cli_args.files.clone().or(Some("*".to_string())),
                format: Some(cli_args.format.clone()),
                out: cli_args.output_file.clone().or(Some("".to_string())),
                gitignore: Some(cli_args.gitignore),
                ignore: if cli_args.ignore.len() == 1 && cli_args.ignore[0] == "default" {
                    Some(default_ignore.clone())
                } else {
                    Some(cli_args.ignore.clone())
                },
                line_numbers: Some(cli_args.line_numbers),
                recursive: Some(cli_args.recursive),
                source_dir: Some(cli_args.source_dir.clone()),
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
        let cli_conf = full_config.cli.unwrap_or_default();
        let files = cli_args.files.or(cli_conf.files);
        let format = if !cli_args.format.is_empty() {
            cli_args.format.clone()
        } else {
            cli_conf.format.unwrap_or_else(|| "llm".to_string())
        };
        let source_dir = effective_source_dir.clone();
        let gitignore = cli_args.gitignore || cli_conf.gitignore.unwrap_or(true);
        let line_numbers = cli_args.line_numbers || cli_conf.line_numbers.unwrap_or(false);
        let recursive = cli_args.recursive || cli_conf.recursive.unwrap_or(false);
        let ignore = if !cli_args.ignore.is_empty() {
            cli_args.ignore.clone()
        } else {
            cli_conf.ignore.unwrap_or_default()
        };

        run_cli_mode(CliModeOptions {
            files_pattern: files,
            source_dir,
            format,
            gitignore,
            line_numbers,
            recursive,
            ignore_list: ignore,
            output_file: cli_args.output_file,
            output_console: cli_args.output_console,
        })
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
            PathBuf::from(effective_source_dir.clone()),
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
                app.state.config.default_ignore = Some(ignore.clone());
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
        app.run()?;
        disable_raw_mode()?;
        Ok(())
    }
}

// TODO: Add support for additional CLI subcommands or modes.
// TODO: Add support for config migration/versioning.
