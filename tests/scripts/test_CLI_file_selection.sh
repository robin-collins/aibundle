#!/bin/bash

# Test script for aibundle CLI: File selection options

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI File Selection Test"
echo "Starting test: $TEST_NAME"

# Setup temporary directory for any output files if needed, though these tests focus on selection
setup_temp_dir
trap cleanup_temp_dir EXIT SIGINT SIGTERM

TEST_SOURCE_DIR_RELATIVE="tests/files" # Relative to project root
OUTPUT_FILE_PROJECT_RELATIVE="$TEMP_TEST_DIR_PATH_FROM_ROOT/selection_output.md"

# Test 1: Basic SOURCE_DIR - non-recursive (default)
echo "Test 1: Basic SOURCE_DIR (non-recursive by default)"
# Expect only files/folders in the root of tests/files, not in subdirs like models/
run_aibundle_cli "$TEST_SOURCE_DIR_RELATIVE --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Test 1 Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Test 1 Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - Test 1 File Exists"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Test 1 Contains main.py"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "config.py" || print_cli_test_result "FAIL" "$TEST_NAME - Test 1 Contains config.py"
# Check for models directory - might be shown in different formats
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "models" || print_cli_test_result "FAIL" "$TEST_NAME - Test 1 Contains models dir"
# Check that a file from a subdirectory is NOT present (default non-recursive)
assert_file_contains "../../$OUTPUT_FILE_PROJECT_RELATIVE" "app_state.rs" && (
    echo -e "${RED}✗ File content for Test 1 unexpectedly contains 'app_state.rs' (should be non-recursive).${NC}"
    print_cli_test_result "FAIL" "$TEST_NAME - Test 1 Not Contains app_state.rs"
)
# Using ! grep to check for non-existence
if ! grep -q "app_state.rs" "../../$OUTPUT_FILE_PROJECT_RELATIVE"; then
    echo -e "${GREEN}✓ File does not contain 'app_state.rs' as expected (non-recursive).${NC}"
else
    echo -e "${RED}✗ File unexpectedly contains 'app_state.rs' (non-recursive).${NC}"
    print_cli_test_result "FAIL" "$TEST_NAME - Test 1 Recursive Check"
fi
echo "Test 1 passed."
rm -f "../../$OUTPUT_FILE_PROJECT_RELATIVE"

# Test 2: SOURCE_DIR with --recursive
echo "Test 2: SOURCE_DIR with --recursive"
run_aibundle_cli "$TEST_SOURCE_DIR_RELATIVE --recursive --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Test 2 Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Test 2 Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - Test 2 File Exists"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Test 2 Contains main.py"
# Look for subdirectory files that should be included when recursive
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "models/user.py" || print_cli_test_result "FAIL" "$TEST_NAME - Test 2 Contains models/user.py (recursive)"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "services/" || print_cli_test_result "FAIL" "$TEST_NAME - Test 2 Contains services directory (recursive)"
echo "Test 2 passed."
rm -f "../../$OUTPUT_FILE_PROJECT_RELATIVE"

# Test 3: --files with a single file pattern (e.g., *.py)
echo "Test 3: --files '*.py' (non-recursive by default)"
run_aibundle_cli "$TEST_SOURCE_DIR_RELATIVE --files '*.py' --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Test 3 Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Test 3 Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - Test 3 File Exists"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Test 3 Contains main.py"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "config.py" || print_cli_test_result "FAIL" "$TEST_NAME - Test 3 Contains config.py"
# Ensure non-Python files are not present
if grep -q "requirements.txt" "../../$OUTPUT_FILE_PROJECT_RELATIVE"; then
    echo -e "${RED}✗ File unexpectedly contains 'requirements.txt' for --files '*.py'.${NC}"
    print_cli_test_result "FAIL" "$TEST_NAME - Test 3 Not Contains requirements.txt"
fi
echo "Test 3 passed."
rm -f "../../$OUTPUT_FILE_PROJECT_RELATIVE"

# Test 4: --files with multiple patterns (e.g., '*.rs,*.toml') and --recursive
echo "Test 4: --files '*.rs,*.toml' --recursive"
# This will search from project root if CWD is project root. Or relative to $TEST_SOURCE_DIR_RELATIVE if specified.
# The command-line-options.md implies SOURCE_DIR is mandatory for --files, let's assume that.
run_aibundle_cli "$TEST_SOURCE_DIR_RELATIVE --files '*.rs,*.toml' --recursive --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Test 4 Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Test 4 Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - Test 4 File Exists"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "app_state.rs" || print_cli_test_result "FAIL" "$TEST_NAME - Test 4 Contains app_state.rs"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "Cargo.toml" || print_cli_test_result "FAIL" "$TEST_NAME - Test 4 Contains Cargo.toml (from project root, if applicable based on behavior)"
# Based on `command-line-options.md`, SOURCE_DIR + --files means patterns are relative to SOURCE_DIR
# So, Cargo.toml from project root should NOT be included if SOURCE_DIR is tests/files
# Let's ensure it doesn't include a Cargo.toml from outside tests/files
# This needs careful check of actual behavior - if --files always implies CWD search for those patterns.
# The spec says: "Applies glob patterns RELATIVE TO THE SOURCE_DIR", so Cargo.toml shouldn't be found from tests/files.
if grep -q "name = \"aibundle-modular\"" "../../$OUTPUT_FILE_PROJECT_RELATIVE"; then # Content from root Cargo.toml
    echo -e "${RED}✗ File unexpectedly contains root Cargo.toml details for --files relative to $TEST_SOURCE_DIR_RELATIVE.${NC}"
    print_cli_test_result "FAIL" "$TEST_NAME - Test 4 Root Cargo.toml Check"
fi
# Check a .toml file within tests/files if one existed for a more direct test
# For now, just check that Python files are NOT included
if grep -q "main.py" "../../$OUTPUT_FILE_PROJECT_RELATIVE"; then
    echo -e "${RED}✗ File unexpectedly contains 'main.py' for --files '*.rs,*.toml'.${NC}"
    print_cli_test_result "FAIL" "$TEST_NAME - Test 4 Not Contains main.py"
fi
echo "Test 4 passed."
rm -f "../../$OUTPUT_FILE_PROJECT_RELATIVE"

# Test 5: No files found with --files pattern
echo "Test 5: --files with a pattern that matches no files (e.g., '*.nonexistent')"
run_aibundle_cli "$TEST_SOURCE_DIR_RELATIVE --files '*.nonexistent' --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Test 5 Exit Code (should be 0 for no matches)"
# Depending on implementation, stderr might have a warning or be empty. stdout should be minimal.
# The command-line-options.md says output is generated. Empty structure is fine.
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Test 5 Stderr (expected empty)"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - Test 5 File Exists (empty structure)"
# Check for an empty project structure indication or lack of file list
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "Project Structure" || print_cli_test_result "FAIL" "$TEST_NAME - Test 5 Contains Project Structure header"
# And it should not contain any of the actual files from tests/files
if grep -q "main.py" "../../$OUTPUT_FILE_PROJECT_RELATIVE"; then
    echo -e "${RED}✗ File unexpectedly contains 'main.py' when no files should match '*.nonexistent'.${NC}"
    print_cli_test_result "FAIL" "$TEST_NAME - Test 5 Not Contains main.py (no match)"
fi
echo "Test 5 passed."
rm -f "../../$OUTPUT_FILE_PROJECT_RELATIVE"

# Test 6: Invalid SOURCE_DIR
echo "Test 6: Invalid SOURCE_DIR"
run_aibundle_cli "invalid/source/dir/hopefully --output-console"
assert_exit_code 1 || print_cli_test_result "FAIL" "$TEST_NAME - Test 6 Exit Code (should fail)"
assert_stderr_contains "Error: Source directory" || print_cli_test_result "FAIL" "$TEST_NAME - Test 6 Stderr Message (dir not found)"
assert_stderr_contains "does not exist or is not a directory" || print_cli_test_result "FAIL" "$TEST_NAME - Test 6 Stderr Message (dir not found detail)"
echo "Test 6 passed."

print_cli_test_result "PASS" "$TEST_NAME"