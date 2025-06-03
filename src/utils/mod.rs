// src/utils/mod.rs
//!
//! # Utilities Module
//!
//! This module provides utility functions for UI layout and formatting, including:
//! - Centering rectangles for popups and modals in the TUI.
//! - Formatting file sizes as human-readable strings.
//!
//! ## Usage
//! These functions are used throughout the TUI and CLI for layout and display purposes.
//!
//! ## Examples
//! ```rust
//! use crate::utils::{centered_rect, human_readable_size};
//! let rect = ratatui::layout::Rect::new(0, 0, 100, 40);
//! let popup = centered_rect(50, 20, rect);
//! assert_eq!(human_readable_size(2048), "2.00 KB");
//! ```

use ratatui::layout::Rect;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use chrono::Local;
use std::env;

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

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

/// Initializes the log file for this run, using hostname, OS, and datetime.
/// Called automatically on first log_event call.
fn init_log_file() -> File {
    let hostname = get_hostname();
    let os = get_os();
    let datetime = Local::now().format("%Y%m%d%H%M%S");
    let filename = format!("{}-{}-{}.log", hostname, os, datetime);
    File::create(filename).expect("Failed to create log file")
}

/// Appends a timestamped message to the log file for this run.
///
/// # Example
/// ```rust
/// crate::utils::log_event("Navigated to /src");
/// ```
pub fn log_event(msg: &str) {
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
}

fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknownhost".to_string())
}

fn get_os() -> String {
    env::consts::OS.to_string()
}

// TODO: Add more utility functions for string formatting, path manipulation, or error handling as needed.
