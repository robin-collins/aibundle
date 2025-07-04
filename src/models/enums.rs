// src/models/enums.rs
//!
//! # Enums Module
//!
//! Defines enums and error types used throughout the application, including:
//! - [`OutputFormat`]: Supported output formats for file aggregation and formatting.
//! - [`ParseOutputFormatError`]: Error type for parsing output formats from strings.
//!
//! ## Purpose
//! - Provide type-safe handling of output formats and related logic.
//!
//! ## Example
//! ```rust
//! use crate::models::enums::OutputFormat;
//! let fmt = OutputFormat::Json;
//! assert_eq!(fmt.to_string(), "JSON");
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents the supported output formats for file aggregation and formatting.
///
/// Used to select the output format for CLI and TUI operations, and is serializable for config files.
///
/// # Variants
/// * `Xml` - XML output format.
/// * `Markdown` - Markdown output format.
/// * `Json` - JSON output format.
/// * `Llm` - LLM (Large Language Model) prompt format (default).
///
/// # Example
/// ```rust
/// use crate::models::enums::OutputFormat;
/// assert_eq!(OutputFormat::Markdown.to_string(), "Markdown");
/// ```
#[doc(alias = "output-format")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OutputFormat {
    /// XML output format.
    Xml,
    /// Markdown output format.
    Markdown,
    /// JSON output format.
    Json,
    /// LLM (Large Language Model) prompt format (default).
    #[default]
    Llm,
}

impl fmt::Display for OutputFormat {
    /// Formats the OutputFormat as a user-friendly string.
    ///
    /// # Arguments
    /// * `f` - The formatter.
    ///
    /// # Returns
    /// * `fmt::Result` - The result of the formatting operation.
    ///
    /// # Example
    /// ```rust
    /// use crate::models::enums::OutputFormat;
    /// assert_eq!(OutputFormat::Xml.to_string(), "XML");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Xml => write!(f, "XML"),
            OutputFormat::Markdown => write!(f, "Markdown"),
            OutputFormat::Json => write!(f, "JSON"),
            OutputFormat::Llm => write!(f, "LLM"),
        }
    }
}

/// Error type for parsing OutputFormat from a string.
///
/// Used when converting user input or config values to OutputFormat.
///
/// # Example
/// ```rust
/// use crate::models::enums::{OutputFormat, ParseOutputFormatError};
/// use std::str::FromStr;
/// assert!(OutputFormat::from_str("invalid").is_err());
/// ```
#[doc(alias = "parse-output-format-error")]
#[derive(Debug, PartialEq, Eq)]
pub struct ParseOutputFormatError;

impl fmt::Display for ParseOutputFormatError {
    /// Formats the error as a user-friendly string.
    ///
    /// # Example
    /// ```rust
    /// use crate::models::enums::ParseOutputFormatError;
    /// let err = ParseOutputFormatError;
    /// assert_eq!(err.to_string(), "Invalid output format string");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid output format string")
    }
}

impl FromStr for OutputFormat {
    type Err = ParseOutputFormatError;

    /// Parses an OutputFormat from a string, case-insensitive.
    ///
    /// # Arguments
    /// * `s` - The string to parse.
    ///
    /// # Returns
    /// * `Ok(OutputFormat)` if the string matches a known format.
    /// * `Err(ParseOutputFormatError)` if the string is invalid.
    ///
    /// # Example
    /// ```rust
    /// use crate::models::enums::OutputFormat;
    /// use std::str::FromStr;
    /// assert_eq!(OutputFormat::from_str("xml").unwrap(), OutputFormat::Xml);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "xml" => Ok(OutputFormat::Xml),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "json" => Ok(OutputFormat::Json),
            "llm" => Ok(OutputFormat::Llm),
            _ => Err(ParseOutputFormatError),
        }
    }
}

impl OutputFormat {
    /// Cycles to the next output format in the order: Xml → Markdown → Json → Llm → Xml.
    ///
    /// # Returns
    /// * The next OutputFormat variant.
    ///
    /// # Example
    /// ```rust
    /// use crate::models::enums::OutputFormat;
    /// assert_eq!(OutputFormat::Xml.toggle(), OutputFormat::Markdown);
    /// ```
    pub fn toggle(&self) -> Self {
        match self {
            OutputFormat::Xml => OutputFormat::Markdown,
            OutputFormat::Markdown => OutputFormat::Json,
            OutputFormat::Json => OutputFormat::Llm,
            OutputFormat::Llm => OutputFormat::Xml,
        }
    }
}

