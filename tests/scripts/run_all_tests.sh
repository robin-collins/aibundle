#!/bin/bash

# Main script to run all test suites (TUI and CLI)

# Ensure we are in the tests/scripts directory
cd "$(dirname "$0")" || exit 1

# Make all test scripts executable
chmod +x ./*.sh

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

LOG_DIR="../../logs"
MAIN_LOG_FILE="$LOG_DIR/test_run_$(date +%Y%m%d_%H%M%S).log"

# if log_dir does not exist, create it, else delete all files in it after user confirmation
if [ ! -d "$LOG_DIR" ]; then
    mkdir -p "$LOG_DIR"
else
    echo "Deleting old logs in $LOG_DIR"
    rm -rf "${LOG_DIR}/*.log"

fi

# Function to log messages to both console and main log file
log_message() {
    echo -e "$1" | tee -a "$MAIN_LOG_FILE"
}

log_message "Starting all tests..."

# Delete old captured panes
current_dir=$(pwd)
cd "../captured-panes" || exit 1
captured_panes_folder=$(pwd)

echo "Deleting old captured panes in $captured_panes_folder"
    rm -rf ./*

cd "$current_dir" || exit 1

# --- TUI Tests ---
log_message "\n${YELLOW}Running TUI Tests...${NC}"

TUI_TESTS=(
    "test_A1_default_startup.sh"
    "test_A2_startup_specific_dir.sh"
    "test_B1_list_navigation.sh"
    "test_B2_directory_traversal.sh"
    "test_C1_single_selection.sh"
    "test_C2_select_all.sh"
    "test_D1_search_functionality.sh"
    "test_E1_toggle_format.sh"
    "test_G1_help_modal.sh"
    "test_I1_quit.sh"
    # Add new TUI test scripts here
)

FAILED_TUI_TESTS=0
PASSED_TUI_TESTS=0

for test_script in "${TUI_TESTS[@]}"; do
    log_message "\nExecuting TUI test: $test_script"
    # Run in a subshell to handle exit codes properly
    (./"$test_script" >> "$MAIN_LOG_FILE" 2>&1)
    if [ $? -eq 0 ]; then
        log_message "${GREEN}✓ TUI Test PASSED: $test_script${NC}"
        PASSED_TUI_TESTS=$((PASSED_TUI_TESTS + 1))
    else
        log_message "${RED}✗ TUI Test FAILED: $test_script. See $MAIN_LOG_FILE for details.${NC}"
        FAILED_TUI_TESTS=$((FAILED_TUI_TESTS + 1))
    fi
done

# --- CLI Tests ---
log_message "\n${YELLOW}Running CLI Tests...${NC}"

CLI_TESTS=(
    "test_CLI_A1_version.sh"
    "test_CLI_A2_help.sh"
    "test_CLI_B1_source_dir.sh"
    "test_CLI_B2_source_dir_nonexistent.sh"
    "test_CLI_C1_files_pattern.sh"
    "test_CLI_C2_files_multiple_patterns.sh"
    "test_CLI_C3_recursive.sh"
    "test_CLI_C4_no_matches.sh"
    "test_CLI_D1_output_file.sh"
    "test_CLI_D2_output_console.sh"
    "test_CLI_D3_output_both.sh"
    "test_CLI_E1_format_json.sh"
    "test_CLI_E2_format_xml.sh"
    # Add new CLI test scripts here
)

FAILED_CLI_TESTS=0
PASSED_CLI_TESTS=0

for test_script in "${CLI_TESTS[@]}"; do
    log_message "\nExecuting CLI test: $test_script"
    (./"$test_script" >> "$MAIN_LOG_FILE" 2>&1)
    if [ $? -eq 0 ]; then
        log_message "${GREEN}✓ CLI Test PASSED: $test_script${NC}"
        PASSED_CLI_TESTS=$((PASSED_CLI_TESTS + 1))
    else
        log_message "${RED}✗ CLI Test FAILED: $test_script. See $MAIN_LOG_FILE for details.${NC}"
        FAILED_CLI_TESTS=$((FAILED_CLI_TESTS + 1))
    fi
done

# --- Summary ---
log_message "\n${YELLOW}Test Summary:${NC}"
log_message "TUI Tests: Passed $PASSED_TUI_TESTS, Failed $FAILED_TUI_TESTS"
log_message "CLI Tests: Passed $PASSED_CLI_TESTS, Failed $FAILED_CLI_TESTS"

TOTAL_FAILED=$((FAILED_TUI_TESTS + FAILED_CLI_TESTS))

if [ $TOTAL_FAILED -eq 0 ]; then
    log_message "${GREEN}All tests passed successfully!${NC}"
    exit 0
else
    log_message "${RED}$TOTAL_FAILED test(s) failed. Check $MAIN_LOG_FILE for details.${NC}"
    exit 1
fi