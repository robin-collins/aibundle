// src/models/mod.rs
//!
//! # Core Data Models
//!
//! Root for all core data structures and configuration types used throughout the application.
//!
//! ## Organization
//! - [`app_config`]: Application and mode configuration, ignore rules, copy stats, file tree nodes, and dependencies.
//! - [`constants`]: Canonical constants for icons, ignored directories, and language mappings.
//! - [`enums`]: Output format enums and related error types.
//!
//! ## Example
//! ```rust
//! use aibundle_modular::models::{AppConfig, OutputFormat, DEFAULT_SELECTION_LIMIT};
//! let config = AppConfig::default();
//! let fmt = OutputFormat::Llm;
//! assert_eq!(DEFAULT_SELECTION_LIMIT, 400);
//! ```
//!
//! # Doc Aliases
//! - "data-structures"
//! - "config-types"
//!
#![doc(alias = "data-structures")]
#![doc(alias = "config-types")]

pub mod app_config;
pub mod constants;
pub mod enums;

/// Application configuration struct. See [`app_config::AppConfig`] for details.
#[doc(alias = "app-config")]
pub use app_config::AppConfig;
/// Copy statistics struct. See [`app_config::CopyStats`] for details.
#[doc(alias = "copy-stats")]
pub use app_config::CopyStats;
/// Ignore configuration struct. See [`app_config::IgnoreConfig`] for details.
#[doc(alias = "ignore-config")]
pub use app_config::IgnoreConfig;
/// Default selection limit constant. See [`constants::DEFAULT_SELECTION_LIMIT`] for details.
#[doc(alias = "selection-limit")]
pub use constants::DEFAULT_SELECTION_LIMIT;
/// Output format enum. See [`enums::OutputFormat`] for details.
#[doc(alias = "output-format")]
pub use enums::OutputFormat;

// TODO: Add more re-exports as new core types are introduced.
