# File Format Reference 📄

## Overview

AIBundle supports three output formats, each optimized for different use cases.

## XML Format 📑

### Basic Structure
```xml
<files>
  <file name="src/main.rs">
    fn main() {
        println!("Hello, world!");
    }
  </file>
  <folder name="src/modules">
    <file name="mod.rs">
      pub mod utils;
    </file>
  </folder>
</files>
```

### Features
- Hierarchical structure
- Directory nesting
- File metadata
- Binary file handling

### Line Numbers
```xml
<file name="main.rs" lines="true">
  <line num="1">fn main() {</line>
  <line num="2">    println!("Hello, world!");</line>
  <line num="3">}</line>
</file>
```

## Markdown Format 📝

### Basic Structure
````text
```rust:src/main.rs
fn main() {
    println!("Hello, world!");
}
```

```rust:src/modules/mod.rs
pub mod utils;
```
````

### Features
- Language-specific syntax highlighting
- Clean, readable output
- Documentation-friendly
- GitHub compatibility

### Line Numbers
````text
```rust:main.rs
1  | fn main() {
2  |     println!("Hello, world!");
3  | }
```
````

## JSON Format 🔄

### Basic Structure
```json
{
  "files": [
    {
      "name": "src/main.rs",
      "type": "file",
      "content": "fn main() {\n    println!(\"Hello, world!\");\n}"
    },
    {
      "name": "src/modules",
      "type": "folder",
      "children": [
        {
          "name": "mod.rs",
          "type": "file",
          "content": "pub mod utils;"
        }
      ]
    }
  ]
}
```

### Features
- Machine-readable
- Structured data
- Easy parsing
- API friendly

## Format Selection 🎯

### Command Line
```bash
# XML format (default)
aibundle --cli --files "*.rs"

# Markdown format
aibundle --cli --files "*.rs" --format markdown

# JSON format
aibundle --cli --files "*.rs" --format json
```

### Configuration
```toml
# .aibundle.config
[package]
default_format = "xml"  # xml, markdown, or json
```

## Special Cases 🔧

### Binary Files
```xml
<!-- XML Format -->
<file name="image.png" binary="true">
  <binary size="1024" type="image/png" />
</file>
```

```json
// JSON Format
{
  "name": "image.png",
  "type": "binary",
  "size": 1024,
  "mime": "image/png"
}
```

### Symbolic Links
```xml
<!-- XML Format -->
<link name="link.rs" target="src/main.rs" />
```

```json
// JSON Format
{
  "name": "link.rs",
  "type": "link",
  "target": "src/main.rs"
}
```

## Format Comparison 📊

### Use Cases
| Format | Best For | Features |
|--------|----------|----------|
| XML | AI/LLM | Structure, Metadata |
| Markdown | Documentation | Readability, Highlighting |
| JSON | Integration | Parsing, API Use |

### Performance
| Format | Size | Processing |
|--------|------|------------|
| XML | Medium | Fast |
| Markdown | Small | Very Fast |
| JSON | Large | Medium |

## Best Practices 💡

### Format Selection
1. AI Integration
   - Use XML for structured data
   - Include metadata
   - Preserve hierarchy

2. Documentation
   - Use Markdown
   - Enable syntax highlighting
   - Include line numbers

3. API Integration
   - Use JSON
   - Include full metadata
   - Maintain structure

### Output Options
```bash
# Pretty-printed XML
aibundle --cli --files "*.rs" --format xml --pretty

# Compact JSON
aibundle --cli --files "*.rs" --format json --compact

# Markdown with line numbers
aibundle --cli --files "*.rs" --format markdown --line-numbers
```

## Format Migration 🔄

### Converting Formats
```bash
# XML to Markdown
aibundle --cli --files "*.rs" --format xml | \
  aibundle convert --to markdown

# Markdown to JSON
aibundle --cli --files "*.rs" --format markdown | \
  aibundle convert --to json
```

### Format Validation
```bash
# Validate XML
aibundle --cli --files "*.rs" --format xml --validate

# Check JSON structure
aibundle --cli --files "*.rs" --format json --validate
```
