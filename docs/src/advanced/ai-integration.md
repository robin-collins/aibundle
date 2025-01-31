# AI/LLM Integration Guide 🤖

## Overview

AIBundle is specifically designed to work seamlessly with AI language models like ChatGPT, Claude, and others.

## Format Selection for AI Platforms 🎯

### ChatGPT
```bash
# Recommended format
aibundle --cli --files "*.rs" --format markdown

# Example output:
```rust:src/main.rs
fn main() {
    println!("Hello, world!");
}
```

### Claude
```bash
# Recommended format
aibundle --cli --files "*.rs" --format xml

# Example output:
<file name="src/main.rs">
fn main() {
    println!("Hello, world!");
}
</file>
```

### GitHub Copilot
```bash
# Recommended format
aibundle --cli --files "*.rs" --format markdown --line-numbers
```

## Best Practices for AI Interaction 💡

### Context Management
1. Select relevant files only
2. Include configuration files
3. Maintain directory structure
4. Provide necessary imports

### File Organization
```bash
# Include related files
aibundle --cli --files "{main,lib}.rs,Cargo.toml"

# Include full module
aibundle --cli --files "src/module/**/*.rs"
```

## Common AI Workflows 🔄

### Code Review
```bash
# Bundle changed files
aibundle --cli --files "$(git diff --name-only)"

# Include tests
aibundle --cli --files "{src,tests}/**/*.rs"
```

### Architecture Discussion
```bash
# Core architecture files
aibundle --cli --files "src/{main,lib}.rs,**/mod.rs"

# Configuration context
aibundle --cli --files "*.{rs,toml}"
```

### Bug Investigation
```bash
# Related source files
aibundle --cli --search "ERROR" --format markdown

# Test files
aibundle --cli --files "tests/**/*_test.rs"
```

## Size Optimization 📊

### For Large Codebases
- Use specific file patterns
- Exclude test files when unnecessary
- Focus on relevant modules
- Use search to find specific code

### For Small Changes
- Select specific files
- Include immediate dependencies
- Add configuration context
- Include relevant tests

## Platform-Specific Tips 🎯

### ChatGPT
- Use Markdown format
- Include line numbers
- Split large codebases
- Maintain conversation context

### Claude
- Use XML format
- Include full context
- Leverage directory structure
- Use consistent formatting

### Copilot
- Focus on single files
- Include related tests
- Maintain project structure
- Add type information

## Error Handling 🚨

### Common Issues
1. Token Limits
   - Split large codebases
   - Focus on relevant files
   - Use selective patterns

2. Format Issues
   - Verify output format
   - Check file encoding
   - Validate syntax

3. Context Loss
   - Include configuration
   - Add type definitions
   - Maintain dependencies

## Best Practices 💡

1. Format Selection
   - Match AI platform requirements
   - Consider readability
   - Include necessary context

2. File Selection
   - Choose relevant files
   - Include dependencies
   - Maintain structure

3. Context Management
   - Provide configuration
   - Include type information
   - Add documentation

4. Performance
   - Optimize file selection
   - Manage token limits
   - Split large contexts
