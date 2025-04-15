// src/tui/components/modal.rs
//!
//! # Modal Component
//!
//! This module defines the `Modal` component for rendering modal dialogs in the TUI.
//! It supports messages, help, copy stats, paging, and custom sizing.
//!
//! ## Usage
//! Use `Modal` to display messages, help, or status in a modal overlay.
//!
//! ## Examples
//! ```rust
//! use crate::tui::components::Modal;
//! let modal = Modal::new("Hello!".to_string(), 40, 10);
//! modal.render(f, area);
//! ```

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
pub struct Modal {
    /// The message or content to display in the modal.
    pub message: String,
    /// Timestamp when the modal was created (for timeouts or animations).
    pub timestamp: Instant,
    /// Width of the modal in characters.
    pub width: u16,
    /// Height of the modal in lines.
    pub height: u16,
    /// Current page for paginated content.
    pub page: usize,
}

impl Modal {
    /// Creates a new `Modal` with the given message, width, and height.
    pub fn new(message: String, width: u16, height: u16) -> Self {
        Self {
            message,
            timestamp: Instant::now(),
            width,
            height,
            page: 0,
        }
    }

    /// Creates a modal displaying copy statistics for clipboard operations.
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
    pub fn help() -> Self {
        let help_text = "Keyboard Shortcuts
═════════════════

Navigation
──────────
↑/↓        - Move selection
PgUp/PgDn  - Move by 10 items
Enter      - Open directory
Tab        - Expand/collapse folder

Selection
─────────
Space      - Select/deselect item
*          - Select/deselect all

Actions
───────
c          - Copy to clipboard
f          - Toggle format (XML/MD/JSON/LLM)
n          - Toggle line numbers
/          - Search (ESC to cancel)

Filters
───────
i          - Toggle default ignores
g          - Toggle .gitignore
b          - Toggle binary files

Other
─────
h          - Show this help
q          - Quit (copies if items selected)

Help Navigation
──────────────
PgUp/PgDn  - Scroll help pages
Any key    - Close help"
            .to_string();

        Self::new(help_text, 60, 30)
    }

    /// Returns the visible content for the current page, given the available height.
    /// Returns a tuple of (content, has_more_pages).
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
    pub fn next_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
            self.page = (self.page + 1) % total_pages;
        }
    }

    /// Goes to the previous page of content, if available.
    pub fn prev_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
            self.page = (self.page + total_pages - 1) % total_pages;
        }
    }

    /// Renders the modal dialog in the given area.
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Message ")
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
