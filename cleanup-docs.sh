#!/bin/bash

# Create necessary directories first
mkdir -p docs/src/{cli,tui,advanced,reference,configuration}

# Remove all .md files from docs/src except SUMMARY.md
find docs/src -maxdepth 1 -type f -name "*.md" ! -name "SUMMARY.md" -exec rm {} +

# Remove all files from subdirectories
find docs/src/cli -type f -delete
find docs/src/tui -type f -delete
find docs/src/advanced -type f -delete
find docs/src/reference -type f -delete
find docs/src/configuration -type f -delete

# Create empty files for our new structure
touch docs/src/introduction.md
touch docs/src/quick-start.md
touch docs/src/installation.md
touch docs/src/configuration.md
touch docs/src/contributing.md
touch docs/src/changelog.md

# CLI section
touch docs/src/cli/{index,basic-usage,file-selection,output-formats,filtering,advanced}.md

# TUI section
touch docs/src/tui/{index,navigation,selection,operations,search,output,shortcuts,indicators}.md

# Configuration section
touch docs/src/configuration/{initial-setup,config-file,default-settings}.md

# Advanced section
touch docs/src/advanced/{integration,ai-integration,vcs,large-projects,performance}.md

# Reference section
touch docs/src/reference/{cli,configuration,keyboard,formats,icons,troubleshooting}.md

echo "Documentation structure cleaned and reset!" 