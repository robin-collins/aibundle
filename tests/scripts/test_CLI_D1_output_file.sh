#!/bin/bash

# Test script for aibundle CLI: --output-file option

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Output File Option"
echo "Starting test: $TEST_NAME"

# Setup temporary directory for output files
setup_temp_dir
trap cleanup_temp_dir EXIT SIGINT SIGTERM

TEST_SOURCE_DIR="tests/files" # Relative to project root
OUTPUT_FILE_PROJECT_RELATIVE="$TEMP_TEST_DIR_PATH_FROM_ROOT/output_file.txt"

# Test: --output-file with default format
echo "Test: --output-file (default format)"
run_aibundle_cli "$TEST_SOURCE_DIR --output-file $OUTPUT_FILE_PROJECT_RELATIVE"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - File Exists"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains main.py"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "config.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains config.py"
# Verify no console output when --output-file is used without --output-console
assert_stdout_not_contains "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - No Console Output"
echo "--output-file test passed."

print_cli_test_result "PASS" "$TEST_NAME"