# Command Line and TUI Options

## Command Line Options

### Basic Syntax
```bash
aibundle [OPTIONS] [SOURCE_DIR]
```

### Core Features
- File pattern matching and selection
- Multiple output formats (XML, Markdown, JSON, LLM)
- Content search capabilities
- Gitignore integration
- Clipboard, file, or console output
- Configurable ignore patterns
- Configuration saving

### Essential Options
| Option        | Description                     | Example                  |
|---------------|---------------------------------|--------------------------|
| `--files`, `-f` | File pattern to match           | `--files "*.rs"`         |
| `--format`, `-m` | Output format (xml/markdown/json/llm)| `--format markdown`     |
| `--output-file`, `-o` | Write to file instead of clipboard| `--output-file bundle.md`      |
| `--output-console`, `-p` | Write to console instead of clipboard| `--output-console`      |
| `--source-dir`, `-d` | Source directory (default: current)| `--source-dir ./src`  |
| `--search`, `-s` | Search pattern to match file contents | `--search "function"` |
| `--recursive`, `-r` | Include subfolders recursively | `--recursive` |
| `--line-numbers`, `-n` | Show line numbers in output | `--line-numbers` |
| `--gitignore`, `-g` | Use .gitignore files for filtering (default: true) | `--gitignore` |
| `--ignore`, `-i` | Ignore patterns (comma-separated list) | `--ignore node_modules,target` |
| `--save-config`, `-S` | Save current settings to .aibundle.config | `--save-config` |

### Output Formats
- XML Format (Default)
- Markdown Format
- JSON Format
- LLM Format (Optimized for language models)

### Line Numbers
Add line numbers to your output (except JSON format):
```bash
# XML with line numbers
aibundle --files "*.rs" --line-numbers
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
- Press 'f' to cycle through formats: XML, Markdown, JSON, LLM
- Press 'n' to toggle line numbers

### File Operations
- Press 'c' to copy selected files
- Press 'Tab' to expand/collapse folders
- Press 'Enter' to navigate into directories

### Configuration Options
- Press 'i' to toggle default ignores
- Press 'g' to toggle gitignore usage
- Press 'b' to toggle binary files inclusion
- Press 's' to save current configuration

### Help and Navigation
- Press 'h' to show help
- Press 'q' to quit (copies selected items to clipboard)

### Visual Indicators
- ⏳ Operation in progress
- ✅ Operation complete
- ❌ Operation failed

## Best Practices
1. Use XML for AI platforms that expect structured input
2. Use Markdown for documentation and readability
3. Use JSON for programmatic processing
4. Use LLM for optimized language model input
5. Enable line numbers for code review contexts
6. Save your preferred configuration with `--save-config`