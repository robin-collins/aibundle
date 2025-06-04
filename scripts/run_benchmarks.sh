#!/bin/bash

# AIBundle Benchmarking Script
# This script runs all benchmarks and generates reports

set -e

echo "🚀 AIBundle Performance Benchmarking Suite"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create reports directory
REPORTS_DIR="benchmark_reports"
mkdir -p "$REPORTS_DIR"

# Get timestamp for report naming
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

echo -e "${BLUE}📊 Starting benchmark suite at $(date)${NC}"
echo ""

# Function to run a benchmark and capture output
run_benchmark() {
    local bench_name=$1
    local description=$2

    echo -e "${YELLOW}🔧 Running $description...${NC}"

    # Run benchmark with HTML report generation
    if cargo bench --bench "$bench_name" -- --output-format html > "$REPORTS_DIR/${bench_name}_${TIMESTAMP}.log" 2>&1; then
        echo -e "${GREEN}✅ $description completed successfully${NC}"

        # Move HTML report if generated
        if [ -d "target/criterion" ]; then
            cp -r target/criterion "$REPORTS_DIR/${bench_name}_${TIMESTAMP}_html"
        fi
    else
        echo -e "${RED}❌ $description failed${NC}"
        echo "Check $REPORTS_DIR/${bench_name}_${TIMESTAMP}.log for details"
    fi
    echo ""
}

# Function to run quick benchmarks (shorter duration)
run_quick_benchmark() {
    local bench_name=$1
    local description=$2

    echo -e "${YELLOW}⚡ Running $description (quick mode)...${NC}"

    # Run benchmark with reduced sample size for faster execution
    if cargo bench --bench "$bench_name" -- --quick > "$REPORTS_DIR/${bench_name}_quick_${TIMESTAMP}.log" 2>&1; then
        echo -e "${GREEN}✅ $description (quick) completed successfully${NC}"
    else
        echo -e "${RED}❌ $description (quick) failed${NC}"
        echo "Check $REPORTS_DIR/${bench_name}_quick_${TIMESTAMP}.log for details"
    fi
    echo ""
}

# Check if quick mode is requested
QUICK_MODE=false
if [[ "$1" == "--quick" || "$1" == "-q" ]]; then
    QUICK_MODE=true
    echo -e "${BLUE}⚡ Running in quick mode (reduced sample sizes)${NC}"
    echo ""
fi

# Build the project first
echo -e "${BLUE}🔨 Building project...${NC}"
if cargo build --release; then
    echo -e "${GREEN}✅ Build completed successfully${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo ""

# Run benchmarks
if [ "$QUICK_MODE" = true ]; then
    run_quick_benchmark "filesystem_bench" "File System Operations Benchmark"
    run_quick_benchmark "config_bench" "Configuration Loading Benchmark"
    run_quick_benchmark "tui_bench" "TUI Operations Benchmark"
    run_quick_benchmark "memory_bench" "Memory Usage Benchmark"
else
    run_benchmark "filesystem_bench" "File System Operations Benchmark"
    run_benchmark "config_bench" "Configuration Loading Benchmark"
    run_benchmark "tui_bench" "TUI Operations Benchmark"
    run_benchmark "memory_bench" "Memory Usage Benchmark"
fi

# Generate summary report
echo -e "${BLUE}📋 Generating summary report...${NC}"

SUMMARY_FILE="$REPORTS_DIR/benchmark_summary_${TIMESTAMP}.md"

cat > "$SUMMARY_FILE" << EOF
# AIBundle Benchmark Summary

**Generated:** $(date)
**Mode:** $([ "$QUICK_MODE" = true ] && echo "Quick" || echo "Full")

## Benchmark Results

### File System Operations
- **Purpose:** Measures file listing, directory traversal, and path operations
- **Key Metrics:** Throughput for different directory sizes, file operation latency
- **Report:** filesystem_bench_${TIMESTAMP}.log

### Configuration Loading
- **Purpose:** Measures config loading, serialization, and validation performance
- **Key Metrics:** Config loading time, TOML parsing speed, validation overhead
- **Report:** config_bench_${TIMESTAMP}.log

### TUI Operations
- **Purpose:** Measures UI state management, navigation, and rendering simulation
- **Key Metrics:** State update speed, selection operations, search performance
- **Report:** tui_bench_${TIMESTAMP}.log

### Memory Usage
- **Purpose:** Measures memory allocation patterns and data structure efficiency
- **Key Metrics:** Allocation overhead, memory fragmentation, cache performance
- **Report:** memory_bench_${TIMESTAMP}.log

## Performance Baseline

These benchmarks establish performance baselines for:
- File system scanning operations
- Configuration management overhead
- UI responsiveness metrics
- Memory usage patterns

## Interpreting Results

- **Throughput:** Higher is better (operations per second)
- **Latency:** Lower is better (time per operation)
- **Memory:** Lower allocation and fragmentation is better

## Next Steps

1. Compare results against previous benchmarks
2. Identify performance regressions
3. Focus optimization efforts on bottlenecks
4. Re-run benchmarks after optimizations

EOF

echo -e "${GREEN}✅ Summary report generated: $SUMMARY_FILE${NC}"
echo ""

# Display quick results summary
echo -e "${BLUE}📈 Quick Results Summary${NC}"
echo "========================"

for log_file in "$REPORTS_DIR"/*_${TIMESTAMP}.log; do
    if [ -f "$log_file" ]; then
        bench_name=$(basename "$log_file" | cut -d'_' -f1)
        echo -e "${YELLOW}$bench_name:${NC}"

        # Extract key metrics (this is a simplified extraction)
        if grep -q "time:" "$log_file"; then
            echo "  - Completed successfully"
        else
            echo "  - Check log for details"
        fi
    fi
done

echo ""
echo -e "${GREEN}🎉 Benchmark suite completed!${NC}"
echo -e "${BLUE}📁 Reports saved in: $REPORTS_DIR${NC}"
echo -e "${BLUE}📊 HTML reports: $REPORTS_DIR/*_html${NC}"
echo ""

# Optional: Open HTML report in browser (uncomment if desired)
# if command -v xdg-open > /dev/null; then
#     echo "Opening HTML report in browser..."
#     xdg-open "$REPORTS_DIR/filesystem_bench_${TIMESTAMP}_html/report/index.html" 2>/dev/null || true
# fi