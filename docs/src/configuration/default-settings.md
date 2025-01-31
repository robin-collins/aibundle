# Default Settings Guide ⚙️

## Core Settings 🎯

AIBundle comes with carefully chosen defaults that work well for most use cases.

### Default Values
```toml
[package]
default_format = "xml"
default_gitignore = true
default_line_numbers = false
default_recursive = true
default_ignore = ["node_modules", ".git", "target", "dist", "build", "coverage"]
```

## Format Settings 📝

### Output Format
```toml
default_format = "xml"    # Default format for all operations
```

| Format | Best For | Use Case |
|--------|----------|----------|
| `xml` | AI/LLM | Structured code sharing with AI assistants |
| `markdown` | Documentation | README files, documentation systems |
| `json` | Integration | API responses, programmatic processing |

## File Handling 📂

### Gitignore Integration
```toml
default_gitignore = true    # Use .gitignore rules
```

Benefits:
- Respects project exclusions
- Maintains consistency with VCS
- Reduces noise in output

### Recursive Search
```toml
default_recursive = true    # Include subdirectories
```

When enabled:
- Searches all subdirectories
- Respects ignore patterns
- Maintains directory structure

## Display Options 🎨

### Line Numbers
```toml
default_line_numbers = false    # Line number display
```

Considerations:
- Adds context for code review
- Increases output size
- Not available in JSON format

## Ignore Patterns 🚫

### Default Ignores
```toml
default_ignore = [
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    "coverage"
]
```

Common patterns:
- Build directories
- Dependencies
- Version control
- Test coverage
- Temporary files

## Customization Examples 📚

### For Documentation
```toml
[package]
default_format = "markdown"
default_line_numbers = true
default_gitignore = true
default_recursive = true
default_ignore = ["node_modules", ".git", "target"]
```

### For AI Integration
```toml
[package]
default_format = "xml"
default_line_numbers = false
default_gitignore = true
default_recursive = true
default_ignore = ["node_modules", ".git", "target", "tests"]
```

### For Development
```toml
[package]
default_format = "json"
default_line_numbers = true
default_gitignore = false
default_recursive = false
default_ignore = []
```

## Best Practices 💡

1. Start with Default Settings
   - Use defaults initially
   - Customize based on needs
   - Document changes

2. Project-Specific Settings
   - Consider project requirements
   - Match team preferences
   - Maintain consistency

3. Performance Considerations
   - Limit recursive depth when needed
   - Use specific ignore patterns
   - Balance completeness with speed

4. Maintenance
   - Review settings periodically
   - Update ignore patterns
   - Keep documentation current
