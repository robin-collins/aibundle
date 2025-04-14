use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OutputFormat {
    Xml,
    Markdown,
    Json,
    #[default]
    Llm,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Xml => write!(f, "XML"),
            OutputFormat::Markdown => write!(f, "Markdown"),
            OutputFormat::Json => write!(f, "JSON"),
            OutputFormat::Llm => write!(f, "LLM"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseOutputFormatError;

impl fmt::Display for ParseOutputFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid output format string")
    }
}

impl FromStr for OutputFormat {
    type Err = ParseOutputFormatError;

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

#[derive(Clone, Debug)]
pub struct IgnoreConfig {
    pub use_default_ignores: bool,
    pub use_gitignore: bool,
    pub include_binary_files: bool,
    pub extra_ignore_patterns: Vec<String>,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            use_default_ignores: true,
            use_gitignore: true,
            include_binary_files: false,
            extra_ignore_patterns: Vec::new(),
        }
    }
}

impl OutputFormat {
    pub fn toggle(&self) -> Self {
        match self {
            OutputFormat::Xml => OutputFormat::Markdown,
            OutputFormat::Markdown => OutputFormat::Json,
            OutputFormat::Json => OutputFormat::Llm,
            OutputFormat::Llm => OutputFormat::Xml,
        }
    }
}
