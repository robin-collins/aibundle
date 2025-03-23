mod format;
mod json;
mod llm;
mod markdown;
mod xml;

pub use format::*;
pub use json::*;
pub use llm::*;
pub use markdown::*;
pub use xml::*;

pub fn get_language_name(extension: &str) -> &'static str {
    match extension {
        "py" => "Python",
        "c" => "C",
        "cpp" => "C++",
        "h" => "C/C++ Header",
        "hpp" => "C++ Header",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "java" => "Java",
        "html" => "HTML",
        "css" => "CSS",
        "php" => "PHP",
        "rb" => "Ruby",
        "go" => "Go",
        "rs" => "Rust",
        "swift" => "Swift",
        "kt" => "Kotlin",
        "sh" => "Shell",
        "md" => "Markdown",
        "json" => "JSON",
        "xml" => "XML",
        "yaml" => "YAML",
        "yml" => "YAML",
        "sql" => "SQL",
        "r" => "R",
        _ => "Plain Text",
    }
}
