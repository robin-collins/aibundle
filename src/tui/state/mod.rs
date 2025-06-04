// src/tui/state/mod.rs
//!
//! # TUI State Module
//!
//! This is the root module for all TUI state management, including application, search, and selection state.
//! It re-exports the main state types for use throughout the TUI system.
//!
//! ## Submodules
//! - `app_state`: Main application state (files, config, UI, etc.).
//! - `search`: Search state and logic.
//! - `selection`: Selection state and logic.
//!
//! ## Re-exports
//! The most commonly used state types are re-exported for ergonomic access.
//!
//! ## Examples
//! ```rust
//! use crate::tui::state::{AppState, SearchState, SelectionState};
//! let mut app_state = AppState::default();
//! let mut search_state = SearchState::new();
//! let mut selection_state = SelectionState::new();
//! ```

mod app_state;
mod search;
mod selection;

/// Main application state struct. See [`app_state::AppState`] for details.
pub use app_state::AppState;
/// Message type enum for user feedback. See [`app_state::MessageType`] for details.
pub use app_state::MessageType;
/// AppMessage struct for user feedback. See [`app_state::AppMessage`] for details.
/// pub use app_state::AppMessage;
/// Search state struct. See [`search::SearchState`] for details.
pub use search::SearchState;
/// Selection state struct. See [`selection::SelectionState`] for details.
pub use selection::SelectionState;

// TODO: Add more state modules as new TUI features are implemented.
