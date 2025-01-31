# Output Formats 📄

AIBundle supports three output formats, each designed for different use cases and AI platforms.

## Available Formats

### XML Format (Default) 📑
```xml
<file name="src/main.rs">
// Your Rust code here
fn main() {
    println!("Hello, world!");
}
</file>
```

### Markdown Format 📝
````text
```rust:src/main.rs
// Your Rust code here
fn main() {
    println!("Hello, world!");
}
```
````

### JSON Format 🔄
```json
[{
  "type": "file",
  "path": "src/main.rs",
  "binary": false,
  "content": "// Your Rust code here\nfn main() {\n    println!(\"Hello, world!\");\n}"
}]
```

## Format Selection 🎯

```bash
# Default XML format
aibundle --cli --files "*.rs"

# Markdown format
aibundle --cli --files "*.rs" --format markdown

# JSON format
aibundle --cli --files "*.rs" --format json
```

## Line Numbers 🔢

Add line numbers to your output (except JSON format):

```bash
# XML with line numbers
aibundle --cli --files "*.rs" --line-numbers

# Markdown with line numbers
aibundle --cli --files "*.rs" --format markdown --line-numbers
```

Example output with line numbers:
```
     1 | fn main() {
     2 |     println!("Hello, world!");
     3 | }
```

## Format-Specific Features 🛠️

### XML Format
- Hierarchical structure
- Directory nesting support
- Binary file indicators
- Line number integration

### Markdown Format
- Language-specific syntax highlighting
- Clean, readable output
- Ideal for documentation
- File path as code fence info

### JSON Format
- Machine-readable
- Structured data
- Binary file flags
- Path preservation

## Best Practices 💡

1. Use XML for AI platforms that expect structured input
2. Use Markdown for documentation and readability
3. Use JSON for programmatic processing
4. Enable line numbers for code review contexts