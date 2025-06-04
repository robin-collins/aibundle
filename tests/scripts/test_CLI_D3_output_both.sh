#!/bin/bash

# Test script for aibundle CLI: Both --output-file and --output-console options

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Both Output Options"
echo "Starting test: $TEST_NAME"

# Setup temporary directory for output files
setup_temp_dir
trap cleanup_temp_dir EXIT SIGINT SIGTERM

TEST_SOURCE_DIR="tests/files" # Relative to project root
OUTPUT_FILE_PROJECT_RELATIVE="$TEMP_TEST_DIR_PATH_FROM_ROOT/both_output.txt"

# Test: --output-file AND --output-console together
echo "Test: --output-file and --output-console together"
run_aibundle_cli "$TEST_SOURCE_DIR --output-file $OUTPUT_FILE_PROJECT_RELATIVE --output-console"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Stderr"

# Check file
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - File Exists"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - File Contains main.py"

# Check console
assert_stdout_contains "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Console Contains main.py"
echo "Both output options test passed."

print_cli_test_result "PASS" "$TEST_NAME"