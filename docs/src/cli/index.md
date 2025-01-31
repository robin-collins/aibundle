# Command Line Interface Overview 🖥️

AIBundle's CLI mode provides powerful command-line options for quick file bundling and automation workflows.

## Basic Syntax

```bash
aibundle --cli [OPTIONS] --files <PATTERN>
```

## Core Features 🌟

- 📁 File pattern matching and selection
- 📤 Multiple output formats (XML, Markdown, JSON)
- 🔍 Content search capabilities
- 🎯 Gitignore integration
- 📋 Clipboard or file output
- 🔧 Configurable ignore patterns

## Quick Examples

```bash
# Copy all Rust files to clipboard in XML format
aibundle --cli --files "*.rs"

# Save Python files as Markdown with line numbers
aibundle --cli --files "*.py" --format markdown --line-numbers --out code.md

# Search and bundle files containing "TODO"
aibundle --cli --search "TODO" --format json

# Non-recursive file selection
aibundle --cli --files "*.rs" --no-recursive
```

See the following sections for detailed information about each feature.
