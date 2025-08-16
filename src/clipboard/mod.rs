// src/clipboard/mod.rs
//!
//! # Clipboard Utilities
//!
//! This module provides robust, cross-platform clipboard utilities for copying to and reading from the system clipboard.
//! It supports Windows, macOS, Linux (Wayland/X11), and WSL environments, handling Unicode and encoding issues transparently.
//!
//! ## Organization
//! - Clipboard copy and paste (with encoding and platform handling)
//! - WSL and PowerShell integration
//! - Utility detection for environment
//!
//! ## Example
//! ```rust
//! use aibundle::clipboard::{copy_to_clipboard, get_clipboard_contents};
//! copy_to_clipboard("Hello, clipboard!").unwrap();
//! let contents = get_clipboard_contents().unwrap();
//! assert!(contents.contains("Hello"));
//! ```
//!
//! # Doc Aliases
//! - "copy"
//! - "paste"
//! - "wsl"
//!
#![doc(alias = "copy")]
#![doc(alias = "paste")]
#![doc(alias = "wsl")]

use std::env::consts::OS;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// RAII guard to ensure temporary files are cleaned up
struct TempFileGuard {
    path: PathBuf,
}

impl TempFileGuard {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        // Best effort cleanup - ignore errors
        let _ = fs::remove_file(&self.path);
    }
}

/// Copies the given text to the system clipboard, supporting Windows, macOS, Linux (Wayland/X11), and WSL.
///
/// # Arguments
/// * `text` - The text to copy to the clipboard.
///
/// # Returns
/// * `io::Result<()>` - Ok on success, or an error if the operation fails.
///
/// # Panics
/// * Never panics. Returns an error on failure.
///
/// # Examples
/// ```rust
/// crate::clipboard::copy_to_clipboard("Copied!").unwrap();
/// ```
pub fn copy_to_clipboard(text: &str) -> io::Result<()> {
    if is_wsl() {
        // For WSL2, write to a temporary file with explicit UTF-8 BOM and use PowerShell to read it
        let temp_file = std::env::temp_dir().join("aibundle_clipboard_temp.txt");

        // Ensure temp file cleanup with RAII guard
        let _temp_guard = TempFileGuard::new(&temp_file);

        // Add UTF-8 BOM to ensure correct encoding in Windows
        let mut content_with_bom = Vec::new();
        // UTF-8 BOM (EF BB BF)
        content_with_bom.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        content_with_bom.extend_from_slice(text.as_bytes());

        fs::write(&temp_file, content_with_bom)
            .map_err(|e| io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to write temp file for WSL clipboard: {}", e)
            ))?;

        // Convert Linux path to Windows path with error handling
        let wslpath_output = Command::new("wslpath")
            .arg("-w")
            .arg(&temp_file)
            .output()
            .map_err(|e| io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to run wslpath command: {}", e)
            ))?;

        if !wslpath_output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("wslpath command failed with status: {}", wslpath_output.status)
            ));
        }

        let windows_path = String::from_utf8(wslpath_output.stdout)
            .map_err(|e| io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to convert wslpath output to UTF-8: {}", e)
            ))?
            .trim()
            .to_string();

        // Enhanced PowerShell command to ensure proper Unicode handling
        let ps_command = format!(
            "[Console]::InputEncoding = [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
            $content = [System.IO.File]::ReadAllText('{}', [System.Text.Encoding]::UTF8); \
            [System.Windows.Forms.Clipboard]::SetText($content, [System.Windows.Forms.TextDataFormat]::UnicodeText);",
            windows_path.replace("'", "''")
        );

        let status = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                // Add namespace reference for Windows Forms
                &format!("Add-Type -AssemblyName System.Windows.Forms; {}", ps_command)
            ])
            .status()
            .map_err(|e| io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to execute PowerShell command: {}", e)
            ))?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("PowerShell clipboard command failed with exit code: {}", 
                    status.code().unwrap_or(-1))
            ));
        }
    } else {
        match OS {
            "windows" => {
                // For Windows, use the same temp file approach with UTF-8 BOM
                let temp_file = std::env::temp_dir().join("aibundle_clipboard_temp.txt");
                
                // Ensure temp file cleanup with RAII guard
                let _temp_guard = TempFileGuard::new(&temp_file);

                // Add UTF-8 BOM to ensure correct encoding
                let mut content_with_bom = Vec::new();
                content_with_bom.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
                content_with_bom.extend_from_slice(text.as_bytes());

                fs::write(&temp_file, content_with_bom)
                    .map_err(|e| io::Error::new(
                        io::ErrorKind::Other, 
                        format!("Failed to write temp file for Windows clipboard: {}", e)
                    ))?;

                // Enhanced PowerShell command to ensure proper Unicode handling
                let ps_command = format!(
                    "[Console]::InputEncoding = [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
                    $content = [System.IO.File]::ReadAllText('{}', [System.Text.Encoding]::UTF8); \
                    [System.Windows.Forms.Clipboard]::SetText($content, [System.Windows.Forms.TextDataFormat]::UnicodeText);",
                    temp_file.to_string_lossy().replace("'", "''")
                );

                let status = Command::new("powershell.exe")
                    .args([
                        "-NoProfile",
                        "-NonInteractive",
                        "-Command",
                        &format!("Add-Type -AssemblyName System.Windows.Forms; {}", ps_command)
                    ])
                    .status()
                    .map_err(|e| io::Error::new(
                        io::ErrorKind::Other, 
                        format!("Failed to execute PowerShell command: {}", e)
                    ))?;

                if !status.success() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("PowerShell clipboard command failed with exit code: {}", 
                            status.code().unwrap_or(-1))
                    ));
                }
            }
            "macos" => {
                let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;

                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(text.as_bytes())?;
                }
                child.wait()?;
            }
            _ => {
                // Try Wayland first
                let wayland_result = Command::new("wl-copy").stdin(Stdio::piped()).spawn();

                match wayland_result {
                    Ok(mut child) => {
                        if let Some(mut stdin) = child.stdin.take() {
                            stdin.write_all(text.as_bytes())?;
                        }
                        child.wait()?;
                    }
                    Err(_) => {
                        // Fall back to X11 (xclip)
                        let mut child = Command::new("xclip")
                            .arg("-selection")
                            .arg("clipboard")
                            .stdin(Stdio::piped())
                            .spawn()?;

                        if let Some(mut stdin) = child.stdin.take() {
                            stdin.write_all(text.as_bytes())?;
                        }
                        child.wait()?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Retrieves the current contents of the system clipboard as a String.
///
/// Supports Windows, macOS, Linux (Wayland/X11), and WSL.
///
/// # Returns
/// * `io::Result<String>` - The clipboard contents, or an error if the operation fails.
///
/// # Panics
/// * Never panics. Returns an error on failure.
///
/// # Examples
/// ```rust
/// let contents = crate::clipboard::get_clipboard_contents().unwrap();
/// assert!(contents.len() >= 0);
/// ```
pub fn get_clipboard_contents() -> io::Result<String> {
    if is_wsl() {
        // For WSL2, use PowerShell with UTF-8 encoding and error handling
        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[Console]::InputEncoding = [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-Clipboard",
            ])
            .output()
            .map_err(|e| io::Error::new(
                io::ErrorKind::Other, 
                format!("Failed to execute PowerShell Get-Clipboard command: {}", e)
            ))?;

        if output.status.success() {
            return String::from_utf8(output.stdout)
                .map_err(|e| io::Error::new(
                    io::ErrorKind::Other, 
                    format!("Invalid UTF-8 in WSL clipboard contents: {}", e)
                ));
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("PowerShell Get-Clipboard failed with exit code: {}", 
                    output.status.code().unwrap_or(-1))
            ));
        }
    }

    match OS {
        "windows" => {
            let output = Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "[Console]::InputEncoding = [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-Clipboard",
                ])
                .output()
                .map_err(|e| io::Error::new(
                    io::ErrorKind::Other, 
                    format!("Failed to execute PowerShell Get-Clipboard command: {}", e)
                ))?;

            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map_err(|e| io::Error::new(
                        io::ErrorKind::Other, 
                        format!("Invalid UTF-8 in Windows clipboard contents: {}", e)
                    ))
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("PowerShell Get-Clipboard failed with exit code: {}", 
                        output.status.code().unwrap_or(-1))
                ))
            }
        }
        "macos" => {
            let output = Command::new("pbpaste").output()?;
            String::from_utf8(output.stdout)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"))
        }
        _ => {
            // Try Wayland first
            let wayland_result = Command::new("wl-paste").output();
            if let Ok(output) = wayland_result {
                if output.status.success() {
                    return String::from_utf8(output.stdout).map_err(|_| {
                        io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard")
                    });
                }
            }

            // Fall back to X11 (xclip)
            let output = Command::new("xclip")
                .arg("-selection")
                .arg("clipboard")
                .arg("-o")
                .output()?;

            String::from_utf8(output.stdout)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"))
        }
    }
}

/// Returns true if running under Windows Subsystem for Linux (WSL).
///
/// Used to determine clipboard strategy for Linux environments.
///
/// # Returns
/// * `bool` - True if running under WSL, false otherwise.
///
/// # Examples
/// ```rust
/// if crate::clipboard::is_wsl() {
///     println!("Running under WSL");
/// }
/// ```
pub fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|version| {
            version.to_lowercase().contains("microsoft") || version.to_lowercase().contains("wsl")
        })
        .unwrap_or(false)
}

// TODO: Add support for additional clipboard managers if needed (e.g., OSC52 for SSH/tmux).
// TODO: Add error messages for common clipboard failures (e.g., missing xclip/wl-copy).
