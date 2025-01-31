# Configuration Reference 📝

## Configuration File

AIBundle uses a TOML configuration file for persistent settings.

### File Location
```bash
.aibundle.config           # Project-specific config
$HOME/.aibundle/config    # User default config
/etc/aibundle/config      # System-wide config
```

## Basic Configuration 🎯

### Package Settings
```toml
[package]
name = "my-project"
version = "0.1.0"
default_format = "xml"
default_recursive = true
default_gitignore = true
```

### Format Settings
```toml
[format]
default = "xml"          # xml, markdown, json
line_numbers = false     # Enable line numbers
pretty = false          # Pretty print output
compact = false         # Compact output
```

### File Settings
```toml
[files]
ignore = [              # Default ignore patterns
    "target",
    "node_modules",
    ".git",
    "*.log"
]
binary = false          # Include binary files
recursive = true        # Recursive search
```

## Advanced Configuration 🔧

### Search Settings
```toml
[search]
case_sensitive = false   # Case-sensitive search
regex = false           # Use regex patterns
whole_word = false      # Match whole words
```

### Output Settings
```toml
[output]
clipboard = true        # Copy to clipboard
console = false         # Write to console
file = ""              # Default output file
```

### Integration Settings
```toml
[integration]
git = true             # Git integration
editor = "vim"         # Default editor
terminal = "xterm"     # Terminal type
```

## Feature Configuration 📊

### Icons and UI
```toml
[ui]
icons = true           # Show icons
colors = true          # Use colors
unicode = true         # Unicode support
ascii = false          # ASCII fallback
```

### Performance Settings
```toml
[performance]
cache = true           # Enable caching
cache_size = 1000      # Max cache entries
threads = 4            # Thread count
timeout = 30          # Operation timeout
```

### Debug Settings
```toml
[debug]
log_level = "info"     # debug, info, warn, error
trace = false          # Enable tracing
verbose = false        # Verbose output
```

## Environment Variables 🌍

### Configuration Override
```bash
# Override config location
export AIBUNDLE_CONFIG="/path/to/config"

# Override format
export AIBUNDLE_FORMAT="markdown"

# Override ignore patterns
export AIBUNDLE_IGNORE="target,dist"
```

## Configuration Examples 📚

### Minimal Configuration
```toml
[package]
default_format = "xml"
default_recursive = true

[files]
ignore = ["target", ".git"]
```

### Full Configuration
```toml
[package]
name = "my-project"
version = "0.1.0"
default_format = "markdown"
default_recursive = true
default_gitignore = true

[format]
default = "markdown"
line_numbers = true
pretty = true

[files]
ignore = [
    "target",
    "node_modules",
    ".git",
    "dist"
]
binary = false
recursive = true

[search]
case_sensitive = false
regex = true

[output]
clipboard = true
console = false

[ui]
icons = true
colors = true
```

## Best Practices 💡

### Configuration Structure
1. Package Settings
   - Set project defaults
   - Define core behavior
   - Specify version

2. File Management
   - Configure ignore patterns
   - Set search behavior
   - Define output format

3. Integration
   - Configure external tools
   - Set up git integration
   - Define editor preferences

### Version Control
```toml
# .gitignore
.aibundle-cache/
*.bundle.md
```

```toml
# .aibundle.config
[package]
version = "0.1.0"
```

## Troubleshooting 🔧

### Common Issues
1. Config Not Loading
   ```bash
   # Verify config
   cat .aibundle.config
   
   # Check permissions
   ls -l .aibundle.config
   ```

2. Invalid Settings
   ```toml
   # Valid format values
   default_format = "xml"    # Correct
   default_format = "text"   # Invalid
   ```

3. Path Issues
   ```bash
   # Absolute paths
   output_dir = "/absolute/path"
   
   # Relative paths
   output_dir = "./relative/path"
   ```
