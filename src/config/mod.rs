use crate::models::{AppConfig, FullConfig};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Returns the config file path in the user's home directory.
pub fn config_file_path() -> io::Result<PathBuf> {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").map(PathBuf::from)
    } else {
        std::env::var("HOME").map(PathBuf::from)
    }
    .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
    Ok(home.join(".aibundle.config.toml"))
}

/// Loads a config file from the user's home directory if present.
pub fn load_config() -> io::Result<FullConfig> {
    let config_path = config_file_path()?;
    if config_path.exists() {
        let contents = fs::read_to_string(&config_path)?;
        let parsed: FullConfig = toml::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("TOML parse error: {e}")))?;
        Ok(parsed)
    } else {
        Ok(FullConfig::default())
    }
}

/// Save configuration to a file
pub fn save_config(config: &AppConfig, file_path: &str) -> io::Result<()> {
    if Path::new(file_path).exists() {
        if !crate::fs::confirm_overwrite(file_path)? {
            println!("Aborted saving configuration.");
            return Ok(());
        }
    }

    let toml_str = toml::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("TOML serialize error: {e}")))?;

    fs::write(file_path, toml_str)?;
    println!("Configuration saved successfully.");
    Ok(())
}
