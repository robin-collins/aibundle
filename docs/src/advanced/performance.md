# Performance Tips & Optimization 🚀

## Performance Overview

AIBundle is designed to handle large codebases efficiently, but proper usage can significantly improve performance.

## Quick Tips 💨

### Command Line Performance
```bash
# Faster than recursive search
aibundle --cli --files "*.rs" --no-recursive

# More efficient than wildcard
aibundle --cli --files "src/{core,util}/*.rs"

# Faster than searching all files
aibundle --cli --search "TODO" --files "*.rs"
```

## Memory Optimization 💾

### Selection Strategies
1. Direct Selection
   ```bash
   # Better
   aibundle --cli --files "src/core/*.rs"
   
   # Avoid
   aibundle --cli --files "**/*.rs"
   ```

2. Chunked Processing
   ```bash
   # Process by module
   for module in core api utils; do
     aibundle --cli --files "src/$module/**/*.rs" --out "$module.md"
   done
   ```

3. Smart Filtering
   ```bash
   # Use specific ignore patterns
   aibundle --cli --files "*.rs" --ignore "tests,examples,benches"
   ```

## File System Tips 📂

### Directory Structure
```
project/
├── .aibundle.config    # Configure defaults
├── src/               # Organize source
│   ├── core/         # Core modules
│   └── features/     # Feature modules
└── .gitignore        # Exclude patterns
```

### Ignore Patterns
```toml
# .aibundle.config
default_ignore = [
  "target",
  "node_modules",
  "dist",
  "build",
  "**/tests",
  "**/*.test.*"
]
```

## Search Optimization 🔍

### Pattern Matching
```bash
# Efficient pattern
aibundle --cli --files "src/**/*.{rs,toml}"

# Avoid deep recursion
aibundle --cli --files "*.rs" --source-dir ./src/core
```

### Content Search
```bash
# Scope search to specific files
aibundle --cli --search "TODO" --files "src/**/*.rs"

# Combine with git
aibundle --cli --files "$(git diff --name-only)" --search "FIXME"
```

## Resource Usage 📊

### CPU Optimization
1. Pattern Matching
   - Use specific patterns
   - Limit recursion depth
   - Leverage .gitignore

2. File Processing
   - Filter early
   - Process in chunks
   - Use incremental approach

3. Search Operations
   - Scope searches
   - Use file filters
   - Combine operations

### Memory Management
1. Selection Limits
   - Stay under 400 items
   - Split large selections
   - Clear cache regularly

2. File Handling
   - Process incrementally
   - Avoid large files
   - Use streaming when possible

## Best Practices 💡

### Configuration
```toml
# .aibundle.config
[package]
default_recursive = false     # Limit recursion
default_gitignore = true     # Use efficient filtering
default_line_numbers = false  # Reduce output size
```

### Command Structure
```bash
# Efficient command structure
aibundle --cli \
  --files "src/core/*.rs" \
  --no-recursive \
  --format markdown \
  --ignore "tests"
```

### Workflow Optimization
1. Project Setup
   - Configure ignore patterns
   - Set default options
   - Organize directory structure

2. Selection Strategy
   - Use specific patterns
   - Filter early
   - Process incrementally

3. Output Management
   - Choose appropriate format
   - Control output size
   - Use streaming output

## Common Issues & Solutions 🔧

### Performance Problems
1. Slow Selection
   ```bash
   # Instead of
   aibundle --cli --files "**/*"
   
   # Use
   aibundle --cli --files "src/**/*.rs" --no-recursive
   ```

2. High Memory Usage
   ```bash
   # Split large selections
   for dir in $(ls src/); do
     aibundle --cli --files "src/$dir/**/*.rs"
   done
   ```

3. Search Performance
   ```bash
   # Scope searches effectively
   aibundle --cli --search "TODO" --files "src/core/**/*.rs"
   ```

## Advanced Optimization 🔬

### Custom Scripts
```bash
#!/bin/bash
# optimize-bundle.sh
modules=(core api utils)
for module in "${modules[@]}"; do
  aibundle --cli \
    --files "src/$module/**/*.rs" \
    --format markdown \
    --out "$module.md"
done
```

### Integration Tips
1. CI/CD Pipeline
   - Use specific patterns
   - Cache results
   - Split large operations

2. Development Workflow
   - Focus on changed files
   - Use incremental updates
   - Leverage git integration
