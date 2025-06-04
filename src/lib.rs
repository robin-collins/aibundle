//! AIBundle Library
//!
//! This library exposes the core functionality of AIBundle for benchmarking and testing purposes.

pub mod cli;
pub mod clipboard;
pub mod config;
pub mod fs;
pub mod models;
pub mod output;
pub mod tui;
pub mod utils;

// Re-export commonly used types for easier access in benchmarks
pub use cli::CliOptions;
pub use models::app_config::{AppConfig, FullConfig, IgnoreConfig, ModeConfig};
