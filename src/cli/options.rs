use crate::models::{AppConfig, IgnoreConfig, OutputFormat};
use crate::models::app_config::FullConfig;
use crate::ModeConfig;
use crate::tui::App;
use clap::Parser;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::models::constants::VERSION;

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
    /// Convert CLI options to a configuration object
    pub fn to_app_config(&self) -> AppConfig {
        AppConfig {
            default_format: Some(self.format.clone()),
            default_gitignore: Some(self.gitignore),
            default_ignore: Some(self.ignore.clone()),
            default_line_numbers: Some(self.line_numbers),
            default_recursive: Some(self.recursive),
            selection_limit: None,
        }
    }

    /// Convert CLI options to mode config
    pub fn to_mode_config(&self) -> ModeConfig {
        ModeConfig {
            files: self.files.clone(),
            format: Some(self.format.clone()),
            out: self.output_file.clone(),
            gitignore: Some(self.gitignore),
            ignore: Some(self.ignore.clone()),
            line_numbers: Some(self.line_numbers),
            recursive: Some(self.recursive),
            source_dir: Some(self.effective_source_dir()),
            selection_limit: None, // Use default value
        }
    }

    /// Get the effective source directory (considering both positional and named arguments)
    pub fn effective_source_dir(&self) -> String {
        self.source_dir_pos.clone().unwrap_or_else(|| self.source_dir.clone())
    }

    /// Convert CLI options to CLI mode options
    pub fn to_cli_mode_options(&self) -> CliModeOptions {
        CliModeOptions {
            files_pattern: self.files.clone(),
            source_dir: self.effective_source_dir(),
            format: self.format.clone(),
            gitignore: self.gitignore,
            line_numbers: self.line_numbers,
            recursive: self.recursive,
            ignore_list: self.ignore.clone(),
            output_file: self.output_file.clone(),
            output_console: self.output_console,
        }
    }
}

/// Load configuration and merge with CLI options
pub fn load_merged_config(cli_opts: &CliOptions) -> io::Result<FullConfig> {
    let mut config = load_config()?;

    // If saving is requested, update config with current CLI options
    if cli_opts.save_config {
        // Update CLI mode config
        config.cli = Some(cli_opts.to_mode_config());
    }

    Ok(config)
}

/// Load configuration from file
pub fn load_config() -> io::Result<FullConfig> {
    // Determine config file path
    let config_path = config_file_path()?;

    // Check if config file exists
    if !Path::new(&config_path).exists() {
        return Ok(FullConfig {
            cli: None,
            tui: None,
        });
    }

    // Load and parse config file (simplified implementation)
    // In a real implementation, this would read and parse the file
    Ok(FullConfig {
        cli: None,
        tui: None,
    })
}

/// Get the path to the configuration file
pub fn config_file_path() -> io::Result<PathBuf> {
    // Fixed error handling for environment variable
    let home = match std::env::var("HOME") {
        Ok(path) => path,
        Err(_) => {
            if cfg!(windows) {
                match std::env::var("USERPROFILE") {
                    Ok(path) => path,
                    Err(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "Home directory not found"
                        ));
                    }
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Home directory not found"
                ));
            }
        }
    };

    Ok(PathBuf::from(home).join(".aibundle.config"))
}

/// Create ignore configuration from CLI options
pub fn create_ignore_config(cli_opts: &CliOptions) -> IgnoreConfig {
    let use_default_ignores = cli_opts.ignore.contains(&"default".to_string());

    IgnoreConfig {
        use_default_ignores,
        use_gitignore: cli_opts.gitignore,
        include_binary_files: false, // Default to false, could be a CLI option
        extra_ignore_patterns: cli_opts.ignore.clone(),
    }
}

/// Convert a string format to OutputFormat enum
pub fn string_to_output_format(format: &str) -> OutputFormat {
    match format.to_lowercase().as_str() {
        "markdown" => OutputFormat::Markdown,
        "xml" => OutputFormat::Xml,
        "json" => OutputFormat::Json,
        _ => OutputFormat::Llm,
    }
}

/// Convert CLI options to output format
pub fn get_output_format(cli_opts: &CliOptions) -> OutputFormat {
    string_to_output_format(&cli_opts.format)
}

/// This function runs the tool in CLI mode, bypassing the TUI entirely.
pub fn run_cli_mode(options: CliModeOptions) -> io::Result<()> {
    // 1. Create AppConfig
    let app_config = AppConfig {
        default_format: Some(options.format.clone()),
        default_gitignore: Some(options.gitignore),
        default_ignore: Some(options.ignore_list.clone()),
        default_line_numbers: Some(options.line_numbers),
        default_recursive: Some(options.recursive),
        selection_limit: None, // Selection limit override handled later
    };
    // 2. Create IgnoreConfig
    let ignore_config = IgnoreConfig {
        use_default_ignores: options.ignore_list.contains(&"default".to_string()),
        use_gitignore: options.gitignore,
        include_binary_files: false, // Assuming false for CLI
        extra_ignore_patterns: options.ignore_list.clone(),
    };
    // 3. Create start_dir PathBuf
    let start_dir = PathBuf::from(options.source_dir.clone());

    // 4. Call App::new with arguments and handle Result
    let mut app = App::new(app_config, start_dir.clone(), ignore_config)?;

    // Subsequent operations should now work on the unwrapped `app`
    // No need to change these lines as App has compatibility fields
    // app.current_dir = start_dir; // Set implicitly by App::new
    // app.ignore_config.use_gitignore = options.gitignore; // Set implicitly by App::new
    // app.output_format = ... // Set implicitly by App::new
    // app.show_line_numbers = ... // Set implicitly by App::new
    // app.recursive = options.recursive; // Set implicitly by App::new
    // app.config.default_ignore = Some(options.ignore_list.to_vec()); // Set implicitly by App::new
    // app.ignore_config.extra_ignore_patterns = options.ignore_list.to_vec(); // Set implicitly by App::new

    // Override selection limit from loaded CLI config if provided.
    // Note: load_config might need adjustment to handle potential errors better
    if let Ok(full_config) = crate::config::load_config() {
        if let Some(cli_conf) = full_config.cli {
            if let Some(limit) = cli_conf.selection_limit {
                app.selection_limit = limit;
            }
        }
    }

    // Load items based on patterns and recursion setting
    if app.recursive { // Check app.recursive which was set via AppConfig
        app.expanded_folders =
            crate::fs::collect_all_subdirs(&app.current_dir, &app.ignore_config)?;
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
