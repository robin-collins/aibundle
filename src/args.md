# Command Line and TUI Options

## Command Line Options

### Basic Syntax
```bash
aibundle --cli [OPTIONS] --files <PATTERN>
```

### Core Features
- File pattern matching and selection
- Multiple output formats (XML, Markdown, JSON)
- Content search capabilities
- Gitignore integration
- Clipboard or file output
- Configurable ignore patterns

### Essential Options
| Option        | Description                     | Example                  |
|---------------|---------------------------------|--------------------------|
| `--cli`       | Enable CLI mode                 | `--cli`                  |
| `--files`     | File pattern to match           | `--files "*.rs"`         |
| `--format`    | Output format (xml/markdown/json)| `--format markdown`     |
| `--out`       | Write to file instead of clipboard| `--out bundle.md`      |
| `--source-dir`| Source directory (default: current)| `--source-dir ./src`  |

### Output Formats
- XML Format (Default)
- Markdown Format
- JSON Format

### Line Numbers
Add line numbers to your output (except JSON format):
```bash
# XML with line numbers
aibundle --cli --files "*.rs" --line-numbers
```

### File Selection Patterns
| Pattern | Description | Example |
|---------|-------------|---------|
| `*`     | Match any characters | `*.rs` |
| `?`     | Match single character | `test?.rs` |
| `**`    | Match directories recursively | `src/**/*.rs` |
| `{}`    | Group patterns | `*.{rs,toml}` |

## TUI Options

### Selection Methods
- Space: Toggle selection of current item
- * (asterisk): Toggle all visible items

### Search & Filter Features
- Press `/` to enter search mode
- Case-insensitive by default
- Matches file names and paths

### Output Options
- Toggle formats with 'f' key: XML, Markdown, JSON
- Press 'n' to toggle line numbers

### File Operations
- Press 'c' to copy selected files
- Use 'f' to cycle through formats

### Visual Indicators
- ⏳ Operation in progress
- ✅ Operation complete
- ❌ Operation failed

## Best Practices
1. Use XML for AI platforms that expect structured input
2. Use Markdown for documentation and readability
3. Use JSON for programmatic processing
4. Enable line numbers for code review contexts