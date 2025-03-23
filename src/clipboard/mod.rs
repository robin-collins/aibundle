use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::fs;
use std::env::consts::OS;

pub fn copy_to_clipboard(text: &str) -> io::Result<()> {
    if is_wsl() {
        // For WSL2, write to a temporary file and use PowerShell to read it
        let temp_file = std::env::temp_dir().join("aibundle_clipboard_temp.txt");
        fs::write(&temp_file, text)?;

        // Convert Linux path to Windows path
        let windows_path = String::from_utf8(
            Command::new("wslpath")
                .arg("-w")
                .arg(&temp_file)
                .output()?
                .stdout,
        )
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to convert path"))?
        .trim()
        .to_string();

        let ps_command = format!(
            "Get-Content -Raw -Path '{}' | Set-Clipboard",
            windows_path.replace("'", "''")
        );

        let status = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_command])
            .status()?;

        // Clean up temp file
        let _ = fs::remove_file(temp_file);

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to copy to Windows clipboard",
            ));
        }
    } else {
        match OS {
            "windows" => {
                // For Windows, use the same temp file approach
                let temp_file = std::env::temp_dir().join("aibundle_clipboard_temp.txt");
                fs::write(&temp_file, text)?;

                let ps_command = format!(
                    "Get-Content -Raw -Path '{}' | Set-Clipboard",
                    temp_file.to_string_lossy().replace("'", "''")
                );

                let status = Command::new("powershell.exe")
                    .args(["-NoProfile", "-NonInteractive", "-Command", &ps_command])
                    .status()?;

                // Clean up temp file
                let _ = fs::remove_file(temp_file);

                if !status.success() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to copy to clipboard",
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
            .output()?;

        if output.status.success() {
            return String::from_utf8(output.stdout)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"));
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
                .output()?;

            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 in clipboard"))
            } else {
                Ok(String::new())
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

pub fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|version| {
            version.to_lowercase().contains("microsoft") || version.to_lowercase().contains("wsl")
        })
        .unwrap_or(false)
}