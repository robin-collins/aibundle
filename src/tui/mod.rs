// src/tui/mod.rs
//!
//! # Terminal User Interface (TUI) System
//!
//! Entry point for the TUI system, organizing all submodules and re-exports for the application UI.
//!
//! ## Purpose
//!
//! - Provide access to all TUI components, handlers, state, and views.
//! - Re-export the main [`App`] type for launching the TUI.
//! - Serve as the root for TUI-related documentation and discoverability.
//!
//! ## Organization
//! - [`components`]: UI widgets and reusable components.
//! - [`handlers`]: Event and input handlers.
//! - [`state`]: Application and UI state management.
//! - [`views`]: Layouts and rendering logic.
//!
//! ## Example
//! ```rust
//! use aibundle_modular::tui::App;
//! # use aibundle_modular::models::{AppConfig, IgnoreConfig};
//! # use std::path::PathBuf;
//! let config = AppConfig::default();
//! let start_dir = PathBuf::from(".");
//! let ignore_config = IgnoreConfig::default();
//! let mut app = App::new(config, start_dir, ignore_config).unwrap();
//! app.run().unwrap();
//! ```
//!
//! # Doc Aliases
//! - "terminal-ui"
//! - "user-interface"
//!
#![doc(alias = "terminal-ui")]
#![doc(alias = "user-interface")]
pub mod app;
pub mod components;
pub mod handlers;
pub mod state;
pub mod views;

// App re-export
/// Main TUI application struct. See [`app::App`] for details.
pub use app::App;

// Internal types for TUI system communication
/// Type alias for application results with boxed errors.
#[allow(dead_code)]
pub(crate) type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: Add additional TUI utilities or global types here as needed.
