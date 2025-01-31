# Basic CLI Usage 📝

## Command Structure

```bash
aibundle --cli --files <PATTERN> [OPTIONS]
```

## Essential Options

| Option | Description | Example |
|--------|-------------|---------|
| `--cli` | Enable CLI mode | `--cli` |
| `--files` | File pattern to match | `--files "*.rs"` |
| `--format` | Output format (xml/markdown/json) | `--format markdown` |
| `--out` | Write to file instead of clipboard | `--out bundle.md` |
| `--source-dir` | Source directory (default: current) | `--source-dir ./src` |

## Basic Examples 🎯

### Copy to Clipboard
```bash
# Copy all Rust files
aibundle --cli --files "*.rs"

# Copy specific files
aibundle --cli --files "main.rs,lib.rs"
```

### Save to File
```bash
# Save as Markdown
aibundle --cli --files "*.rs" --format markdown --out code.md

# Print to console
aibundle --cli --files "*.rs" --output-console
```

### Directory Selection
```bash
# Bundle files from specific directory
aibundle --cli --files "*.rs" --source-dir ./src

# Bundle files from parent directory
aibundle --cli --files "*.rs" --source-dir ../
```

## Output Preview 👀

[Image: Screenshot showing example CLI output with annotations]
*Example output showing file bundling with different options*

## Common Patterns 📋

### Multiple File Types
```bash
# Bundle both Rust and TOML files
aibundle --cli --files "*.{rs,toml}"

# Bundle all source files
aibundle --cli --files "*.{rs,py,js,cpp}"
```

### Specific Files
```bash
# Bundle configuration files
aibundle --cli --files "config.{json,yaml,toml}"

# Bundle main source files
aibundle --cli --files "{main,lib}.rs"
```
