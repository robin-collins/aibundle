// src/tui/components/modal.rs
//!
//! # Modal Component
//!
//! Provides the [`Modal`] component for rendering modal dialogs in the TUI, supporting messages, help, copy stats, paging, and custom sizing.
//!
//! ## Purpose
//!
//! - Display overlay dialogs for messages, help, confirmations, and status.
//! - Support paging for long content and custom modal sizing.
//!
//! ## Organization
//!
//! - [`Modal`]: Main modal dialog component.
//! - [`ModalType`]: Enum for modal dialog types.
//!
//! ## Example
//! ```rust
//! use crate::tui::components::Modal;
//! let modal = Modal::new("Hello!".to_string(), 40, 10);
//! # // See Modal::render for rendering example
//! ```
//!
//! # Doc Aliases
//! - "modal-dialog"
//! - "tui-modal"
//!
#![doc(alias = "modal-dialog")]
#![doc(alias = "tui-modal")]

use crate::models::OutputFormat;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::time::Instant;

/// Modal dialog component for displaying messages, help, and status overlays.
///
/// # Fields
/// * `message` - The message or content to display in the modal.
/// * `timestamp` - Timestamp when the modal was created (for timeouts or animations).
/// * `width` - Width of the modal in characters.
/// * `height` - Height of the modal in lines.
/// * `page` - Current page for paginated content.
/// * `modal_type` - Modal type for different behaviors.
///
/// # Examples
/// ```rust
/// use crate::tui::components::Modal;
/// let modal = Modal::new("Test message".to_string(), 40, 10);
/// assert_eq!(modal.width, 40);
/// ```
pub struct Modal {
    /// The message or content to display in the modal.
    pub message: String,
    /// Timestamp when the modal was created (for timeouts or animations).
    #[allow(dead_code)]
    pub timestamp: Instant,
    /// Width of the modal in characters.
    pub width: u16,
    /// Height of the modal in lines.
    pub height: u16,
    /// Current page for paginated content.
    pub page: usize,
    /// Modal type for different behaviors.
    pub modal_type: ModalType,
}

/// Types of modals with different interaction behaviors.
#[derive(Debug, Clone, PartialEq)]
pub enum ModalType {
    /// Standard informational modal (ESC/q to close)
    Info,
    /// Confirmation modal (Y/N to respond)
    Confirmation,
}

impl Modal {
    /// Creates a new [`Modal`] with the given message, width, and height.
    ///
    /// # Arguments
    /// * `message` - The message or content to display.
    /// * `width` - Width of the modal in characters.
    /// * `height` - Height of the modal in lines.
    ///
    /// # Returns
    /// A new [`Modal`] instance.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::Modal;
    /// let modal = Modal::new("Hello!".to_string(), 40, 10);
    /// assert_eq!(modal.width, 40);
    /// ```
    pub fn new(message: String, width: u16, height: u16) -> Self {
        Self {
            message,
            timestamp: Instant::now(),
            width,
            height,
            page: 0,
            modal_type: ModalType::Info,
        }
    }

    /// Creates a new confirmation modal with the given message, width, and height.
    ///
    /// # Arguments
    /// * `message` - The message or content to display.
    /// * `width` - Width of the modal in characters.
    /// * `height` - Height of the modal in lines.
    ///
    /// # Returns
    /// A new [`Modal`] instance with [`ModalType::Confirmation`].
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::Modal;
    /// let modal = Modal::new_confirmation("Confirm?".to_string(), 40, 10);
    /// assert_eq!(modal.modal_type, crate::tui::components::ModalType::Confirmation);
    /// ```
    pub fn new_confirmation(message: String, width: u16, height: u16) -> Self {
        Self {
            message,
            timestamp: Instant::now(),
            width,
            height,
            page: 0,
            modal_type: ModalType::Confirmation,
        }
    }

    /// Creates a modal displaying copy statistics for clipboard operations.
    ///
    /// # Arguments
    /// * `file_count` - Number of files copied.
    /// * `folder_count` - Number of folders copied.
    /// * `line_count` - Number of lines copied.
    /// * `byte_size` - Total size in bytes.
    /// * `format` - Output format used.
    ///
    /// # Returns
    /// A [`Modal`] instance summarizing the copy operation.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::Modal;
    /// use crate::models::OutputFormat;
    /// let modal = Modal::copy_stats(3, 2, 100, 2048, &OutputFormat::Json);
    /// assert!(modal.message.contains("Copied to clipboard"));
    /// ```
    pub fn copy_stats(
        file_count: usize,
        folder_count: usize,
        line_count: usize,
        byte_size: usize,
        format: &OutputFormat,
    ) -> Self {
        let message = format!(
            "Copied to clipboard ({:?})\n\nFiles: {}\nFolders: {}\nLines: {}\nSize: {}",
            format,
            file_count,
            folder_count,
            line_count,
            crate::utils::human_readable_size(byte_size)
        );
        Self::new(message, 60, 8)
    }

    /// Creates a help modal with keyboard shortcuts and navigation info.
    ///
    /// # Returns
    /// A [`Modal`] instance containing help text.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::Modal;
    /// let modal = Modal::help();
    /// assert!(modal.message.contains("Keyboard Shortcuts"));
    /// ```
    pub fn help() -> Self {
        let help_text = "📋 Keyboard Shortcuts
═══════════════════════

🧭 Navigation
─────────────
  ↑/↓        │ Move selection up/down
  PgUp/PgDn  │ Move by 10 items
  Enter      │ Open directory
  Backspace  │ Go to parent directory
  Tab        │ Expand/collapse folder
  Home/End   │ Jump to first/last item

🎯 Selection
────────────
  Space      │ Select/deselect item
  a          │ Select/deselect all items

⚡ Actions
─────────
  c          │ Copy to clipboard
  f          │ Toggle format (XML/MD/JSON/LLM)
  n          │ Toggle line numbers
  /          │ Search (ESC to cancel)

🔍 Filters
──────────
  d          │ Toggle default ignores
  g          │ Toggle .gitignore rules
  b          │ Toggle binary files
  r          │ Toggle recursive mode

🛠️  Other
────────
  h / ?      │ Show/hide this help
  S          │ Save current config
  q          │ Quit (copies if items selected)
  Ctrl+C     │ Quit immediately

💡 Help Navigation
─────────────────
  PgUp/PgDn  │ Scroll help pages
  Esc / q    │ Close help"
            .to_string();

        Self::new(help_text, 68, 32)
    }

    /// Creates a confirmation modal for config file overwrite.
    ///
    /// # Arguments
    /// * `config_path` - Path to the configuration file.
    ///
    /// # Returns
    /// A [`Modal`] instance prompting for overwrite confirmation.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::Modal;
    /// let modal = Modal::config_overwrite_confirmation(std::path::Path::new("/tmp/config.toml"));
    /// assert_eq!(modal.modal_type, crate::tui::components::ModalType::Confirmation);
    /// ```
    pub fn config_overwrite_confirmation(config_path: &std::path::Path) -> Self {
        let message = format!(
            "Configuration file already exists:\n{}\n\nDo you want to overwrite it?\n\n[Y] Yes, overwrite\n[N] No, cancel",
            config_path.display()
        );
        Self::new_confirmation(message, 70, 8)
    }

    /// Returns visible content for the modal with pagination support.
    ///
    /// # Arguments
    /// * `available_height` - Height available for content display.
    ///
    /// # Returns
    /// Tuple of (visible content string, has_more_pages).
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::Modal;
    /// let modal = Modal::new("Line1\nLine2\nLine3".to_string(), 40, 3);
    /// let (content, has_more) = modal.get_visible_content(3);
    /// assert!(content.contains("Line1"));
    /// ```
    #[allow(dead_code)]
    pub fn get_visible_content(&self, available_height: u16) -> (String, bool) {
        let content_height = (available_height - 4) as usize;
        let lines: Vec<&str> = self.message.lines().collect();
        let total_lines = lines.len();
        let total_pages = total_lines.div_ceil(content_height);
        let has_more_pages = total_lines > content_height;
        let start = self.page * content_height;
        let end = (start + content_height).min(total_lines);
        let visible_content = lines[start..end].join("\n");
        let content = if has_more_pages {
            format!(
                "{}\n\nPage {} of {}",
                visible_content,
                self.page + 1,
                total_pages
            )
        } else {
            visible_content
        };
        (content, has_more_pages)
    }

    /// Advances to the next page of content, if available.
    ///
    /// # Arguments
    /// * `available_height` - Height available for content display.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::Modal;
    /// let mut modal = Modal::new("Line1\nLine2\nLine3".to_string(), 40, 3);
    /// modal.next_page(3);
    /// ```
    pub fn next_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
            self.page = (self.page + 1) % total_pages;
        }
    }

    /// Goes to the previous page of content, if available.
    ///
    /// # Arguments
    /// * `available_height` - Height available for content display.
    ///
    /// # Examples
    /// ```rust
    /// use crate::tui::components::Modal;
    /// let mut modal = Modal::new("Line1\nLine2\nLine3".to_string(), 40, 3);
    /// modal.prev_page(3);
    /// ```
    pub fn prev_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
            self.page = (self.page + total_pages - 1) % total_pages;
        }
    }

    /// Renders the modal dialog in the given area.
    ///
    /// # Arguments
    /// * `f` - The TUI frame to render into.
    /// * `area` - The area to render the modal in.
    ///
    /// # Examples
    /// ```rust,ignore
    /// # // Rendering requires a ratatui Frame and area
    /// # use crate::tui::components::Modal;
    /// # let modal = Modal::new("Hello!".to_string(), 40, 10);
    /// # // modal.render(f, area);
    /// ```
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let title = match self.modal_type {
            ModalType::Confirmation => " Confirmation ",
            ModalType::Info => " Message ",
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));

        let text = Text::from(self.message.as_str());
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);

        // Render a clear overlay before the modal
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }
}

// TODO: Add support for modal timeouts or auto-dismiss.
// TODO: Add support for custom modal titles or styles.
