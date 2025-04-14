#!/bin/bash

# Change to the project root if not already there
cd "$(dirname "$0")" || exit 1

# Ensure GNU sed (on macOS use "brew install gnu-sed" and alias 'gsed' to 'sed')
# SED=$(which gsed || which sed)

find src -type f -name '*.rs' | while read -r file; do
    # Get the relative path from src/ onward, using forward slashes
    rel_path="${file#src/}"

    # Compose the comment line
    comment="// src/${rel_path}"

    # Check if the first line is already the intended comment
    first_line=$(head -n 1 "$file")
    if [[ "$first_line" != "$comment" ]]; then
        # Insert the comment as the first line, preserving file permissions and content
        (
            echo "$comment"
            cat "$file"
        ) > "${file}.tmp" && mv "${file}.tmp" "$file"
        echo "Added header to $file"
    else
        echo "Header already present in $file"
    fi
done