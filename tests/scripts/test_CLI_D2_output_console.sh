#!/bin/bash

# Test script for aibundle CLI: --output-console option

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Output Console Option"
echo "Starting test: $TEST_NAME"

TEST_SOURCE_DIR="tests/files" # Relative to project root

# Test: --output-console with default format
echo "Test: --output-console (default format)"
run_aibundle_cli "$TEST_SOURCE_DIR --output-console"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Exit Code"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Stderr"
assert_stdout_contains "main.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains main.py"
assert_stdout_contains "config.py" || print_cli_test_result "FAIL" "$TEST_NAME - Contains config.py"
echo "--output-console test passed."

print_cli_test_result "PASS" "$TEST_NAME"