// src/cli/mod.rs
//!
//! # Command-Line Interface (CLI) Module
//!
//! This module serves as the entry point for all command-line interface (CLI) logic in the application.
//! It provides argument parsing, CLI workflow execution, and re-exports the most important CLI types and functions.
//!
//! ## Organization
//! - [`options`]: Contains argument parsing, config conversion, and CLI workflow logic.
//!
//! ## Re-exports
//! - [`run_cli_mode`]: Entrypoint for running CLI workflows.
//! - [`CliOptions`]: Struct for parsing CLI arguments.
//! - [`CliModeOptions`]: Struct for internal CLI mode configuration.
//!
//! ## Example
//! ```rust
//! use aibundle::cli::{CliOptions, run_cli_mode};
//! # tokio_test::block_on(async {
//! let opts = CliOptions::parse();
//! run_cli_mode(opts.to_cli_mode_options()).await.unwrap();
//! # });
//! ```
//!
//! # Doc Aliases
//! - "cli"
//! - "command-line"
//! - "arguments"
//!
//! ---
//!
//! For more details, see [`options`].
#![doc(alias = "command-line")]
#![doc(alias = "arguments")]

pub mod options;

/// Runs the CLI mode workflow. See [`options::run_cli_mode`] for details.
#[doc(alias = "run-cli")]
pub use options::run_cli_mode;
/// CLI mode options struct. See [`options::CliModeOptions`] for details.
#[doc(alias = "cli-mode-options")]
pub use options::CliModeOptions;
/// CLI argument parsing struct. See [`options::CliOptions`] for details.
#[doc(alias = "cli-options")]
pub use options::CliOptions;

// TODO: Add more CLI utilities as new features are implemented.
