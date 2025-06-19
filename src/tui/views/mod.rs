// src/tui/views/mod.rs

//!
//! # TUI Views Module
//!
//! Root module for all TUI view components, including help, main, and message views.
//!
//! ## Purpose
//!
//! - Organize and re-export all TUI view components for ergonomic access.
//! - Provide a single entry point for help, main, and message overlays.
//!
//! ## Submodules
//!
//! - [`help_view`]: Help popup/modal view.
//! - [`main_view`]: Main TUI layout view.
//! - [`message_view`]: Temporary message popup view.
//!
//! ## Usage
//!
//! ```rust
//! use crate::tui::views::{HelpView, MainView, MessageView};
//! let main_view = MainView::new();
//! ```
//!
//! # Doc Aliases
//! - "views"
//! - "tui views"

mod help_view;
mod main_view;
mod message_view;

/// Help popup/modal view. See [`help_view::HelpView`] for details.
pub use help_view::HelpView;
/// Main TUI layout view. See [`main_view::MainView`] for details.
pub use main_view::MainView;
/// Temporary message popup view. See [`message_view::MessageView`] for details.
pub use message_view::MessageView;
