#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Xml,
    Markdown,
    Json,
    Llm,
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
