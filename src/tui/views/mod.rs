// src/tui/views/mod.rs

//!
//! # TUI Views Module
//!
//! This is the root module for all TUI view components, including help, main, and message views.
//! It re-exports the main view types for use throughout the TUI system.
//!
//! ## Submodules
//! - `help_view`: Help popup/modal view.
//! - `main_view`: Main TUI layout view.
//! - `message_view`: Temporary message popup view.
//!
//! ## Re-exports
//! The most commonly used view types are re-exported for ergonomic access.
//!
//! ## Examples
//! ```rust
//! use crate::tui::views::{HelpView, MainView, MessageView};
//! let main_view = MainView::new();
//! ```

mod help_view;
mod main_view;
mod message_view;

/// Help popup/modal view. See [`help_view::HelpView`] for details.
pub use help_view::HelpView;
/// Main TUI layout view. See [`main_view::MainView`] for details.
pub use main_view::MainView;
/// Temporary message popup view. See [`message_view::MessageView`] for details.
pub use message_view::MessageView;
