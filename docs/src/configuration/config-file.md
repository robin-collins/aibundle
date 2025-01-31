# Configuration File 📝

## Overview

AIBundle uses a TOML configuration file (`.aibundle.config`) to store default settings and preferences.

## File Location 📍

- Project root: `./.aibundle.config`
- Created automatically when saving settings
- Can be manually edited
- Version controlled (not gitignored by default)

## Configuration Format 📋

```toml
# .aibundle.config
[package]
default_format = "xml"          # xml, markdown, or json
default_gitignore = true        # use .gitignore rules
default_line_numbers = false    # show line numbers
default_recursive = true        # recursive file search
```

## Settings Reference ⚙️

### Output Format
```toml
default_format = "xml"    # Default output format
# Possible values:
# - "xml"      (Default structured format)
# - "markdown" (Documentation friendly)
# - "json"     (Machine readable)
```

### Git Integration
```toml
default_gitignore = true    # Respect .gitignore rules
# Possible values:
# - true   (Use .gitignore rules)
# - false  (Ignore .gitignore rules)
```

### Display Options
```toml
default_line_numbers = false    # Line number display
# Possible values:
# - true   (Show line numbers)
# - false  (Hide line numbers)
```

### Search Behavior
```toml
default_recursive = true    # Recursive search
# Possible values:
# - true   (Search subdirectories)
# - false  (Current directory only)
```

## Saving Configuration 💾

### From TUI
1. Adjust settings using keyboard shortcuts
2. Press 's' to save current configuration
3. Configuration saved to `.aibundle.config`

### From CLI
```bash
# Save current settings as default
aibundle --cli --files "*.rs" --format markdown --save-config

# Use saved configuration
aibundle --cli --files "*.rs"
```

## Configuration Priority 🎯

1. Command-line arguments (highest)
2. Project `.aibundle.config`
3. Default settings (lowest)

## Example Configurations 📚

### Documentation Focus
```toml
default_format = "markdown"
default_line_numbers = true
default_gitignore = true
default_recursive = true
```

### AI Assistant Focus
```toml
default_format = "xml"
default_line_numbers = false
default_gitignore = true
default_recursive = true
```

### Development Focus
```toml
default_format = "json"
default_line_numbers = true
default_gitignore = true
default_recursive = false
```

## Best Practices 💡

1. Version control your `.aibundle.config`
2. Document any custom settings
3. Use project-specific configurations
4. Review settings before large operations
5. Save common configurations for reuse
