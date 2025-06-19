//!
//! # AIBundle Library
//!
//! Core library for the AIBundle modular code analysis and selection tool.
//!
//! ## Purpose
//! This library exposes the main functionality of AIBundle for benchmarking, testing, and integration with other tools. It is intended for advanced code analysis, dependency mapping, and project structure extraction in Rust workspaces.
//!
//! ## Organization
//! - **cli**: CLI argument parsing and logic
//! - **clipboard**: Clipboard integration
//! - **config**: Configuration management
//! - **fs**: Filesystem traversal and filtering
//! - **models**: Core data structures and configuration types
//! - **output**: Output formatting and serialization
//! - **tui**: Terminal user interface components
//! - **utils**: Utility functions
//!
//! ## Usage
//! Import the library in your Rust project to access AIBundle's core features for custom workflows or benchmarking.
//!
//! ## Examples
//! ```rust
//! use aibundle_modular::CliOptions;
//! let opts = CliOptions::parse_from(["aibundle", "--files", "src/**/*.rs"]);
//! assert_eq!(opts.files, Some("src/**/*.rs".to_string()));
//! ```

pub mod cli;
pub mod clipboard;
pub mod config;
pub mod fs;
pub mod models;
pub mod output;
pub mod tui;
pub mod utils;

/// CLI argument options for AIBundle.
///
/// # Examples
///
/// ```rust
/// use aibundle_modular::CliOptions;
/// let opts = CliOptions::parse_from(["aibundle", "--files", "src/**/*.rs"]);
/// assert_eq!(opts.files, Some("src/**/*.rs".to_string()));
/// ```
pub use cli::CliOptions;

/// Application configuration types for AIBundle.
///
/// # Aliases
/// * `AppConfig` - General application configuration
/// * `FullConfig` - Complete configuration for CLI and TUI
/// * `IgnoreConfig` - Ignore pattern configuration
/// * `ModeConfig` - Mode-specific configuration
///
/// # Examples
///
/// ```rust
/// use aibundle_modular::{AppConfig, FullConfig};
/// let app_config = AppConfig { ..Default::default() };
/// let full_config = FullConfig { cli: None, tui: None };
/// ```
pub use models::app_config::{AppConfig, FullConfig, IgnoreConfig, ModeConfig};
