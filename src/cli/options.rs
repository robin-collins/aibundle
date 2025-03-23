use clap::Parser;
use crate::models::{AppConfig, FullConfig, IgnoreConfig, OutputFormat};
use crate::tui::App;
use std::io;
use std::path::{Path, PathBuf};
use std::fs;

use crate::VERSION;

/// Command-line options parsed via clap.
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
    /// Optional positional source directory.
    #[arg(value_name = "SOURCE_DIR", index = 1)]
    pub source_dir_pos: Option<String>,

    /// Write output to file instead of clipboard
    #[arg(short = 'o', long)]
    pub output_file: Option<String>,

    /// Write output to console instead of clipboard
    #[arg(short = 'p', long)]
    pub output_console: bool,

    /// File pattern to match (e.g., "*.rs" or "*.{rs,toml}")
    #[arg(short = 'f', long)]
    pub files: Option<String>,

    /// Search pattern to match file contents (e.g., "test" to match files containing 'test')
    #[arg(short = 's', long)]
    pub search: Option<String>,

    /// Output format to use [possible values: markdown, xml, json, llm]
    #[arg(short = 'm', long, value_parser = ["markdown", "xml", "json", "llm"], default_value = "llm")]
    pub format: String,

    /// Source directory to start from (overridden by the positional SOURCE_DIR if supplied)
    #[arg(short = 'd', long, default_value = ".")]
    pub source_dir: String,

    /// Include subfolders (recursively) in CLI mode
    #[arg(short = 'r', long, default_value = "false")]
    pub recursive: bool,

    /// Show line numbers in output (ignored in JSON format)
    #[arg(short = 'n', long, default_value = "false")]
    pub line_numbers: bool,

    /// Use .gitignore files for filtering
    #[arg(short = 'g', long, default_value = "true")]
    pub gitignore: bool,

    /// Ignore patterns (comma-separated list)
    #[arg(
        short = 'i',
        long,
        use_value_delimiter = true,
        default_value = "default"
    )]
    pub ignore: Vec<String>,

    /// Save current settings to .aibundle.config
    #[arg(short = 'S', long)]
    pub save_config: bool,
}

/// Options for running in CLI mode
pub struct CliModeOptions<'a> {
    pub files_pattern: Option<&'a str>,
    pub source_dir: &'a str,
    pub format: &'a str,
    pub gitignore: bool,
    pub line_numbers: bool,
    pub recursive: bool,
    pub ignore_list: &'a [String],
    pub output_file: Option<&'a str>,
    pub output_console: bool,
}

/// This function runs the tool in CLI mode, bypassing the TUI entirely.
pub fn run_cli_mode(options: CliModeOptions) -> io::Result<()> {
    let mut app = App::new();
    app.current_dir = PathBuf::from(options.source_dir);
    app.ignore_config.use_gitignore = options.gitignore;
    app.output_format = match options.format.to_lowercase().as_str() {
        "markdown" => OutputFormat::Markdown,
        "json" => OutputFormat::Json,
        "llm" => OutputFormat::Llm,
        _ => OutputFormat::Xml,
    };
    app.show_line_numbers = options.line_numbers && app.output_format != OutputFormat::Json;

    // Set the recursive flag based on the CLI parameter.
    app.recursive = options.recursive;

    // Set up ignore patterns from the CLI flag (--ignore)
    app.config.default_ignore = Some(options.ignore_list.to_vec());
    app.ignore_config.extra_ignore_patterns = options.ignore_list.to_vec();
    
    // Override selection limit from CLI config if provided.
    if let Some(cli_conf) = crate::config::load_config()?.cli {
        if let Some(limit) = cli_conf.selection_limit {
            app.selection_limit = limit;
        }
    }

    // Load items based on patterns and recursion setting
    if options.recursive {
        app.expanded_folders = crate::fs::collect_all_subdirs(&app.current_dir, &app.ignore_config)?;
        app.load_items()?;
    } else {
        app.load_items_nonrecursive()?;
    }

    // Apply file pattern filter
    if let Some(pattern) = options.files_pattern {
        app.search_query = pattern.to_string();
        app.update_search();
        // In CLI mode, only select files that match the pattern (exclude directories)
        // For LLM format, we need to keep directories to build a proper structure
        if app.output_format != OutputFormat::Llm {
            app.filtered_items.retain(|p| p.is_file());
        }
    }

    // Select all filtered items
    app.selected_items
        .extend(app.filtered_items.iter().cloned());

    // For LLM format, we need the whole directory structure
    if app.output_format == OutputFormat::Llm {
        // Add all parent directories of selected files
        let mut to_add = std::collections::HashSet::new();
        for path in &app.selected_items {
            let mut current = path.as_path();
            while let Some(parent) = current.parent() {
                if parent.starts_with(&app.current_dir) && parent != app.current_dir {
                    to_add.insert(parent.to_path_buf());
                }
                current = parent;
            }
        }
        app.selected_items.extend(to_add);
    }

    // Generate output
    let output = app.format_selected_items()?;

    // Handle output
    if let Some(file_path) = options.output_file {
        fs::write(file_path, output)?;
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