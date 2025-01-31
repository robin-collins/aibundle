# Troubleshooting Guide 🔧

## Common Issues

### Installation Problems 📥

#### Cargo Install Fails
```bash
# Error: Failed to install aibundle
cargo install aibundle fails
```

Solutions:
1. Update Rust toolchain
   ```bash
   rustup update
   ```

2. Check dependencies
   ```bash
   cargo clean
   cargo install aibundle --verbose
   ```

3. Verify system requirements
   - Rust 1.70 or later
   - Git 2.0 or later
   - Terminal with UTF-8 support

### TUI Issues 🖥️

#### Display Problems
1. Broken Layout
   ```bash
   # Reset terminal
   reset
   
   # Launch with fallback mode
   TERM=xterm-256color aibundle
   ```

2. Missing Icons
   - Verify terminal UTF-8 support
   - Check font compatibility
   - Use ASCII mode if needed:
   ```toml
   # .aibundle.config
   [accessibility]
   use_ascii = true
   ```

#### Navigation Issues
1. Keyboard Not Responding
   - Check terminal mode
   - Verify key bindings
   - Reset terminal settings

2. Selection Problems
   ```bash
   # Clear selection cache
   rm -rf .aibundle-cache
   ```

### CLI Problems 🛠️

#### Pattern Matching
1. No Files Found
   ```bash
   # Debug pattern matching
   aibundle --cli --files "*.rs" --verbose
   ```

2. Unexpected Files
   ```bash
   # Check ignore patterns
   aibundle --cli --files "*.rs" --no-gitignore
   ```

#### Output Issues
1. Empty Output
   ```bash
   # Verify file access
   ls -la $(aibundle --cli --files "*.rs" --list)
   ```

2. Format Problems
   ```bash
   # Force specific format
   aibundle --cli --files "*.rs" --format markdown --force
   ```

### Configuration Issues ⚙️

#### Config Not Loading
1. Check Location
   ```bash
   ls -la .aibundle.config
   ```

2. Validate Syntax
   ```bash
   cat .aibundle.config
   ```

3. Reset Configuration
   ```bash
   mv .aibundle.config .aibundle.config.bak
   ```

#### Invalid Settings
```toml
# Common fixes in .aibundle.config
[package]
default_format = "xml"  # Must be: xml, markdown, or json
default_recursive = true  # Must be boolean
default_ignore = []  # Must be array of strings
```

## Performance Issues 🚀

### Slow Operation
1. Large Projects
   ```bash
   # Limit recursion
   aibundle --cli --files "*.rs" --no-recursive
   
   # Use specific patterns
   aibundle --cli --files "src/core/**/*.rs"
   ```

2. Memory Usage
   ```bash
   # Monitor usage
   top -p $(pgrep aibundle)
   
   # Split operations
   for dir in src/*; do
     aibundle --cli --files "$dir/**/*.rs"
   done
   ```

### System Resources
1. High CPU Usage
   - Limit search depth
   - Use specific patterns
   - Split large operations

2. Memory Constraints
   - Clear selection cache
   - Process in chunks
   - Use streaming output

## Integration Issues 🔌

### Git Integration
1. Gitignore Not Working
   ```bash
   # Debug gitignore
   git check-ignore -v $(aibundle --cli --files "*.rs" --list)
   ```

2. Branch Problems
   ```bash
   # Verify git status
   git status
   
   # Check branch
   git branch --show-current
   ```

### Clipboard Issues
1. Copy Fails
   ```bash
   # Check clipboard provider
   which xclip || which wl-copy
   
   # Force console output
   aibundle --cli --files "*.rs" --output-console
   ```

2. Format Problems
   ```bash
   # Verify output
   aibundle --cli --files "*.rs" --format markdown --output-console
   ```

## Recovery Steps 🔄

### Quick Fixes
1. Reset Terminal
   ```bash
   reset
   clear
   ```

2. Clear Cache
   ```bash
   rm -rf .aibundle-cache
   ```

3. Verify Installation
   ```bash
   cargo install --force aibundle
   ```

### Debug Mode
```bash
# Enable debug output
RUST_LOG=debug aibundle

# Full verbose mode
RUST_LOG=trace aibundle --cli --verbose
```

## Best Practices 💡

### Prevention
1. Regular Updates
   ```bash
   cargo install aibundle --force
   ```

2. Configuration Backup
   ```bash
   cp .aibundle.config .aibundle.config.backup
   ```

3. Test Changes
   ```bash
   # Test in isolation
   mkdir test-dir && cd test-dir
   aibundle --cli --files "test.*"
   ```

### Maintenance
1. Clean Installation
   ```bash
   cargo uninstall aibundle
   cargo install aibundle
   ```

2. Cache Management
   ```bash
   # Regular cleanup
   find . -name ".aibundle-cache" -exec rm -rf {} +
   ```

3. Config Validation
   ```bash
   # Verify syntax
   cat .aibundle.config | grep -v '^#' | grep .
   ```
