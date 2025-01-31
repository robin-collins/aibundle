# File Selection Patterns 🎯

## Pattern Syntax

AIBundle supports various file selection patterns to help you precisely target the files you need.

### Basic Patterns

| Pattern | Description | Example |
|---------|-------------|---------|
| `*` | Match any characters | `*.rs` |
| `?` | Match single character | `test?.rs` |
| `**` | Match directories recursively | `src/**/*.rs` |
| `{}` | Group patterns | `*.{rs,toml}` |

## Examples 📝

### Simple Patterns
```bash
# All Rust files
aibundle --cli --files "*.rs"

# All source files
aibundle --cli --files "src/*.rs"

# Specific file types
aibundle --cli --files "*.{rs,toml,md}"
```

### Advanced Patterns
```bash
# Files starting with 'test'
aibundle --cli --files "test*.rs"

# Files in any src directory
aibundle --cli --files "**/src/*.rs"

# Multiple specific files
aibundle --cli --files "{main,lib,test}.rs"
```

## Search Patterns 🔍

Use the `--search` option to filter files by content:

```bash
# Files containing "TODO"
aibundle --cli --search "TODO"

# Combine with file patterns
aibundle --cli --files "*.rs" --search "FIXME"
```

## Exclusion Patterns 🚫

### Using .gitignore
```bash
# Respect .gitignore (default)
aibundle --cli --files "*.rs" --gitignore

# Ignore .gitignore
aibundle --cli --files "*.rs" --no-gitignore
```

### Custom Ignore Patterns
```bash
# Ignore specific patterns
aibundle --cli --files "*.rs" --ignore "target,tests"
```

## Visual Selection Guide 📊

[Image: Diagram showing pattern matching examples]
*Visual representation of different pattern matching scenarios*

## Best Practices 💡

1. Start with specific patterns
2. Use grouping for related files
3. Leverage search for content-based selection
4. Combine patterns for precise targeting
