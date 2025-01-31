# Quick Start Guide 🚀

Get up and running with AIBundle in minutes! This guide will walk you through installation and basic usage.

## Installation 📥

### Using Cargo (Recommended)

```bash
cargo install aibundle
```

### From Source

```bash
git clone https://github.com/robin-collins/aibundle
cd aibundle
cargo build --release
```

## Basic Usage 🎯

### TUI Mode (Interactive)

1. Launch AIBundle in the current directory:
   ```bash
   aibundle
   ```

2. Navigate using arrow keys ⬆️⬇️
3. Select files with Space
4. Copy to clipboard with 'c'
5. Quit with 'q'

### CLI Mode (Command Line)

Bundle all Rust files in the current directory:
```bash
aibundle --cli --files "*.rs"
```

Output to a file in Markdown format:
```bash
aibundle --cli --files "*.rs" --format markdown --out bundle.md
```

## Common Operations 🛠️

### File Selection
```bash
# Select all Python files
aibundle --cli --files "*.py"

# Select specific files
aibundle --cli --files "main.rs,lib.rs"

# Select files containing text
aibundle --cli --search "TODO"
```

### Output Formats
```bash
# XML format (default)
aibundle --cli --files "*.rs"

# Markdown format
aibundle --cli --files "*.rs" --format markdown

# JSON format
aibundle --cli --files "*.rs" --format json
```

## Next Steps 🎯

- Explore the [TUI Interface](./tui/index.md)
- Learn about [CLI Options](./cli/index.md)
- Configure your [Default Settings](./configuration/default-settings.md)