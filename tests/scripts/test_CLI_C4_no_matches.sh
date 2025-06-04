#!/bin/bash

# Test script for aibundle CLI: --files with pattern that matches no files

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI No Matching Files"
echo "Starting test: $TEST_NAME"

# Setup temporary directory for output files
setup_temp_dir
trap cleanup_temp_dir EXIT SIGINT SIGTERM

TEST_SOURCE_DIR="tests/files" # Relative to project root
OUTPUT_FILE_PROJECT_RELATIVE="$TEMP_TEST_DIR_PATH_FROM_ROOT/no_matches_output.txt"

# Test: --files with a pattern that matches no files
echo "Test: --files with a pattern that matches no files (e.g., '*.nonexistent')"
run_aibundle_cli "$TEST_SOURCE_DIR --files '*.nonexistent' --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Exit Code (should be 0 for no matches)"
# The application should produce an empty output or empty structure, but not fail
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Stderr (expected empty)"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - File Exists (empty structure)"

# And it should not contain any of the actual files from tests/files
if grep -q "main.py" "../../$OUTPUT_FILE_PROJECT_RELATIVE"; then
    echo -e "${RED}✗ File unexpectedly contains 'main.py' when no files should match '*.nonexistent'.${NC}"
    print_cli_test_result "FAIL" "$TEST_NAME - Should Not Contain main.py (no match)"
fi
echo "No matches test passed."

print_cli_test_result "PASS" "$TEST_NAME"