// src/tui/mod.rs
//!
//! # TUI Module Root
//!
//! This module is the entry point for the TUI system, organizing all submodules and re-exports for the application UI.
//! It provides access to components, handlers, state, and views, and re-exports the main `App` type for launching the TUI.
//!
//! ## Submodules
//! - `components`: UI widgets and reusable components.
//! - `handlers`: Event and input handlers.
//! - `state`: Application and UI state management.
//! - `views`: Layouts and rendering logic.
//!
//! ## Re-exports
//! - `App`: Main TUI application struct.
//! - `AppResult`: Internal result type for TUI operations.
//!
//! ## Examples
//! ```rust
//! use crate::tui::App;
//! let mut app = App::new(config, start_dir, ignore_config).unwrap();
//! app.run().unwrap();
//! ```
pub mod components;
pub mod handlers;
pub mod state;
pub mod views;

// App re-export
mod app;
/// Main TUI application struct. See [`app::App`] for details.
pub use app::App;

// Internal types for TUI system communication
/// Type alias for application results with boxed errors.
#[allow(dead_code)]
pub(crate) type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: Add additional TUI utilities or global types here as needed.
