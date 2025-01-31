# Initial Setup 🚀

## First Run

When you first run AIBundle, it operates with default settings. Here's how to set up your preferred configuration:

## Basic Configuration Steps 📝

1. Launch AIBundle:
   ```bash
   aibundle
   ```

2. Configure basic settings:
   - Format (f): Choose output format
   - Gitignore (g): Toggle .gitignore support
   - Line numbers (n): Toggle line numbers
   - Binary files (b): Toggle binary file handling

3. Save configuration:
   - Press 's' to save settings
   - Creates `.aibundle.config` in current directory

## CLI Configuration 🖥️

Configure via command line:
```bash
# Set up with specific preferences
aibundle --cli \
  --files "*.rs" \
  --format markdown \
  --line-numbers \
  --gitignore \
  --save-config
```

## Default Settings ⚙️

Out of the box defaults:
- Format: XML
- Gitignore: Enabled
- Line Numbers: Disabled
- Recursive: Enabled
- Binary Files: Hidden

## Environment Setup 🌟

### Recommended Setup

1. Global Ignore Patterns:
   ```toml
   # .aibundle.config
   default_ignore = [
     "node_modules",
     ".git",
     "target",
     "dist"
   ]
   ```

2. Version Control:
   ```gitignore
   # .gitignore
   !.aibundle.config    # Include config in git
   ```

3. Editor Integration:
   - Add to project root
   - Configure IDE to recognize config

## Verification ✅

Test your configuration:

1. Run AIBundle
2. Check status bar for settings
3. Make test selection
4. Verify output format
5. Confirm ignore patterns

## Troubleshooting 🔧

Common setup issues:

1. Config not saving:
   - Check write permissions
   - Verify directory path

2. Settings not applying:
   - Check config syntax
   - Verify file location

3. Ignore patterns not working:
   - Check .gitignore syntax
   - Verify pattern format

## Next Steps 🎯

1. Customize your [Default Settings](./default-settings.md)
2. Learn about [File Selection](../cli/file-selection.md)
3. Explore [Output Formats](../cli/output-formats.md)
