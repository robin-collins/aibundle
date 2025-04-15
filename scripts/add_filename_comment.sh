#!/bin/bash

# Should be run from your project root.

find src -type f -name '*.rs' | while read -r file; do
    # Get path relative to project root and remove any leading ./
    rel_path="${file#./}"

    comment="// ${rel_path}"

    # Read the first line
    read -r first_line < "$file"

    # Check if first line is a comment (starts with '//')
    if [[ "$first_line" =~ ^// ]]; then
        echo "Previous comment found in $file"
        # Remove the first line, prepend the correct file path comment, and write back
        { echo "$comment"; tail -n +2 "$file"; } > "${file}.tmp" && mv "${file}.tmp" "$file"
    elif [[ "$first_line" != "$comment" ]]; then
        # No comment, just add the correct file path comment line
        { echo "$comment"; cat "$file"; } > "${file}.tmp" && mv "${file}.tmp" "$file"
        echo "Added header to $file"
    else
        echo "Header already present in $file"
    fi
done