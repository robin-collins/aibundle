#!/bin/bash

# Test script for aibundle CLI: --version option
# Tests both --version and -V for showing version information

# shellcheck disable=SC1091
source "$(dirname "$0")/common_cli_test_functions.sh"

TEST_NAME="CLI Version Check"
echo "Starting test: $TEST_NAME"

# Test 1: --version
echo "Test 1: Checking --version"
run_aibundle_cli "--version"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Version Check"
assert_stdout_contains "aibundle" || print_cli_test_result "FAIL" "$TEST_NAME - Version Output"
# Version format is typically X.Y.Z, so check for at least one dot
assert_stdout_contains "." || print_cli_test_result "FAIL" "$TEST_NAME - Version Format"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Version Stderr"
echo "Version test passed."

# Test 2: -V (short for --version)
echo "Test 2: Checking -V"
run_aibundle_cli "-V"
assert_exit_code 0 || print_cli_test_result "FAIL" "$TEST_NAME - Short Version Check"
assert_stdout_contains "aibundle" || print_cli_test_result "FAIL" "$TEST_NAME - Short Version Output"
assert_stdout_contains "." || print_cli_test_result "FAIL" "$TEST_NAME - Short Version Format"
assert_stderr_empty || print_cli_test_result "FAIL" "$TEST_NAME - Short Version Stderr"
echo "Short version test passed."

print_cli_test_result "PASS" "$TEST_NAME"