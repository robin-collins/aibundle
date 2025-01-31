# CLI Reference Guide 🖥️

## Command Overview

AIBundle's command-line interface provides powerful options for file bundling and processing.

## Basic Syntax

```bash
aibundle [--cli] [OPTIONS] --files <PATTERN>
```

## Core Options 🎯

### File Selection
```bash
# Basic file selection
--files <PATTERN>       # File pattern to match
--source-dir <DIR>      # Source directory (default: current)
--recursive             # Enable recursive search
--no-recursive         # Disable recursive search
```

### Format Options
```bash
--format <FORMAT>       # Output format (xml, markdown, json)
--line-numbers         # Enable line numbers
--no-line-numbers     # Disable line numbers
--pretty              # Pretty-print output
--compact             # Compact output
```

### Output Control
```bash
--out <FILE>           # Write to file
--output-console      # Write to console
--clipboard           # Copy to clipboard (default)
--no-clipboard        # Disable clipboard copy
```

## Advanced Options 🔧

### Filtering
```bash
--ignore <PATTERNS>     # Ignore patterns
--gitignore            # Use .gitignore rules
--no-gitignore        # Ignore .gitignore rules
--binary              # Include binary files
--no-binary           # Exclude binary files
```

### Search Options
```bash
--search <PATTERN>      # Search file contents
--case-sensitive       # Case-sensitive search
--regex               # Use regex pattern
--whole-word          # Match whole words only
```

## Configuration 📝

### Config Management
```bash
--save-config          # Save current options as default
--reset-config        # Reset to default configuration
--config <FILE>       # Use specific config file
```

### Default Settings
```bash
--set-format <FORMAT>  # Set default format
--set-ignore <LIST>    # Set default ignore patterns
--set-recursive       # Set recursive as default
```

## Examples 📚

### Basic Usage
```bash
# Bundle Rust files
aibundle --cli --files "*.rs"

# Multiple file types
aibundle --cli --files "*.{rs,toml}"

# Specific directory
aibundle --cli --files "src/**/*.rs"
```

### Format Examples
```bash
# Markdown with line numbers
aibundle --cli \
  --files "*.rs" \
  --format markdown \
  --line-numbers

# Pretty JSON
aibundle --cli \
  --files "*.rs" \
  --format json \
  --pretty
```

### Output Examples
```bash
# Save to file
aibundle --cli \
  --files "*.rs" \
  --format markdown \
  --out bundle.md

# Console output
aibundle --cli \
  --files "*.rs" \
  --output-console
```

## Option Categories 📊

### Input Options
| Option | Description | Default |
|--------|-------------|---------|
| `--files` | File patterns | Required |
| `--source-dir` | Source directory | Current |
| `--recursive` | Recursive search | true |
| `--ignore` | Ignore patterns | [] |

### Output Options
| Option | Description | Default |
|--------|-------------|---------|
| `--format` | Output format | xml |
| `--line-numbers` | Line numbers | false |
| `--pretty` | Pretty print | false |
| `--out` | Output file | None |

### Filter Options
| Option | Description | Default |
|--------|-------------|---------|
| `--gitignore` | Use .gitignore | true |
| `--binary` | Binary files | false |
| `--search` | Search pattern | None |
| `--regex` | Regex search | false |

## Environment Variables 🌍

### Configuration
```bash
AIBUNDLE_CONFIG       # Config file location
AIBUNDLE_FORMAT       # Default format
AIBUNDLE_IGNORE       # Default ignore patterns
```

### Debug Options
```bash
RUST_LOG             # Log level (debug, trace)
AIBUNDLE_DEBUG       # Enable debug output
AIBUNDLE_TRACE       # Enable trace output
```

## Exit Codes 🚦

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | File access error |
| 4 | Pattern error |

## Best Practices 💡

### Command Structure
1. Order Options
   ```bash
   aibundle --cli \
     --files "*.rs" \
     --format markdown \
     --line-numbers \
     --out output.md
   ```

2. Use Long Options
   ```bash
   # Preferred
   --format markdown

   # Avoid
   -f markdown
   ```

3. Quote Patterns
   ```bash
   # Good
   --files "*.{rs,toml}"

   # Avoid
   --files *.{rs,toml}
   ```
