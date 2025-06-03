#!/bin/bash

# Main test runner for AIBundle TUI testing
# Executes all individual test scripts and provides summary

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
CAPTURE_DIR="$(pwd)/tests/captured-panes"
RESULTS_FILE="$CAPTURE_DIR/test_results_$(date +%Y%m%d_%H%M%S).txt"

# Ensure directories exist
mkdir -p "$CAPTURE_DIR"
mkdir -p "$(dirname "$RESULTS_FILE")"

# Test tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
declare -a FAILED_TEST_NAMES=()

echo "=============================================="
echo -e "${BLUE}AIBundle TUI Test Suite${NC}"
echo "=============================================="
echo "Starting comprehensive TUI testing..."
echo "Results will be saved to: $RESULTS_FILE"
echo ""

# Start results file
{
    echo "AIBundle TUI Test Results"
    echo "========================="
    echo "Test run started: $(date)"
    echo ""
} > "$RESULTS_FILE"

# Function to run a single test
run_test() {
    local test_script="$1"
    local test_name="$(basename "$test_script" .sh)"

    echo -e "${BLUE}Running test: $test_name${NC}"
    echo "----------------------------------------"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    # Run the test and capture output
    local test_output
    local test_exit_code

    if [ -f "$test_script" ] && [ -x "$test_script" ]; then
        test_output=$(cd "$(dirname "$SCRIPT_DIR")" && timeout 120 "$test_script" 2>&1)
        test_exit_code=$?

        if [ $test_exit_code -eq 0 ]; then
            echo -e "${GREEN}✓ PASSED: $test_name${NC}"
            PASSED_TESTS=$((PASSED_TESTS + 1))

            # Log to results file
            {
                echo "PASS: $test_name"
                echo "Output:"
                echo "$test_output"
                echo ""
            } >> "$RESULTS_FILE"
        else
            echo -e "${RED}✗ FAILED: $test_name (exit code: $test_exit_code)${NC}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            FAILED_TEST_NAMES+=("$test_name")

            # Log to results file
            {
                echo "FAIL: $test_name (exit code: $test_exit_code)"
                echo "Output:"
                echo "$test_output"
                echo ""
            } >> "$RESULTS_FILE"

            # Show first few lines of failure output
            echo -e "${YELLOW}Failure output (first 10 lines):${NC}"
            echo "$test_output" | head -10
        fi
    else
        echo -e "${RED}✗ SKIPPED: $test_name (script not found or not executable)${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_TEST_NAMES+=("$test_name (not executable)")

        # Log to results file
        {
            echo "SKIP: $test_name (script not found or not executable)"
            echo ""
        } >> "$RESULTS_FILE"
    fi

    echo ""
    sleep 1  # Brief pause between tests
}

# Check prerequisites
echo "Checking prerequisites..."

# Check if tmux is installed
if ! command -v tmux &> /dev/null; then
    echo -e "${RED}ERROR: tmux is not installed or not in PATH${NC}"
    echo "Please install tmux to run these tests"
    exit 1
fi



# Check if the application binary exists
cd "tests/files" 2>/dev/null || {
    echo -e "${RED}ERROR: tests/files directory not found${NC}"
    echo "Please run this script from the project root directory"
    exit 1
}

if [ ! -f "../../target/release/aibundle" ]; then
    echo -e "${YELLOW}WARNING: Release binary not found, checking for debug binary...${NC}"
    if [ ! -f "../../target/debug/aibundle" ]; then
        echo -e "${RED}ERROR: Neither release nor debug binary found${NC}"
        echo "Please build the application first:"
        echo "  cargo build --release"
        exit 1
    else
        echo -e "${GREEN}Debug binary found, tests will use debug build${NC}"
    fi
else
    echo -e "${GREEN}Release binary found${NC}"
fi

cd - > /dev/null

echo -e "${GREEN}Prerequisites check passed${NC}"
echo ""

# List of test scripts to run (in order)
TEST_SCRIPTS=(
    "$SCRIPT_DIR/test_A1_default_startup.sh"
    "$SCRIPT_DIR/test_A2_startup_specific_dir.sh"
    "$SCRIPT_DIR/test_B1_list_navigation.sh"
    "$SCRIPT_DIR/test_B2_directory_traversal.sh"
    "$SCRIPT_DIR/test_C1_single_selection.sh"
    "$SCRIPT_DIR/test_C2_select_all.sh"
    "$SCRIPT_DIR/test_D1_search_functionality.sh"
    "$SCRIPT_DIR/test_E1_toggle_format.sh"
    "$SCRIPT_DIR/test_G1_help_modal.sh"
    "$SCRIPT_DIR/test_I1_quit.sh"
)

# Make sure test scripts are executable
echo "Setting execute permissions on test scripts..."
for script in "${TEST_SCRIPTS[@]}"; do
    if [ -f "$script" ]; then
        chmod +x "$script"
    fi
done

echo ""

# Run all tests
echo "Running test suite..."
echo ""

for test_script in "${TEST_SCRIPTS[@]}"; do
    run_test "$test_script"
done

# Generate summary
echo "=============================================="
echo -e "${BLUE}TEST SUMMARY${NC}"
echo "=============================================="
echo "Total tests: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

if [ $FAILED_TESTS -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed tests:${NC}"
    for failed_test in "${FAILED_TEST_NAMES[@]}"; do
        echo -e "  ${RED}• $failed_test${NC}"
    done
fi

# Calculate success rate
if [ $TOTAL_TESTS -gt 0 ]; then
    SUCCESS_RATE=$(( (PASSED_TESTS * 100) / TOTAL_TESTS ))
    echo ""
    echo "Success rate: $SUCCESS_RATE%"
fi

echo ""
echo "Detailed results saved to: $RESULTS_FILE"

# Final summary to results file
{
    echo ""
    echo "TEST SUMMARY"
    echo "============"
    echo "Total tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    if [ $TOTAL_TESTS -gt 0 ]; then
        echo "Success rate: $(( (PASSED_TESTS * 100) / TOTAL_TESTS ))%"
    fi
    echo ""
    echo "Test run completed: $(date)"
} >> "$RESULTS_FILE"

# Cleanup any leftover tmux sessions
echo "Cleaning up any leftover tmux sessions..."
tmux list-sessions 2>/dev/null | grep "test_" | cut -d: -f1 | xargs -r -I {} tmux kill-session -t {}

echo "=============================================="

# Exit with appropriate code
if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}All tests passed! 🎉${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Check the results file for details.${NC}"
    exit 1
fi