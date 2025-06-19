// src/models/app_config.rs
//!
//! # App Configuration Module
//!
//! Defines configuration structures for the application, including:
//! - [`AppConfig`]: Top-level application configuration for both CLI and TUI modes.
//! - [`ModeConfig`]: Mode-specific configuration (CLI or TUI).
//! - [`FullConfig`]: Aggregates both CLI and TUI configs for serialization.
//! - [`IgnoreConfig`]: Controls ignore rules for file traversal.
//! - [`CopyStats`]: Tracks statistics for copy operations.
//! - [`Node`]: Represents a node in a file tree for LLM output.
//! - [`FileDependencies`]: Tracks dependencies for files.
//!
//! ## Purpose
//! - Manage user and runtime configuration for output, ignore rules, and selection limits.
//! - Provide structures for file tree and dependency analysis.
//!
//! ## Example
//! ```rust
//! use crate::models::app_config::AppConfig;
//! let config = AppConfig::default();
//! assert_eq!(config.default_format, Some("xml".to_string()));
//! ```

use serde::{Deserialize, Serialize};
// use std::collections::HashSet;
// use std::path::PathBuf;
use crate::models::constants::DEFAULT_IGNORED_DIRS;

/// Top-level application configuration for both CLI and TUI modes.
///
/// Holds user preferences and defaults for output format, ignore rules, and selection limits.
///
/// # Fields
///
/// * `default_format` - Default output format (e.g., "xml", "markdown").
/// * `default_gitignore` - Whether to use .gitignore rules by default.
/// * `default_ignore` - List of additional ignore patterns.
/// * `default_line_numbers` - Whether to show line numbers in output.
/// * `default_recursive` - Whether to enable recursive directory traversal.
/// * `selection_limit` - Maximum number of items that can be selected.
///
/// # Examples
/// ```rust
/// use crate::models::app_config::AppConfig;
/// let config = AppConfig::default();
/// assert_eq!(config.default_format, Some("xml".to_string()));
/// ```
#[doc(alias = "config")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// Default output format (e.g., "xml", "markdown").
    pub default_format: Option<String>,
    /// Whether to use .gitignore rules by default.
    pub default_gitignore: Option<bool>,
    /// List of additional ignore patterns.
    pub default_ignore: Option<Vec<String>>,
    /// Whether to show line numbers in output.
    pub default_line_numbers: Option<bool>,
    /// Whether to enable recursive directory traversal.
    pub default_recursive: Option<bool>,
    /// Maximum number of items that can be selected.
    pub selection_limit: Option<usize>,
}

impl Default for AppConfig {
    /// Returns the default application configuration.
    ///
    /// # Returns
    /// * `AppConfig` with sensible defaults for all fields.
    ///
    /// # Example
    /// ```rust
    /// use crate::models::app_config::AppConfig;
    /// let config = AppConfig::default();
    /// assert_eq!(config.default_format, Some("xml".to_string()));
    /// ```
    fn default() -> Self {
        Self {
            default_format: Some("xml".to_string()),
            default_gitignore: Some(true),
            default_ignore: Some(DEFAULT_IGNORED_DIRS.iter().map(|s| s.to_string()).collect()),
            default_line_numbers: Some(false),
            default_recursive: Some(false),
            selection_limit: None,
        }
    }
}

/// Mode-specific configuration (CLI or TUI).
///
/// Used for serializing/deserializing mode-specific settings in config files.
///
/// # Fields
/// * `files` - File pattern(s) to include.
/// * `format` - Output format for this mode.
/// * `out` - Output file path.
/// * `gitignore` - Whether to use .gitignore rules.
/// * `ignore` - Additional ignore patterns.
/// * `line_numbers` - Whether to show line numbers.
/// * `recursive` - Whether to enable recursive traversal.
/// * `source_dir` - Source directory for this mode.
/// * `selection_limit` - Maximum number of items that can be selected.
#[doc(alias = "mode-config")]
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ModeConfig {
    /// File pattern(s) to include.
    pub files: Option<String>,
    /// Output format for this mode.
    pub format: Option<String>,
    /// Output file path.
    pub out: Option<String>,
    /// Whether to use .gitignore rules.
    pub gitignore: Option<bool>,
    /// Additional ignore patterns.
    pub ignore: Option<Vec<String>>,
    /// Whether to show line numbers.
    pub line_numbers: Option<bool>,
    /// Whether to enable recursive traversal.
    pub recursive: Option<bool>,
    /// Source directory for this mode.
    pub source_dir: Option<String>,
    /// Maximum number of items that can be selected.
    pub selection_limit: Option<usize>,
}

/// Aggregates both CLI and TUI configs for serialization.
///
/// Used for loading and saving the full configuration file.
///
/// # Fields
/// * `cli` - CLI mode configuration.
/// * `tui` - TUI mode configuration.
#[doc(alias = "full-config")]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FullConfig {
    /// CLI mode configuration.
    pub cli: Option<ModeConfig>,
    /// TUI mode configuration.
    pub tui: Option<ModeConfig>,
}

/// Controls ignore rules for file traversal.
///
/// Used at runtime to determine which files and directories should be ignored.
///
/// # Fields
/// * `use_default_ignores` - Whether to use the default ignored directories.
/// * `use_gitignore` - Whether to use .gitignore rules.
/// * `include_binary_files` - Whether to include binary files in output.
/// * `extra_ignore_patterns` - Additional ignore patterns.
#[doc(alias = "ignore-config")]
#[derive(Clone)]
pub struct IgnoreConfig {
    /// Whether to use the default ignored directories.
    pub use_default_ignores: bool,
    /// Whether to use .gitignore rules.
    pub use_gitignore: bool,
    /// Whether to include binary files in output.
    pub include_binary_files: bool,
    /// Additional ignore patterns.
    pub extra_ignore_patterns: Vec<String>,
}

impl Default for IgnoreConfig {
    /// Returns the default ignore configuration.
    ///
    /// # Example
    /// ```rust
    /// use crate::models::app_config::IgnoreConfig;
    /// let ignore = IgnoreConfig::default();
    /// assert!(ignore.use_default_ignores);
    /// ```
    fn default() -> Self {
        Self {
            use_default_ignores: true,
            use_gitignore: true,
            include_binary_files: false,
            extra_ignore_patterns: Vec::new(),
        }
    }
}

/// Tracks statistics for copy operations.
///
/// Used to display the number of files and folders processed during copy/export.
///
/// # Fields
/// * `files` - Number of files processed.
/// * `folders` - Number of folders processed.
#[doc(alias = "copy-stats")]
#[derive(Clone)]
pub struct CopyStats {
    /// Number of files processed.
    pub files: usize,
    /// Number of folders processed.
    pub folders: usize,
}

/// Represents a node in a file tree for LLM output.
///
/// Used to build hierarchical representations of files and folders for LLM formatting.
///
/// # Fields
/// * `name` - Name of the file or directory.
/// * `is_dir` - Whether this node is a directory.
/// * `children` - Child nodes (if directory).
#[doc(alias = "file-tree-node")]
pub struct Node {
    /// Name of the file or directory.
    pub name: String,
    /// Whether this node is a directory.
    pub is_dir: bool,
    /// Child nodes (if directory).
    pub children: Option<std::collections::HashMap<String, Node>>,
}

/// Tracks dependencies for files.
///
/// Used for dependency analysis in LLM and other output formats.
///
/// # Fields
/// * `internal_deps` - Internal dependencies (within the project).
/// * `external_deps` - External dependencies (outside the project).
#[doc(alias = "file-dependencies")]
pub struct FileDependencies {
    /// Internal dependencies (within the project).
    pub internal_deps: Vec<String>,
    /// External dependencies (outside the project).
    pub external_deps: Vec<String>,
}

// TODO: Add validation methods for AppConfig and ModeConfig to ensure config integrity.
// TODO: Consider supporting user-defined ignore patterns at runtime via IgnoreConfig.
// TODO: Add serialization for Node and FileDependencies if needed for future features.
