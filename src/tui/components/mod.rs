// src/tui/components/mod.rs
//!
//! # TUI Components Module
//!
//! Provides the root module for all TUI UI components, including file list, header, modal dialogs, and status bar.
//!
//! ## Purpose
//!
//! - Organize and re-export all reusable UI components for the TUI system.
//! - Serve as the entry point for component imports in the TUI.
//!
//! ## Submodules
//! - [`file_list`]: File/folder list component.
//! - [`header`]: Header bar component.
//! - [`modal`]: Modal dialog component.
//! - [`status_bar`]: Status bar component.
//!
//! ## Example
//! ```rust
//! use crate::tui::components::{FileList, HeaderView, Modal, StatusBar};
//! let file_list = FileList::new();
//! let header = HeaderView::new();
//! let modal = Modal::new("Message".to_string(), 40, 10);
//! let status_bar = StatusBar::new();
//! ```
//!
//! # Doc Aliases
//! - "tui-components"
//! - "ui-widgets"
//!
#![doc(alias = "tui-components")]
#![doc(alias = "ui-widgets")]

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
