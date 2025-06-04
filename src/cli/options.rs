//!
//! # CLI Options Module
//!
//! This module defines the command-line options, argument parsing, and CLI mode logic for the application.
//! It provides conversion utilities for config and output format handling.
//!
//! ## Usage
//! Use `CliOptions` for parsing CLI arguments, and `run_cli_mode` to execute the CLI workflow.
//!
//! ## Examples
//! ```rust
//! use crate::cli::options::{CliOptions, run_cli_mode};
//! let opts = CliOptions::parse();
//! run_cli_mode(opts.to_cli_mode_options()).unwrap();
//! ```

// src/cli/options.rs
use crate::models::app_config::FullConfig;
use crate::models::{AppConfig, IgnoreConfig, OutputFormat};
use crate::tui::App;
use crate::ModeConfig;
use clap::Parser;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::models::constants::VERSION;

/// Command-line options for the application, parsed via clap.
///
/// # Fields
/// * `source_dir_pos` - Optional positional source directory.
/// * `output_file` - Output file path.
/// * `output_console` - Whether to print output to console.
/// * `files` - File pattern(s) to include.
/// * `search` - Search query.
/// * `format` - Output format (markdown, xml, json, llm).
/// * `source_dir` - Source directory (default: ".").
/// * `recursive` - Whether to enable recursive traversal.
/// * `line_numbers` - Whether to show line numbers.
/// * `gitignore` - Whether to use .gitignore rules.
/// * `ignore` - Additional ignore patterns.
/// * `save_config` - Whether to save the current config.
#[derive(Parser, Debug)]
#[command(name = "aibundle", version = VERSION)]
#[command(about = "AIBUNDLE: A CLI & TUI file aggregator and formatter")]
#[command(long_about = "\
A powerful tool for aggregating and formatting files with both CLI and TUI modes.

EXAMPLES:
    aibundle                                                # TUI mode (default, using current folder)
    aibundle d:\\projects                                   # TUI mode, starting in d:\\projects (Windows)
    aibundle /mnt/d/projects                                # TUI mode, starting in /mnt/d/projects (POSIX)
    aibundle --files \"*.rs\"                              # CLI mode, current folder, files that match \"*.rs\"
    aibundle --files \"*.rs\" /mnt/d/projects/rust_aiformat  # CLI mode, starting in specified directory")]
pub struct CliOptions {
    #[arg(value_name = "SOURCE_DIR", index = 1)]
    pub source_dir_pos: Option<String>,

    #[arg(short = 'o', long)]
    pub output_file: Option<String>,

    #[arg(short = 'p', long)]
    pub output_console: bool,

    #[arg(short = 'f', long)]
    pub files: Option<String>,

    #[arg(short = 's', long)]
    pub search: Option<String>,

    #[arg(short = 'm', long, value_parser = ["markdown", "xml", "json", "llm"], default_value = "llm")]
    pub format: String,

    #[arg(short = 'd', long, default_value = ".")]
    pub source_dir: String,

    #[arg(short = 'r', long, default_value = "false")]
    pub recursive: bool,

    #[arg(short = 'n', long, default_value = "false")]
    pub line_numbers: bool,

    #[arg(short = 'g', long, default_value = "true")]
    pub gitignore: bool,

    #[arg(
        short = 'i',
        long,
        use_value_delimiter = true,
        default_value = "default"
    )]
    pub ignore: Vec<String>,

    #[arg(short = 'S', long)]
    pub save_config: bool,
}

/// CLI mode options, derived from `CliOptions` for internal use.
///
/// # Fields
/// * `files_pattern` - File pattern(s) to include.
/// * `source_dir` - Source directory.
/// * `format` - Output format.
/// * `gitignore` - Whether to use .gitignore rules.
/// * `line_numbers` - Whether to show line numbers.
/// * `recursive` - Whether to enable recursive traversal.
/// * `ignore_list` - Additional ignore patterns.
/// * `output_file` - Output file path.
/// * `output_console` - Whether to print output to console.
pub struct CliModeOptions {
    pub files_pattern: Option<String>,
    pub source_dir: String,
    pub format: String,
    pub gitignore: bool,
    pub line_numbers: bool,
    pub recursive: bool,
    pub ignore_list: Vec<String>,
    pub output_file: Option<String>,
    pub output_console: bool,
}

impl CliOptions {
    /// Converts CLI options to an `AppConfig` for use in the application.
    /// Memory optimization: Minimize cloning by using references where possible
    #[allow(dead_code)]
    pub fn to_app_config(&self) -> AppConfig {
        AppConfig {
            default_format: Some(self.format.clone()), // Clone needed for owned field
            default_gitignore: Some(self.gitignore),
            default_ignore: Some(self.ignore.clone()), // Clone needed for owned field
            default_line_numbers: Some(self.line_numbers),
            default_recursive: Some(self.recursive),
            selection_limit: None,
        }
    }

    /// Converts CLI options to a `ModeConfig` for config serialization.
    /// Memory optimization: Use as_ref().cloned() to avoid unnecessary clones
    #[allow(dead_code)]
    pub fn to_mode_config(&self) -> ModeConfig {
        ModeConfig {
            files: self.files.as_ref().cloned(),
            format: Some(self.format.clone()), // Clone needed for owned field
            out: self.output_file.as_ref().cloned(),
            gitignore: Some(self.gitignore),
            ignore: Some(self.ignore.clone()), // Clone needed for owned field
            line_numbers: Some(self.line_numbers),
            recursive: Some(self.recursive),
            source_dir: Some(self.effective_source_dir()),
            selection_limit: None, // Use default value
        }
    }

    /// Returns the effective source directory, preferring the positional argument if present.
    /// Memory optimization: Use references to avoid unnecessary clones
    #[allow(dead_code)]
    pub fn effective_source_dir(&self) -> String {
        self.source_dir_pos
            .as_ref()
            .unwrap_or(&self.source_dir)
            .clone() // Only clone once at the end
    }

    /// Converts CLI options to `CliModeOptions` for running CLI mode.
    /// Memory optimization: Use as_ref().cloned() to avoid unnecessary clones
    #[allow(dead_code)]
    pub fn to_cli_mode_options(&self) -> CliModeOptions {
        CliModeOptions {
            files_pattern: self.files.as_ref().cloned(),
            source_dir: self.effective_source_dir(),
            format: self.format.clone(), // Clone needed for owned field
            gitignore: self.gitignore,
            line_numbers: self.line_numbers,
            recursive: self.recursive,
            ignore_list: self.ignore.clone(), // Clone needed for owned field
            output_file: self.output_file.as_ref().cloned(),
            output_console: self.output_console,
        }
    }
}

/// Converts a string to an `OutputFormat` enum.
///
/// # Arguments
/// * `format` - The format string (e.g., "markdown", "xml").
///
/// # Returns
/// * `OutputFormat` - The corresponding enum variant.
///
/// # Examples
/// ```rust
/// use crate::cli::options::to_output_format;
/// assert_eq!(to_output_format("json"), crate::models::OutputFormat::Json);
/// ```
#[allow(dead_code)]
pub fn to_output_format(format: &str) -> OutputFormat {
    match format.to_lowercase().as_str() {
        "markdown" => OutputFormat::Markdown,
        "xml" => OutputFormat::Xml,
        "json" => OutputFormat::Json,
        _ => OutputFormat::Llm,
    }
}

/// Loads and merges the CLI config with the current options, saving if requested.
///
/// # Arguments
/// * `cli_opts` - The CLI options to merge.
///
/// # Returns
/// * `io::Result<FullConfig>` - The merged configuration.
///
/// # Examples
/// ```rust
/// // Used internally by CLI entrypoint.
/// ```
#[allow(dead_code)]
pub async fn get_merged_config(cli_opts: &CliOptions) -> io::Result<FullConfig> {
    let mut config = crate::config::load_config().await?;
    if cli_opts.save_config {
        config.cli = Some(cli_opts.to_mode_config());
    }
    Ok(config)
}

/// Converts CLI options to an `IgnoreConfig` for file filtering.
///
/// # Arguments
/// * `cli_opts` - The CLI options to convert.
///
/// # Returns
/// * `IgnoreConfig` - The ignore configuration.
///
/// # Examples
/// ```rust
/// // Used internally by CLI entrypoint.
/// ```
#[allow(dead_code)]
pub fn to_ignore_config(cli_opts: &CliOptions) -> IgnoreConfig {
    let use_default_ignores = cli_opts.ignore.iter().any(|s| s == "default");
    IgnoreConfig {
        use_default_ignores,
        use_gitignore: cli_opts.gitignore,
        include_binary_files: false, // Default to false, could be a CLI option
        extra_ignore_patterns: cli_opts.ignore.clone(), // Clone needed for owned field
    }
}

/// Runs the CLI mode workflow, including file loading, filtering, and output generation.
///
/// # Arguments
/// * `options` - The CLI mode options.
///
/// # Returns
/// * `io::Result<()>` - Ok on success, or error if any step fails.
///
/// # Examples
/// ```rust
/// // Used as the main entrypoint for CLI mode.
/// ```
pub async fn run_cli_mode(options: CliModeOptions) -> io::Result<()> {
    // 1. Create AppConfig - Memory optimization: minimize cloning
    let app_config = AppConfig {
        default_format: Some(options.format.clone()), // Clone needed for owned field
        default_gitignore: Some(options.gitignore),
        default_ignore: Some(options.ignore_list.clone()), // Clone needed for owned field
        default_line_numbers: Some(options.line_numbers),
        default_recursive: Some(options.recursive),
        selection_limit: None, // Selection limit override handled later
    };
    // 2. Create IgnoreConfig - Memory optimization: use iterator instead of contains
    let ignore_config = IgnoreConfig {
        use_default_ignores: options.ignore_list.iter().any(|s| s == "default"),
        use_gitignore: options.gitignore,
        include_binary_files: false, // Assuming false for CLI
        extra_ignore_patterns: options.ignore_list.clone(), // Clone needed for owned field
    };
    // 3. Create start_dir PathBuf - Memory optimization: use reference
    let start_dir = PathBuf::from(&options.source_dir);

    // 4. Call App::new with arguments and handle Result - Memory optimization: avoid clone
    let mut app = App::new(app_config, start_dir, ignore_config)?;

    // Override selection limit from loaded CLI config if provided.
    if let Ok(full_config) = crate::config::load_config().await {
        if let Some(cli_conf) = full_config.cli {
            if let Some(limit) = cli_conf.selection_limit {
                app.state.selection_limit = limit;
            }
        }
    }

    // Load items based on patterns and recursion setting
    if app.state.recursive {
        app.state.expanded_folders =
            crate::fs::collect_all_subdirs(&app.state.current_dir, &app.state.ignore_config)?;
        crate::tui::handlers::FileOpsHandler::load_items(&mut app.state)?;
    } else {
        crate::tui::handlers::FileOpsHandler::load_items_nonrecursive(&mut app.state)?;
    }

    // Apply file pattern filter
    if let Some(pattern) = options.files_pattern {
        app.state.search_query = pattern.to_string();
        let mut search_state = crate::tui::state::SearchState::new();
        search_state.search_query = pattern.to_string();
        let _ =
            crate::tui::handlers::FileOpsHandler::update_search(&mut app.state, &mut search_state);
        // In CLI mode, only select files that match the pattern (exclude directories)
        // For LLM format, we need to keep directories to build a proper structure
        if app.state.output_format != OutputFormat::Llm {
            app.state.filtered_items.retain(|p| p.is_file());
        }
    }

    // Select all filtered items
    app.state
        .selected_items
        .extend(app.state.filtered_items.iter().cloned());

    // For LLM format, we need the whole directory structure
    if app.state.output_format == OutputFormat::Llm {
        // Add all parent directories of selected files
        let mut to_add = std::collections::HashSet::new();
        for path in &app.state.selected_items {
            let mut current = path.as_path();
            while let Some(parent) = current.parent() {
                if parent.starts_with(&app.state.current_dir) && parent != app.state.current_dir {
                    to_add.insert(parent.to_path_buf());
                }
                current = parent;
            }
        }
        app.state.selected_items.extend(to_add);
    }

    // Generate output
    let (output, _stats) =
        crate::tui::handlers::ClipboardHandler::format_selected_items(&app.state)?;

    // Handle output
    if let Some(file_path) = options.output_file {
        fs::write(&file_path, output)?;
        println!("Output written to file: {file_path}");
    } else if options.output_console {
        println!("{output}");
    } else {
        // Replace ClipboardContext with our new function
        crate::clipboard::copy_to_clipboard(&output)?;
        println!("Output copied to clipboard");
    }

    Ok(())
}

// TODO: Add support for additional CLI flags (e.g., include-binary, config-path).
// TODO: Add validation for CLI argument combinations.
