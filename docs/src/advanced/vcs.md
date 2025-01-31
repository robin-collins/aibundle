# Version Control Integration 🔄

## Git Integration Features 🌟

### Gitignore Support
```bash
# Use .gitignore patterns (default)
aibundle --cli --files "*.rs" --gitignore

# Ignore .gitignore rules
aibundle --cli --files "*.rs" --no-gitignore
```

### Changed Files Selection
```bash
# Select modified files
aibundle --cli --files "$(git diff --name-only)"

# Include staged files
aibundle --cli --files "$(git diff --cached --name-only)"

# Select files changed in last commit
aibundle --cli --files "$(git diff-tree --no-commit-id --name-only -r HEAD)"
```

## Common VCS Workflows 🔧

### Code Review
```bash
# Review specific branch changes
aibundle --cli --files "$(git diff main...feature-branch --name-only)"

# Review pull request
aibundle --cli --files "$(git diff origin/main...HEAD --name-only)"
```

### Feature Development
```bash
# Bundle feature-related files
aibundle --cli --files "$(git ls-files | grep 'feature-name')"

# Include new files
aibundle --cli --files "$(git ls-files --others --exclude-standard)"
```

## Integration Patterns 📋

### Pre-commit Hooks
```bash
#!/bin/bash
# .git/hooks/pre-commit
changed_files=$(git diff --cached --name-only)
aibundle --cli --files "$changed_files" --format markdown > review.md
```

### CI/CD Integration
```yaml
# Example GitHub Action
steps:
  - name: Bundle Changes
    run: |
      aibundle --cli \
        --files "$(git diff ${{ github.event.before }} --name-only)" \
        --format json \
        --out changes.json
```

## Repository Management 🗂️

### Configuration Files
```toml
# .aibundle.config in repository root
[package]
default_format = "markdown"
default_gitignore = true
default_recursive = true
```

### Ignore Patterns
```gitignore
# .gitignore
.aibundle.config   # Version control config
*.bundle.md        # Generated bundles
```

## Best Practices 💡

### Repository Structure
1. Place `.aibundle.config` in root
2. Version control configuration
3. Ignore generated bundles
4. Document integration patterns

### Workflow Integration
1. Use with code review tools
2. Integrate with CI/CD
3. Automate common patterns
4. Maintain consistent formats

### Performance Tips
1. Use specific branch diffs
2. Filter unnecessary files
3. Leverage .gitignore
4. Cache common selections

## Common Use Cases 📚

### Code Review
```bash
# Review feature branch
aibundle --cli \
  --files "$(git diff main...feature --name-only)" \
  --format markdown \
  --line-numbers \
  --out review.md
```

### Documentation
```bash
# Document changed APIs
aibundle --cli \
  --files "$(git diff --name-only | grep 'api')" \
  --format markdown \
  --out api-changes.md
```

### Bug Investigation
```bash
# Bundle related changes
aibundle --cli \
  --files "$(git log --pretty=format: --name-only --grep='bug-description')" \
  --format xml
```

## Error Handling 🚨

### Common Issues
1. Git Command Failures
   - Verify git installation
   - Check repository status
   - Validate branch names

2. File Access
   - Check permissions
   - Verify file existence
   - Handle deleted files

3. Integration Issues
   - Validate hook scripts
   - Check CI/CD config
   - Test automation scripts

## Advanced Integration 🔬

### Custom Scripts
```bash
#!/bin/bash
# bundle-feature.sh
branch_name=$(git rev-parse --abbrev-ref HEAD)
feature_files=$(git diff main..."$branch_name" --name-only)
aibundle --cli --files "$feature_files" --format markdown
```

### Automation Tools
```python
# pre-commit-hook.py
import subprocess
import sys

def bundle_changes():
    files = subprocess.check_output(['git', 'diff', '--cached', '--name-only'])
    subprocess.run(['aibundle', '--cli', '--files', files, '--format', 'markdown'])
```

## Tips & Tricks 💡

1. Version Control
   - Use consistent formats
   - Document configurations
   - Automate common tasks

2. Integration
   - Leverage git hooks
   - Integrate with tools
   - Maintain workflows

3. Performance
   - Filter effectively
   - Cache results
   - Optimize patterns
