#!/bin/bash
# Disable immediate exit on error so that all tests run.
set +e

# Ensure the script is run from its own directory.
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
current_dir="$(pwd)"
if [ "$script_dir" != "$current_dir" ]; then
    echo "Error: Please run run_tests.sh from its own directory: $script_dir"
    exit 1
fi

# Remove any previous test_filesystem if it exists
rm -rf test_filesystem
echo "Creating test_filesystem structure..."

# Function to create a file with a single line: "# folderpath/filename.ext"
create_file() {
    local filepath="$1"
    mkdir -p "$(dirname "$filepath")" || { echo "Failed to create directory $(dirname "$filepath")"; exit 1; }
    echo "# ${filepath}" > "$filepath" || { echo "Failed to create file $filepath"; exit 1; }
}

# Create the required directories
mkdir -p test_filesystem/level1/level2/level3/level4/level5 || { echo "Failed to create directory test_filesystem/level1/level2/level3/level4/level5"; exit 1; }
mkdir -p test_filesystem/level1/extra || { echo "Failed to create directory test_filesystem/level1/extra"; exit 1; }
mkdir -p test_filesystem/node_modules || { echo "Failed to create directory test_filesystem/node_modules"; exit 1; }
mkdir -p test_filesystem/.git || { echo "Failed to create directory test_filesystem/.git"; exit 1; }
mkdir -p test_filesystem/target || { echo "Failed to create directory test_filesystem/target"; exit 1; }

# Create files in test_filesystem/level1
create_file "test_filesystem/level1/file1.md"
create_file "test_filesystem/level1/file2.py"
create_file "test_filesystem/level1/file3.rs"

# Create files in test_filesystem/level1/level2
create_file "test_filesystem/level1/level2/file4.md"
create_file "test_filesystem/level1/level2/file5.py"

# Create files in test_filesystem/level1/level2/level3
create_file "test_filesystem/level1/level2/level3/file6.rs"
create_file "test_filesystem/level1/level2/level3/file7.md"

# Create a file in test_filesystem/level1/level2/level3/level4
create_file "test_filesystem/level1/level2/level3/level4/file8.py"

# Create files in test_filesystem/level1/level2/level3/level4/level5
create_file "test_filesystem/level1/level2/level3/level4/level5/file9.rs"
create_file "test_filesystem/level1/level2/level3/level4/level5/file10.md"

# Create an extra file in test_filesystem/level1/extra
create_file "test_filesystem/level1/extra/file_extra.rs"

# Create ignored file at the root level
create_file "test_filesystem/ignored.txt"

# Create a .gitignore file in test_filesystem to simulate default exclusion patterns
cat <<EOF > test_filesystem/.gitignore
ignored.txt
node_modules/
target/
EOF

# Create additional ignored files inside ignored directories
create_file "test_filesystem/node_modules/ignored_module.js"
create_file "test_filesystem/.git/config"
create_file "test_filesystem/target/ignored_target.txt"

echo "Test filesystem created successfully under ./test_filesystem."

##########################################
# Run tests on the aibundle application.
##########################################

# Check if aibundle executable is available
if ! command -v aibundle &> /dev/null; then
  echo "aibundle command not found. Please ensure aibundle is built and in your PATH."
  exit 1
fi

# Initialize test counter and results array
test_counter=1
declare -a test_results

# --- Test 1: Filtering .rs files in CLI mode ---
echo "Running Test $test_counter: Filtering .rs files in CLI mode"
echo "Command: aibundle --files \"*.rs\" -d test_filesystem --output-console"
output_rs=$(aibundle --files "*.rs" -d test_filesystem --output-console 2>&1 | tee /dev/tty)
if echo "$output_rs" | grep -q "file3.rs" && \
   echo "$output_rs" | grep -q "file6.rs" && \
   echo "$output_rs" | grep -q "file9.rs"; then
    test_results[$test_counter]="✅passed  Test $test_counter: Filtering .rs files in CLI mode"
else
    test_results[$test_counter]="❌failed  Test $test_counter: Filtering .rs files in CLI mode"
fi
((test_counter++))

# --- Test 2: (Removed) Search functionality test is not applicable in CLI mode ---
echo "Running Test $test_counter: (Removed) Search functionality test not applicable in CLI mode"
echo "Command: (N/A)"
test_results[$test_counter]="✅skipped  Test $test_counter: (Removed) Search functionality test not applicable in CLI mode"
((test_counter++))

# --- Test 3: Recursive processing check (should include deeply nested files) ---
echo "Running Test $test_counter: Recursive processing check"
echo "Command: aibundle --files \"*.*\" -d test_filesystem --output-console"
output_recursive=$(aibundle --files "*.*" -d test_filesystem --output-console 2>&1 | tee /dev/tty)
if echo "$output_recursive" | grep -q "level5/file9.rs"; then
    test_results[$test_counter]="✅passed  Test $test_counter: Recursive processing check (deeply nested files)"
else
    test_results[$test_counter]="❌failed  Test $test_counter: Recursive processing check (deeply nested files)"
fi
((test_counter++))

# --- Test 4: Verify that ignored files are not included ---
echo "Running Test $test_counter: Ignored files check"
echo "Command: (Using previous Recursive processing output)"
if echo "$output_recursive" | grep -q "ignored.txt"; then
    test_results[$test_counter]="❌failed  Test $test_counter: Ignored files check"
else
    test_results[$test_counter]="✅passed  Test $test_counter: Ignored files check"
fi
((test_counter++))

# --- Test 5: Output redirection to file for .py files ---
echo "Running Test $test_counter: Output redirection to file for .py files"
echo "Command: aibundle --files \"*.py\" -d test_filesystem --output-file test_output.txt"
output_file="test_output.txt"
aibundle --files "*.py" -d test_filesystem --output-file "$output_file" 2>&1 | tee /dev/tty
if [ -f "$output_file" ] && grep -q "file2.py" "$output_file"; then
    test_results[$test_counter]="✅passed  Test $test_counter: Output redirection to file for .py files"
else
    test_results[$test_counter]="❌failed  Test $test_counter: Output redirection to file for .py files"
fi
((test_counter++))

# --- Test 6: XML format test ---
echo "Running Test $test_counter: XML format test"
echo "Command: aibundle --files \"*.py\" -d test_filesystem --format xml --output-console"
output_xml=$(aibundle --files "*.py" -d test_filesystem --format xml --output-console 2>&1 | tee /dev/tty)
if echo "$output_xml" | grep -q "<file name=\"level1/file2.py\">"; then
    test_results[$test_counter]="✅passed  Test $test_counter: XML format test"
else
    test_results[$test_counter]="❌failed  Test $test_counter: XML format test"
fi
((test_counter++))

# --- Test 7: Markdown format test ---
echo "Running Test $test_counter: Markdown format test"
echo "Command: aibundle --files \"*.py\" -d test_filesystem --format markdown --output-console"
output_markdown=$(aibundle --files "*.py" -d test_filesystem --format markdown --output-console 2>&1 | tee /dev/tty)
if echo "$output_markdown" | grep -q '```level1/file2.py'; then
    test_results[$test_counter]="✅passed  Test $test_counter: Markdown format test"
else
    test_results[$test_counter]="❌failed  Test $test_counter: Markdown format test"
fi
((test_counter++))

# --- Test 8: JSON format test ---
echo "Running Test $test_counter: JSON format test"
echo "Command: aibundle --files \"*.py\" -d test_filesystem --format json --output-console"
output_json=$(aibundle --files "*.py" -d test_filesystem --format json --output-console 2>&1 | tee /dev/tty)
if command -v jq &> /dev/null; then
    # Check if the JSON output is valid and contains the expected file path
    echo "$output_json" | jq . > /dev/null 2>&1
    if [ $? -eq 0 ] && echo "$output_json" | jq '.[].path' | grep -q "level1/file2.py"; then
        test_results[$test_counter]="✅passed  Test $test_counter: JSON format test"
    else
        test_results[$test_counter]="❌failed  Test $test_counter: JSON format test"
    fi
else
    test_results[$test_counter]="❌failed (jq not installed)  Test $test_counter: JSON format test"
fi
((test_counter++))

# --- Test 9: Version output test ---
echo "Running Test $test_counter: Version output test"
echo "Command: aibundle --version"
output_version=$(aibundle --version 2>&1 | tee /dev/tty)
# Expected version from the .version file is 0.6.7 (or the current version)
if echo "$output_version" | grep -q "0.6.7"; then
    test_results[$test_counter]="✅passed  Test $test_counter: Version output test"
else
    test_results[$test_counter]="❌failed  Test $test_counter: Version output test"
fi
((test_counter++))

# --- Test 10: --gitignore false test ---
echo "Running Test $test_counter: --gitignore false test"
echo "Command: aibundle --files \"*.*\" -d test_filesystem --gitignore false --output-console"
output_gitignore=$(aibundle --files "*.*" -d test_filesystem --gitignore false --output-console 2>&1 | tee /dev/tty)
# With gitignore disabled, the output should include "ignored.txt"
if echo "$output_gitignore" | grep -q "ignored.txt"; then
    test_results[$test_counter]="✅passed  Test $test_counter: --gitignore false test"
else
    test_results[$test_counter]="❌failed  Test $test_counter: --gitignore false test"
fi
((test_counter++))

# --- Test 11: --ignore custom test ---
echo "Running Test $test_counter: --ignore custom test"
echo "Command: aibundle --files \"*.*\" -d test_filesystem --gitignore false --ignore dummy --output-console"
output_ignore=$(aibundle --files "*.*" -d test_filesystem --gitignore false --ignore dummy --output-console 2>&1 | tee /dev/tty)
if echo "$output_ignore" | grep -q "ignored.txt"; then
    test_results[$test_counter]="✅passed  Test $test_counter: --ignore custom test"
else
    test_results[$test_counter]="❌failed  Test $test_counter: --ignore custom test"
fi
((test_counter++))

##########################################
# Summary of test results
##########################################
echo ""
echo "Test Summary:"
for test in "${test_results[@]}"; do
  echo "$test"
done

# Count failures
fail_count=0
for result in "${test_results[@]}"; do
  if echo "$result" | grep -q "❌failed"; then
    ((fail_count++))
  fi
done

if [ $fail_count -eq 0 ]; then
  echo ""
  echo "All tests passed successfully."
else
  echo ""
  echo "$fail_count test(s) failed."
fi

# Note:
# These tests cover core CLI functionality for aibundle, including file filtering,
# recursive directory traversal, various output formatting options (XML, Markdown, JSON),
# version output, and toggling ignore behaviors via --gitignore and --ignore.
# Comprehensive testing of the interactive TUI features (such as real-time search and modal dialogs)
# should be performed manually.