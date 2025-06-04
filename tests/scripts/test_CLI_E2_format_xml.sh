#!/bin/bash

# Test script for aibundle CLI: --format xml option

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Format XML"
echo "Starting test: $TEST_NAME"

# Setup temporary directory for output files
setup_temp_dir
trap cleanup_temp_dir EXIT SIGINT SIGTERM

TEST_SOURCE_DIR="tests/files" # Relative to project root
OUTPUT_FILE_PROJECT_RELATIVE="$TEMP_TEST_DIR_PATH_FROM_ROOT/format_xml_output.txt"

# Test: --format xml
echo "Test: --format xml"
run_aibundle_cli "$TEST_SOURCE_DIR --output-file $OUTPUT_FILE_PROJECT_RELATIVE --format xml"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Stderr"
assert_file_exists "$OUTPUT_FILE_PROJECT_RELATIVE" || print_cli_test_result "FAIL" "$TEST_NAME - File Exists"

# Check for XML structure
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "<" || print_cli_test_result "FAIL" "$TEST_NAME - Contains XML opening tag"
assert_file_contains "$OUTPUT_FILE_PROJECT_RELATIVE" "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains main.py"
assert_stdout_not_contains "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - No Console Output"
echo "--format xml test passed."

print_cli_test_result "PASS" "$TEST_NAME"