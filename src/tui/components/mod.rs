// src/tui/components/mod.rs
//!
//! # TUI Components Module
//!
//! This is the root module for all TUI UI components, including file list, header, modal dialogs, and status bar.
//! It re-exports the main component types for use throughout the TUI system.
//!
//! ## Submodules
//! - `file_list`: File/folder list component.
//! - `header`: Header bar component.
//! - `modal`: Modal dialog component.
//! - `status_bar`: Status bar component.
//!
//! ## Re-exports
//! The most commonly used component types are re-exported for ergonomic access.
//!
//! ## Examples
//! ```rust
//! use crate::tui::components::{FileList, HeaderView, Modal, StatusBar};
//! let file_list = FileList::new();
//! let header = HeaderView::new();
//! let modal = Modal::new("Message".to_string(), 40, 10);
//! let status_bar = StatusBar::new();
//! ```

pub mod file_list;
pub mod header;
pub mod modal;
pub mod status_bar;

/// File/folder list component. See [`file_list::FileList`] for details.
pub use file_list::FileList;
/// Header bar component. See [`header::HeaderView`] for details.
pub use header::HeaderView;
/// Modal dialog component. See [`modal::Modal`] for details.
pub use modal::Modal;
/// Status bar component. See [`status_bar::StatusBar`] for details.
pub use status_bar::StatusBar;

// TODO: Add new components here as the TUI grows (e.g., footer, sidebar, etc.).
