#!/usr/bin/env bash

has_file_changed() {
    local filepath="$1"

    # Check if the file is identical to HEAD (no unstaged changes)
    if git diff --quiet HEAD -- "$filepath"; then
        # File is unchanged compared to the last commit.
        return 1
    else
        # File has unstaged changes.
        # Now, check if these changes have been staged.
        if git diff --name-only --cached -- "$filepath" | grep -q "$filepath"; then
            # If the file is staged, then we do not treat it as changed.
            return 1
        else
            # File has unstaged changes and is not staged.
            return 0
        fi
    fi
}

# --- Version Bump Section ---
# Define file paths (adjust if main.rs is located elsewhere)
main_rs="src/main.rs"
cargo_toml="Cargo.toml"

main() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        cargo fmt && cargo build --release && sudo cp target/release/aibundle /usr/local/bin/aibundle
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        # first check if the version file has changed
        if has_file_changed ".version"; then
            echo "version file has changed since last commit, not updating"
        else
            echo "version file has not changed since last commit, updating patch number"
            # update the .version file's patch number
            source ".version"
            new_patch=$((vpatch + 1))
            sed -i.bak -E "s/(^vpatch *= *)([0-9]+)/\1${new_patch}/" ".version"
            # check if the new patch number got updated
            source ".version"
            if [[ "$vpatch" != "$new_patch" ]]; then
                echo "failed to update patch number"
                exit 1
            else
                echo "Updated patch number."
                rm -f ".version.bak"
            fi
        fi

        # Check if the version in main.rs matches vmajor.vminor.vpatch, if not, update it
        source ".version"

        if [[ -f "$main_rs" ]]; then
            # Extract version from main.rs (e.g., 1.2.3)
            version=$(grep 'const VERSION: &str' "$main_rs" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
            if [[ -n "$version" ]]; then
                IFS='.' read -r major minor patch <<<"$version"
                if [[ "$major" != "$vmajor" || "$minor" != "$vminor" || "$patch" != "$vpatch" ]]; then
                    patch=$((patch + 1))
                    new_version="${vmajor}.${vminor}.${patch}"
                    echo "Updating version in ${main_rs}: ${version} -> ${new_version}"

                    # Update the version line in main.rs using sed
                    sed -i.bak -E "s/(const VERSION: &str = \")([0-9]+\.[0-9]+\.[0-9]+)(\";)/\1${new_version}\3/" "$main_rs"
                    rm -f "${main_rs}.bak"
                else
                    echo "Version in ${main_rs} is up to date."
                fi
            else
                echo "Error: Could not find version string in ${main_rs}."
                exit 1
            fi
        else
            echo "Error: ${main_rs} file not found."
            exit 1
        fi

        # Check if version in cargo.toml matches .version, if not update it
        if [[ -f "$cargo_toml" ]]; then
            source ".version"

            cversion=$(grep 'version' "$cargo_toml" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
            # split cversion into major, minor, patch
            IFS='.' read -r cmajor cminor cpatch <<<"$cversion"
            if [[ "$cmajor" != "$vmajor" || "$cminor" != "$vminor" || "$cpatch" != "$vpatch" ]]; then
                echo "Version in ${cargo_toml} is not up to date."
                echo "Updating version in ${cargo_toml} to ${vmajor}.${vminor}.${vpatch}"
                sed -i.bak -E "s/(^version *= *\")([0-9]+\.[0-9]+\.[0-9]+)(\")/\1${vmajor}.${vminor}.${vpatch}\3/" "$cargo_toml"
                rm -f "${cargo_toml}.bak"
            else
                echo "Version in ${cargo_toml} is up to date."
            fi

        else
            echo "Error: ${cargo_toml} file not found."
            exit 1
        fi

        # build and install
        cargo fmt && cargo build --release && sudo cp target/release/aibundle /c/tools/bin/aibundle.exe

    else
        echo "Unsupported OS"
        exit 1
    fi

}

main
