// src/tui/handlers/mod.rs
//!
//! # TUI Handlers Module
//!
//! Root module for all TUI event and state handlers, including clipboard, file operations, keyboard, and search. Provides ergonomic re-exports for handler types used throughout the TUI system.
//!
//! ## Submodules
//! - [`clipboard`]: Clipboard/copy handler.
//! - [`file_ops`]: File and folder operations handler.
//! - [`keyboard`]: Keyboard input handler.
//! - [`search`]: Search input and filtering handler.
//!
//! ## Re-exports
//! The most commonly used handler types are re-exported for ergonomic access.
//!
//! # Examples
//! ```rust
//! use crate::tui::handlers::{ClipboardHandler, FileOpsHandler, KeyboardHandler, SearchHandler};
//! let keyboard = KeyboardHandler::new();
//! ```

mod clipboard;
mod file_ops;
mod keyboard;
mod search;

/// Clipboard/copy handler. See [`clipboard::ClipboardHandler`] for details.
pub use clipboard::ClipboardHandler;
/// File/folder operations handler. See [`file_ops::FileOpsHandler`] for details.
pub use file_ops::FileOpsHandler;
/// Keyboard input handler. See [`keyboard::KeyboardHandler`] for details.
pub use keyboard::KeyboardHandler;
/// Search input/filtering handler. See [`search::SearchHandler`] for details.
pub use search::SearchHandler;
