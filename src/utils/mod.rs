// src/utils/mod.rs
//!
//! # Utility Functions Module
//!
//! Provides utility functions for UI layout, formatting, and debugging in the TUI application.
//!
//! ## Purpose
//!
//! - Centering rectangles for popups and modals in the TUI.
//! - Formatting file sizes as human-readable strings.
//! - Logging and debug helpers for selection limit issues.
//!
//! ## Organization
//!
//! - `centered_rect`: Center a rectangle within a parent area.
//! - `human_readable_size`: Format file sizes for display.
//! - `log_event`: (No-op) Logging utility.
//! - `write_selection_limit_debug_log`: Write debug logs for selection limit issues.
//!
//! ## Usage
//!
//! ```rust
//! use aibundle_modular::utils::{centered_rect, human_readable_size};
//! let rect = ratatui::layout::Rect::new(0, 0, 100, 40);
//! let popup = centered_rect(50, 20, rect);
//! assert_eq!(human_readable_size(2048), "2.00 KB");
//! ```
//!
//! # Doc Aliases
//! - "utils"
//! - "utility functions"

use chrono::Local;
use ratatui::layout::Rect;
/// use std::env;
/// use std::fs::File;
/// use std::sync::Mutex;
use std::path::PathBuf;

/// Returns a rectangle centered within the given area, with the specified width and height.
///
/// Used for positioning popups, modals, and overlays in the TUI.
///
/// # Arguments
/// * `width` - The width of the centered rectangle.
/// * `height` - The height of the centered rectangle.
/// * `r` - The area within which to center the rectangle.
///
/// # Returns
/// * `Rect` - The centered rectangle.
///
/// # Examples
/// ```rust
/// let area = ratatui::layout::Rect::new(0, 0, 100, 40);
/// let popup = crate::utils::centered_rect(50, 20, area);
/// assert_eq!(popup.width, 50);
/// assert_eq!(popup.height, 20);
/// ```
#[allow(dead_code)]
pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_width = width.min(r.width);
    let popup_height = height.min(r.height);

    let x_margin = (r.width.saturating_sub(popup_width)) / 2;
    let y_margin = (r.height.saturating_sub(popup_height)) / 2;

    Rect {
        x: r.x + x_margin,
        y: r.y + y_margin,
        width: popup_width,
        height: popup_height,
    }
}

/// Formats a file size in bytes as a human-readable string (e.g., "1.23 MB").
///
/// Used for displaying file and clipboard sizes in the UI.
///
/// # Arguments
/// * `size` - The size in bytes.
///
/// # Returns
/// * `String` - The formatted size string.
///
/// # Examples
/// ```rust
/// assert_eq!(crate::utils::human_readable_size(1024), "1.00 KB");
/// assert_eq!(crate::utils::human_readable_size(123), "123 B");
/// ```
pub fn human_readable_size(size: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    if unit_index == 0 {
        format!("{} {}", size as usize, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Appends a timestamped message to the log file for this run.
///
/// # Example
/// ```rust
/// crate::utils::log_event("Navigated to /src");
/// ```
pub fn log_event(msg: &str) {
    // Logging is disabled by making this function a no-op.
    // To re-enable, uncomment the original lines below.
    let _ = msg; // Keep msg parameter to avoid unused variable warning if re-enabled

    /*
    let mut log_file_guard = LOG_FILE.lock().unwrap();
    if log_file_guard.is_none() {
        *log_file_guard = Some(init_log_file());
    }
    if let Some(ref mut file) = log_file_guard.as_mut() {
        let now = Local::now().format("%Y-%m-%dT%H:%M:%S");
        let line = format!("[{}] {}\n", now, msg);
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
    */
}

/// Writes a debug log for selection limit errors.
pub fn write_selection_limit_debug_log(
    current_selected_items: &std::collections::HashSet<PathBuf>,
    path_triggering_count: &Option<PathBuf>,
    num_items_in_triggering_op: usize,
    num_other_selected_items_before_op: usize,
    selection_limit: usize,
) {
    use std::fs::OpenOptions;
    use std::io::Write;

    let filename = "selection_limit_debug.txt";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .expect("Failed to open debug log file");

    let _ = writeln!(file, "--- Selection Limit Debug Log Entry ({}) ---", Local::now().format("%Y-%m-%d %H:%M:%S"));
    let _ = writeln!(file, "Selection Limit: {}", selection_limit);
    let _ = writeln!(file, "Number of items already selected (before this problematic operation): {}", num_other_selected_items_before_op);
    if let Some(p) = path_triggering_count {
        let _ = writeln!(file, "Path whose processing tried to add items: {}", p.display());
    }
    let _ = writeln!(file, "Number of items this operation tried to add: {}", num_items_in_triggering_op);
    let _ = writeln!(file, "Total items if added: {}\n", num_other_selected_items_before_op + num_items_in_triggering_op);

    let _ = writeln!(file, "List A: Selected Items AT THE MOMENT OF THE ERROR MODAL:");
    if current_selected_items.is_empty() {
        let _ = writeln!(file, "  (none)");
    } else {
        for item in current_selected_items {
            let _ = writeln!(file, "  {}", item.display());
        }
    }
    let _ = writeln!(file, "--- End of Entry ---\n");
}

// TODO: Add more utility functions for string formatting, path manipulation, or error handling as needed.
