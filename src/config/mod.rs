// src/config/mod.rs
//!
//! # Configuration Management
//!
//! This module provides functions for loading, saving, and locating the application's configuration file.
//! It supports TOML-based config files for both CLI and TUI modes, and ensures robust serialization and deserialization.
//!
//! ## Organization
//! - Config file path resolution
//! - Async config loading
//! - Synchronous config saving
//!
//! ## Example
//! ```rust
//! use aibundle_modular::config::{load_config, save_config, config_file_path};
//! # tokio_test::block_on(async {
//! let config = load_config().await.unwrap();
//! let path = config_file_path().unwrap();
//! save_config(&config.cli.unwrap(), &path).unwrap();
//! # });
//! ```
//!
//! # Doc Aliases
//! - "configuration"
//! - "settings"
//!
#![doc(alias = "configuration")]
#![doc(alias = "settings")]

use crate::models::app_config::FullConfig;
use crate::models::AppConfig;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs; // Added for async file operations

/// Returns the path to the application's configuration file in the user's home directory.
///
/// # Returns
/// * `io::Result<PathBuf>` - The path to `.aibundle.config.toml`.
///
/// # Errors
/// * Returns an error if the home directory cannot be determined.
///
/// # Examples
/// ```rust
/// let path = crate::config::config_file_path().unwrap();
/// assert!(path.ends_with(".aibundle.config.toml"));
/// ```
pub fn config_file_path() -> io::Result<PathBuf> {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").map(PathBuf::from)
    } else {
        std::env::var("HOME").map(PathBuf::from)
    }
    .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
    Ok(home.join(".aibundle.config.toml"))
}

/// Loads the application's configuration from the config file, or returns defaults if not found.
///
/// # Returns
/// * `io::Result<FullConfig>` - The loaded configuration, or defaults if the file does not exist.
///
/// # Errors
/// * Returns an error if the file cannot be read or parsed.
///
/// # Examples
/// ```rust
/// # tokio_test::block_on(async {
/// let config = crate::config::load_config().await.unwrap();
/// # })
/// ```
pub async fn load_config() -> io::Result<FullConfig> {
    let config_path = config_file_path()?;
    if config_path.exists() {
        let contents = fs::read_to_string(&config_path).await?;
        let parsed: FullConfig = toml::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("TOML parse error: {e}")))?;
        Ok(parsed)
    } else {
        Ok(FullConfig::default())
    }
}

/// Saves the given application configuration to the specified file path.
///
/// Prompts for overwrite confirmation if the file exists.
///
/// # Arguments
/// * `config` - The configuration to save.
/// * `file_path` - The path to save the configuration file to.
///
/// # Returns
/// * `io::Result<()>` - Ok on success, or error if saving fails.
///
/// # Errors
/// * Returns an error if serialization or file writing fails.
///
/// # Examples
/// ```rust
/// let config = crate::models::AppConfig::default();
/// let path = crate::config::config_file_path().unwrap();
/// crate::config::save_config(&config, path.to_str().unwrap()).unwrap();
/// ```
/// Memory optimization: Accept Path reference instead of &str to avoid string operations
#[allow(dead_code)]
pub fn save_config(config: &AppConfig, file_path: &Path) -> io::Result<()> {
    if file_path.exists() && !crate::fs::confirm_overwrite(file_path.to_str().unwrap_or(""))? {
        println!("Aborted saving configuration.");
        return Ok(());
    }

    let toml_str = toml::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("TOML serialize error: {e}")))?;

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { fs::write(file_path, toml_str).await })?;
    println!("Configuration saved successfully.");
    Ok(())
}

// TODO: Add validation for config file contents before saving/loading.
