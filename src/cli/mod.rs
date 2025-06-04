// src/cli/mod.rs
//!
//! # CLI Module
//!
//! This is the root module for all command-line interface (CLI) logic, including argument parsing and CLI mode execution.
//! It re-exports the main CLI entrypoints for use in the application entrypoint.
//!
//! ## Submodules
//! - `options`: CLI argument parsing, config conversion, and CLI workflow logic.
//!
//! ## Re-exports
//! The most commonly used CLI entrypoints are re-exported for ergonomic access.
//!
//! ## Examples
//! ```rust
//! use crate::cli::{run_cli_mode, CliOptions};
//! let opts = CliOptions::parse();
//! run_cli_mode(opts.to_cli_mode_options()).unwrap();
//! ```

pub mod options;

/// Runs the CLI mode workflow. See [`options::run_cli_mode`] for details.
pub use options::run_cli_mode;
/// CLI mode options struct. See [`options::CliModeOptions`] for details.
pub use options::CliModeOptions;
/// CLI argument parsing struct. See [`options::CliOptions`] for details.
pub use options::CliOptions;

// TODO: Add more CLI utilities as new features are implemented.
