// src/models/mod.rs
//!
//! # Models Module
//!
//! This is the root module for all core data structures and configuration types used throughout the application.
//! It re-exports the most important types for convenient access elsewhere in the codebase.
//!
//! ## Submodules
//! - `app_config`: Application and mode configuration, ignore rules, copy stats, file tree nodes, and dependencies.
//! - `constants`: Canonical constants for icons, ignored directories, and language mappings.
//! - `enums`: Output format enums and related error types.
//!
//! ## Re-exports
//! The most commonly used types are re-exported for ergonomic access.
//!
//! ## Examples
//! ```rust
//! use crate::models::{AppConfig, OutputFormat, DEFAULT_SELECTION_LIMIT};
//! let config = AppConfig::default();
//! let fmt = OutputFormat::Llm;
//! assert_eq!(DEFAULT_SELECTION_LIMIT, 400);
//! ```

pub mod app_config;
pub mod constants;
pub mod enums;

/// Application configuration struct. See [`app_config::AppConfig`] for details.
pub use app_config::AppConfig;
/// Copy statistics struct. See [`app_config::CopyStats`] for details.
pub use app_config::CopyStats;
/// Ignore configuration struct. See [`app_config::IgnoreConfig`] for details.
pub use app_config::IgnoreConfig;
/// Default selection limit constant. See [`constants::DEFAULT_SELECTION_LIMIT`] for details.
pub use constants::DEFAULT_SELECTION_LIMIT;
/// Output format enum. See [`enums::OutputFormat`] for details.
pub use enums::OutputFormat;

// TODO: Add more re-exports as new core types are introduced.
