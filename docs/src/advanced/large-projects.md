# Large Projects Guide 📚

## Overview

Managing large codebases with AIBundle requires special consideration for performance, organization, and efficiency.

## Size Considerations 📊

### Project Limits
- Selection limit: 400 items
- Recommended file size: < 1MB per file
- Total bundle size: < 50MB
- Directory depth: Unlimited

## Optimization Strategies 🎯

### Directory Organization
```bash
# Focus on specific modules
aibundle --cli --files "src/module/**/*.rs"

# Exclude test directories
aibundle --cli --files "src/**/*.rs" --ignore "tests,benchmarks"

# Split by feature
aibundle --cli --files "src/features/auth/**/*"
```

### Smart Selection
```bash
# Select core files only
aibundle --cli --files "{main,lib}.rs,**/mod.rs"

# Filter by recent changes
aibundle --cli --files "$(git diff HEAD~5 --name-only)"

# Focus on specific types
aibundle --cli --files "*.{rs,toml}" --no-recursive
```

## Performance Tips ⚡

### Filtering Strategies
1. Use Specific Patterns
   ```bash
   # Good
   aibundle --cli --files "src/core/**/*.rs"
   
   # Avoid
   aibundle --cli --files "**/*"
   ```

2. Leverage Ignore Rules
   ```toml
   # .aibundle.config
   default_ignore = [
     "target",
     "tests",
     "docs",
     "examples",
     "benches"
   ]
   ```

3. Directory Scoping
   ```bash
   # Scope to specific directory
   aibundle --cli --files "*.rs" --source-dir ./src/feature
   ```

## Working with Large Codebases 🗄️

### Module-based Approach
1. Core Modules
   ```bash
   aibundle --cli --files "src/core/**/*.rs"
   ```

2. Feature Modules
   ```bash
   aibundle --cli --files "src/features/**/mod.rs"
   ```

3. API Interfaces
   ```bash
   aibundle --cli --files "src/api/**/*.rs"
   ```

### Incremental Selection
```bash
# Stage 1: Core interfaces
aibundle --cli --files "**/mod.rs" --out core.md

# Stage 2: Implementation details
aibundle --cli --files "src/impl/**/*.rs" --out impl.md

# Stage 3: Configuration
aibundle --cli --files "*.{toml,yaml}" --out config.md
```

## Memory Management 💾

### Optimization Techniques
1. Selective Loading
   - Use specific patterns
   - Limit recursion depth
   - Filter unnecessary files

2. Chunked Processing
   - Split large selections
   - Process by module
   - Use incremental approach

3. Cache Management
   - Clear selection regularly
   - Use fresh searches
   - Restart for large changes

## Best Practices 💡

### Project Organization
1. Structure
   ```
   project/
   ├── src/
   │   ├── core/
   │   ├── features/
   │   └── api/
   ├── tests/
   └── .aibundle.config
   ```

2. Configuration
   ```toml
   # .aibundle.config
   [package]
   default_recursive = false
   default_gitignore = true
   default_format = "markdown"
   ```

### Selection Strategy
1. Top-down Approach
   - Start with architecture
   - Add implementation details
   - Include configurations

2. Feature-based Selection
   - Select by feature
   - Include related tests
   - Add dependencies

3. Change-based Selection
   - Focus on modified files
   - Include affected modules
   - Add context files

## Common Patterns 📋

### Architecture Review
```bash
# Core architecture
aibundle --cli \
  --files "src/**/mod.rs" \
  --format markdown \
  --out architecture.md
```

### Feature Analysis
```bash
# Feature-specific files
aibundle --cli \
  --files "src/features/specific-feature/**/*.rs" \
  --format xml
```

### Dependency Review
```bash
# Project dependencies
aibundle --cli \
  --files "Cargo.{toml,lock}" \
  --format markdown \
  --out dependencies.md
```

## Error Prevention 🚨

### Common Issues
1. Selection Overload
   - Monitor item count
   - Use specific patterns
   - Split large selections

2. Memory Issues
   - Clear selection cache
   - Restart for large projects
   - Use incremental selection

3. Performance Problems
   - Optimize patterns
   - Leverage ignore rules
   - Scope selections
