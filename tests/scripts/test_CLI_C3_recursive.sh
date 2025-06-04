#!/bin/bash

# Test script for aibundle CLI: --recursive option

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Recursive Option"
echo "Starting test: $TEST_NAME"

# Setup temporary directory for output files
setup_temp_dir
trap cleanup_temp_dir EXIT SIGINT SIGTERM

TEST_SOURCE_DIR="tests/files" # Relative to project root
OUTPUT_FILE_PROJECT_RELATIVE="$TEMP_TEST_DIR_PATH_FROM_ROOT/recursive_output.txt"

# Test: SOURCE_DIR with --recursive
echo "Test: SOURCE_DIR with --recursive"
run_aibundle_cli "$TEST_SOURCE_DIR --recursive --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - File Exists"

# Check for root files
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains main.py"

# Look for subdirectory files that should be included when recursive
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "models/user.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains models/user.py (recursive)"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "services/" || print_cli_test_result "FAIL" "$TEST_NAME - Contains services directory (recursive)"
echo "--recursive test passed."

print_cli_test_result "PASS" "$TEST_NAME"