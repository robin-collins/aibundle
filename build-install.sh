#!/usr/bin/env bash

# --- Version Bump Section ---
# Define file paths (adjust if main.rs is located elsewhere)
main_rs="src/main.rs"
cargo_toml="Cargo.toml"

# Check and update version in main.rs
if [[ -f "$main_rs" ]]; then
    # Extract version from main.rs (e.g., 1.2.3)
    version=$(grep 'const VERSION: &str' "$main_rs" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
    if [[ -n "$version" ]]; then
        IFS='.' read -r major minor patch <<< "$version"
        patch=$((patch + 1))
        new_version="${major}.${minor}.${patch}"
        echo "Updating version in ${main_rs}: ${version} -> ${new_version}"
        
        # Update the version line in main.rs using sed
        sed -i.bak -E "s/(const VERSION: &str = \")([0-9]+\.[0-9]+\.[0-9]+)(\";)/\1${new_version}\3/" "$main_rs"
        rm -f "${main_rs}.bak"
    else
        echo "Error: Could not find version string in ${main_rs}."
        exit 1
    fi
else
    echo "Error: ${main_rs} file not found."
    exit 1
fi

# Check and update version in Cargo.toml
if [[ -f "$cargo_toml" ]]; then
    echo "Updating version in ${cargo_toml} to ${new_version}"
    sed -i.bak -E "s/(^version *= *\")([0-9]+\.[0-9]+\.[0-9]+)(\")/\1${new_version}\3/" "$cargo_toml"
    rm -f "${cargo_toml}.bak"
else
    echo "Error: ${cargo_toml} file not found."
    exit 1
fi

# ---------------------------

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cargo fmt && cargo build --release && sudo cp target/release/aibundle /usr/local/bin/aibundle
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    cargo fmt && cargo build --release && cp target/release/aibundle.exe /c/tools/bin/
else
    echo "Unsupported OS"
    exit 1
fi
