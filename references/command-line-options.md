# AIBundle Command Line Options

*Source: [`src/cli/options.rs`](src/cli/options.rs), [`src/main.rs`](src/main.rs)*

## Introduction

AIBundle is a powerful CLI & TUI file aggregator and formatter that helps you collect, filter, and format files from your projects. It operates in two modes:

- **TUI Mode** (default): Interactive terminal user interface for browsing and selecting files
- **CLI Mode**: Command-line batch processing for automated workflows

The application automatically switches to CLI mode when any CLI-specific options are provided.

## Usage Syntax

```bash
# TUI Mode (default)
aibundle [SOURCE_DIR]

# CLI Mode (triggered by any CLI option)
aibundle [OPTIONS] [SOURCE_DIR]
```

## Command Line Options Reference

### Positional Arguments

#### `SOURCE_DIR`
- **Type**: Optional positional argument
- **Source**: [`CliOptions.source_dir_pos`](src/cli/options.rs:57-58)
- **Description**: Source directory to start from
- **Default**: Current directory (`.`)
- **Examples**:
  ```bash
  aibundle                           # Use current directory
  aibundle /path/to/project          # Use specific directory (Unix/Linux)
  aibundle d:\projects               # Use specific directory (Windows)
  aibundle /mnt/d/projects           # Use specific directory (WSL)
  ```

### File Selection Options

#### `-f, --files <PATTERN>`
- **Type**: String (optional)
- **Source**: [`CliOptions.files`](src/cli/options.rs:66-67)
- **Description**: File pattern(s) to include in selection
- **Default**: None (all files when in CLI mode)
- **Triggers**: CLI mode
- **Examples**:
  ```bash
  aibundle --files "*.rs"            # Only Rust files
  aibundle --files "src/**/*.py"     # Python files in src directory
  aibundle -f "*.{js,ts}"           # JavaScript and TypeScript files
  ```

#### `-s, --search <QUERY>`
- **Type**: String (optional)
- **Source**: [`CliOptions.search`](src/cli/options.rs:69-70)
- **Description**: Search query to filter files
- **Default**: None
- **Examples**:
  ```bash
  aibundle --search "config"         # Files containing "config" in name
  aibundle -s "test"                # Files containing "test" in name
  ```

### Output Options

#### `-o, --output-file <FILE>`
- **Type**: String (optional)
- **Source**: [`CliOptions.output_file`](src/cli/options.rs:60-61)
- **Description**: Write output to specified file instead of clipboard
- **Default**: None (output goes to clipboard)
- **Triggers**: CLI mode
- **Examples**:
  ```bash
  aibundle --files "*.rs" --output-file bundle.md
  aibundle -f "*.py" -o python_code.json
  ```

#### `-p, --output-console`
- **Type**: Boolean flag
- **Source**: [`CliOptions.output_console`](src/cli/options.rs:63-64)
- **Description**: Print output to console instead of clipboard
- **Default**: `false`
- **Triggers**: CLI mode
- **Examples**:
  ```bash
  aibundle --files "*.rs" --output-console
  aibundle -f "*.py" -p
  ```

### Format Options

#### `-m, --format <FORMAT>`
- **Type**: String with validation
- **Source**: [`CliOptions.format`](src/cli/options.rs:72-73)
- **Description**: Output format for the aggregated content
- **Default**: `llm`
- **Valid Values**: `markdown`, `xml`, `json`, `llm`
- **Examples**:
  ```bash
  aibundle --files "*.rs" --format json
  aibundle -f "*.py" -m markdown
  aibundle --files "*.js" --format xml
  ```

**Format Descriptions**:
- `llm`: Optimized format for Large Language Models (default)
- `markdown`: Standard Markdown format with code blocks
- `json`: Structured JSON format with metadata
- `xml`: XML format with hierarchical structure

### Directory Options

#### `-d, --source-dir <DIR>`
- **Type**: String
- **Source**: [`CliOptions.source_dir`](src/cli/options.rs:75-76)
- **Description**: Source directory (alternative to positional argument)
- **Default**: `.` (current directory)
- **Note**: Positional `SOURCE_DIR` takes precedence if both are provided
- **Examples**:
  ```bash
  aibundle --files "*.rs" --source-dir /path/to/project
  aibundle -f "*.py" -d ~/projects/myapp
  ```

#### `-r, --recursive`
- **Type**: Boolean flag
- **Source**: [`CliOptions.recursive`](src/cli/options.rs:78-79)
- **Description**: Enable recursive directory traversal
- **Default**: `false`
- **Examples**:
  ```bash
  aibundle --files "*.rs" --recursive
  aibundle -f "*.py" -r
  ```

### Filtering Options

#### `-g, --gitignore`
- **Type**: Boolean flag
- **Source**: [`CliOptions.gitignore`](src/cli/options.rs:84-85)
- **Description**: Use .gitignore rules for filtering files
- **Default**: `true`
- **Examples**:
  ```bash
  aibundle --files "*.rs" --gitignore
  aibundle -f "*.py" --no-gitignore    # Disable gitignore (if supported)
  ```

#### `-i, --ignore <PATTERNS>`
- **Type**: Comma-separated list of strings
- **Source**: [`CliOptions.ignore`](src/cli/options.rs:87-94)
- **Description**: Additional ignore patterns
- **Default**: `default` (uses built-in ignore patterns)
- **Built-in Patterns**: `node_modules`, `.git`, `dist`, `build`, `coverage`, `target` (see [`DEFAULT_IGNORED_DIRS`](src/models/constants.rs:178-185))
- **Examples**:
  ```bash
  aibundle --files "*.rs" --ignore "target,dist"
  aibundle -f "*.py" -i "__pycache__,*.pyc"
  aibundle --files "*.js" --ignore "default,custom_dir"
  ```

### Display Options

#### `-n, --line-numbers`
- **Type**: Boolean flag
- **Source**: [`CliOptions.line_numbers`](src/cli/options.rs:81-82)
- **Description**: Show line numbers in output
- **Default**: `false`
- **Examples**:
  ```bash
  aibundle --files "*.rs" --line-numbers
  aibundle -f "*.py" -n
  ```

### Configuration Options

#### `-S, --save-config`
- **Type**: Boolean flag
- **Source**: [`CliOptions.save_config`](src/cli/options.rs:95-96)
- **Description**: Save current options as default configuration
- **Default**: `false`
- **Triggers**: CLI mode
- **Behavior**: Creates/updates configuration file in user's home directory
- **Examples**:
  ```bash
  aibundle --save-config                    # Save default config
  aibundle --files "*.rs" --save-config     # Save config with file pattern
  ```

**Configuration File Location**: The configuration is saved to a TOML file in the user's home directory and includes both CLI and TUI sections. (Implementation: [`main.rs`](src/main.rs:60-110), [`config::load_config`](src/config/mod.rs))

## Mode Detection

*Source: [`main.rs`](src/main.rs:52-56)*

AIBundle automatically switches to CLI mode when any of these options are provided:
- `--files` / `-f`
- `--output-file` / `-o`
- `--output-console` / `-p`
- `--save-config` / `-S`

Otherwise, it runs in TUI mode for interactive file selection.

**CLI Workflow**: When CLI mode is detected, the application calls [`run_cli_mode`](src/cli/options.rs:264-360) which handles file loading, filtering, and output generation.

## Examples

### Basic Usage

```bash
# TUI mode - interactive file browser
aibundle

# TUI mode starting in specific directory
aibundle /path/to/project

# CLI mode - bundle all Rust files
aibundle --files "*.rs"

# CLI mode - bundle Python files with line numbers
aibundle --files "*.py" --line-numbers
```

### Output Control

```bash
# Save to file
aibundle --files "*.js" --output-file bundle.md

# Print to console
aibundle --files "*.rs" --output-console

# Copy to clipboard (default behavior)
aibundle --files "*.py"
```

### Format Options

```bash
# JSON format for structured data
aibundle --files "*.rs" --format json --output-file rust_files.json

# Markdown format for documentation
aibundle --files "*.md" --format markdown --output-console

# LLM format for AI processing (default)
aibundle --files "*.py" --format llm
```

### Advanced Filtering

```bash
# Recursive search with custom ignore patterns
aibundle --files "*.rs" --recursive --ignore "target,dist,node_modules"

# Disable gitignore and include all files
aibundle --files "*" --gitignore false --ignore ""

# Search for specific files
aibundle --search "config" --recursive
```

### Configuration Management

```bash
# Save current settings as default
aibundle --files "*.rs" --format json --recursive --save-config

# Use saved configuration (automatic when no CLI options provided)
aibundle
```

### Complex Workflows

```bash
# Bundle all source code files for AI analysis
aibundle --files "*.{rs,py,js,ts}" --format llm --recursive --output-file codebase.txt

# Generate documentation from markdown files
aibundle --files "*.md" --format markdown --recursive --output-file docs.md

# Export project structure as JSON
aibundle --files "*" --format json --recursive --ignore "default" --output-file structure.json
```

## Option Interactions

### Precedence Rules
1. **Source Directory**: Positional `SOURCE_DIR` overrides `--source-dir`
2. **Configuration**: CLI options override saved configuration
3. **Output**: `--output-file` and `--output-console` are mutually exclusive (last one wins)

### Format-Specific Behavior
- **LLM Format**: Includes directory structure for context (see [`run_cli_mode`](src/cli/options.rs:327-341))
- **Other Formats**: May exclude directories when using file patterns (see [`run_cli_mode`](src/cli/options.rs:317-320))

### Selection Limits
- Default selection limit: 400 items (see [`DEFAULT_SELECTION_LIMIT`](src/models/constants.rs:45))
- Can be configured via saved configuration
- Prevents excessive memory usage

### Binary File Handling
- Binary files are excluded by default in CLI mode (see [`to_ignore_config`](src/cli/options.rs:247))
- This behavior is currently hardcoded and not configurable via CLI options
- Future enhancement: `--include-binary` flag (see TODO in [`src/cli/options.rs:362`](src/cli/options.rs:362))

## Environment Variables

The application uses the following environment variables:

### Configuration File Location
- **Windows**: `%USERPROFILE%\.aibundle.config.toml`
- **Unix/Linux/macOS**: `$HOME/.aibundle.config.toml`

### Temporary Files
- Uses system temporary directory (`std::env::temp_dir()`) for clipboard operations on Windows/WSL
- Temporary files are named `aibundle_clipboard_temp.txt`

## Error Handling

The application provides clear error messages for:
- Invalid file patterns
- Inaccessible directories
- Permission issues
- Invalid format specifications
- Configuration file errors
- TOML parsing errors in configuration files
- Clipboard access failures
- Home directory detection failures

### Platform-Specific Error Handling
- **WSL Detection**: Automatically detects WSL environment by reading `/proc/version`
- **Clipboard Fallbacks**: Falls back from Wayland to X11 on Linux systems
- **Terminal Reset**: Attempts to reset terminal state on Unix systems using `stty sane`

## Built-in Help and Version Options

### `--help, -h`
- **Type**: Boolean flag (built-in clap option)
- **Description**: Display help information and usage examples
- **Examples**:
  ```bash
  aibundle --help                        # Show full help
  aibundle -h                           # Show brief help
  ```

### `--version, -V`
- **Type**: Boolean flag (built-in clap option)
- **Description**: Display version information
- **Current Version**: 0.7.5 (see [`VERSION`](src/models/constants.rs:35))
- **Examples**:
  ```bash
  aibundle --version                     # Show version
  aibundle -V                           # Show version (short form)
  ```

## Advanced Features and Edge Cases

### Configuration File Format
The configuration file uses TOML format with separate sections for CLI and TUI modes:
```toml
[cli]
files = "*.rs"
format = "llm"
gitignore = true
ignore = ["target", "dist"]
line-numbers = false
recursive = true
source-dir = "."
selection-limit = 400

[tui]
# Same structure as CLI section
```

### Ignore Pattern Behavior
- **Default Patterns**: `node_modules`, `.git`, `dist`, `build`, `coverage`, `target` (see [`DEFAULT_IGNORED_DIRS`](src/models/constants.rs:178-185))
- **Pattern Combination**: Using `--ignore "default,custom"` combines built-in patterns with custom ones
- **Override Behavior**: Using `--ignore "custom"` (without "default") replaces built-in patterns
- **Gitignore Integration**: Respects `.gitignore` files when `--gitignore` is enabled (default: true)

### Output Format Details
- **LLM Format**: Includes file tree structure and optimized for AI model consumption
- **Markdown Format**: Standard markdown with fenced code blocks and language detection
- **JSON Format**: Structured data with metadata, file paths, and content
- **XML Format**: Hierarchical XML structure with file and directory elements

### Platform-Specific Behaviors
- **Windows**: Uses PowerShell for clipboard operations with UTF-8 encoding
- **WSL**: Automatically detects WSL environment and uses Windows clipboard via PowerShell
- **macOS**: Uses `pbcopy`/`pbpaste` for clipboard operations
- **Linux**: Attempts Wayland (`wl-copy`/`wl-paste`) first, falls back to X11 (`xclip`)

## Implementation Notes

- File patterns support glob syntax (`*`, `**`, `?`, `[]`)
- Ignore patterns are additive (combine with built-in patterns) - see [`to_ignore_config`](src/cli/options.rs:242-250)
- Configuration is saved in TOML format via [`get_merged_config`](src/cli/options.rs:219-227)
- Both CLI and TUI modes share the same underlying file processing engine
- The application respects `.gitignore` files by default
- Binary files are excluded by default in CLI mode
- Output format conversion handled by [`to_output_format`](src/cli/options.rs:197-204)
- CLI options are converted to internal structures via [`to_cli_mode_options`](src/cli/options.rs:168-180)
- Memory optimization: Uses references and minimal cloning throughout the codebase

## Source Code Structure

The command line interface is implemented across several key files:

- **[`src/cli/options.rs`](src/cli/options.rs)**: Main CLI option definitions, parsing, and workflow logic
- **[`src/cli/mod.rs`](src/cli/mod.rs)**: CLI module exports and organization
- **[`src/main.rs`](src/main.rs)**: Application entry point and mode detection
- **[`src/models/constants.rs`](src/models/constants.rs)**: Version, defaults, and ignore patterns
- **[`src/config/mod.rs`](src/config/mod.rs)**: Configuration loading and saving
- **[`src/models/enums.rs`](src/models/enums.rs)**: Output format enums and parsing
- **[`src/models/app_config.rs`](src/models/app_config.rs)**: Configuration structures and defaults

## Future Enhancements

Based on TODO comments in the source code, the following features are planned:

### Planned CLI Options
- `--include-binary`: Include binary files in output (see [`src/cli/options.rs:362`](src/cli/options.rs:362))
- `--config-path`: Custom configuration file path (see [`src/cli/options.rs:362`](src/cli/options.rs:362))
- Additional CLI subcommands or modes (see [`src/main.rs:239`](src/main.rs:239))

### Planned Validations
- CLI argument combination validation (see [`src/cli/options.rs:363`](src/cli/options.rs:363))
- Configuration file content validation (see [`src/config/mod.rs:112`](src/config/mod.rs:112))

### Planned Improvements
- More CLI utilities and helper functions (see [`src/cli/mod.rs:30`](src/cli/mod.rs:30))
- Enhanced language mappings for additional file types (see [`src/models/constants.rs:232`](src/models/constants.rs:232))
- Configurable icons and ignore patterns (see [`src/models/constants.rs:233`](src/models/constants.rs:233))

## Troubleshooting

### Common Issues

#### Configuration File Problems
- **Location**: Ensure config file is in the correct location (`~/.aibundle.config.toml`)
- **Format**: Verify TOML syntax is correct
- **Permissions**: Check file permissions allow reading/writing

#### Clipboard Issues
- **WSL**: Ensure PowerShell is available and accessible
- **Linux**: Install `wl-clipboard` (Wayland) or `xclip` (X11)
- **macOS**: Clipboard should work out of the box

#### File Pattern Issues
- **Glob Syntax**: Use proper glob patterns (`*.rs`, `**/*.py`, etc.)
- **Escaping**: Quote patterns in shell to prevent expansion
- **Case Sensitivity**: Patterns are case-sensitive on Unix systems

#### Performance Issues
- **Selection Limit**: Reduce selection limit if memory usage is high
- **Ignore Patterns**: Use appropriate ignore patterns to exclude large directories
- **Recursive Mode**: Be cautious with recursive mode in large directory trees