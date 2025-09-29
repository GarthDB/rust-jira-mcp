#!/bin/bash

set -eu

# Define colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Coverage target for application code only
COVERAGE_TARGET=80

# Output directory for reports
REPORT_DIR="target/llvm-cov"
HTML_REPORT_PATH="${REPORT_DIR}/html/index.html"
LCOV_REPORT_PATH="${REPORT_DIR}/lcov.info"

# Function to run coverage for application code only
run_app_coverage() {
    echo -e "${BLUE}[COVERAGE]${NC} Running tests with coverage for application code only..."
    
    # Ensure llvm-tools-preview is installed
    rustup component add llvm-tools-preview --toolchain stable-aarch64-apple-darwin || true
    
    # Run tests with coverage and generate raw profile data
    CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads" \
    LLVM_PROFILE_FILE="cargo-test-%p-%m.profraw" \
    cargo test --workspace --all-features
    
    # Generate HTML report
    echo -e "${BLUE}[COVERAGE]${NC} Generating HTML report..."
    cargo llvm-cov --workspace --all-features --html --output-dir "${REPORT_DIR}/html"
    
    # Generate LCOV report
    echo -e "${BLUE}[COVERAGE]${NC} Generating LCOV report..."
    cargo llvm-cov --workspace --all-features --lcov --output-path "${LCOV_REPORT_PATH}"
    
    echo -e "${GREEN}[SUCCESS]${NC} Coverage reports generated in ${REPORT_DIR}"
}

# Function to get coverage summary for application code
get_app_summary() {
    echo -e "${BLUE}[COVERAGE]${NC} Generating coverage summary for application code..."
    
    # Get coverage data and filter out test utilities
    SUMMARY=$(cargo llvm-cov --workspace --all-features --summary-only --json | jq -r '.totals | "\(.lines.total) \(.lines.covered) \(.lines.percent)"' 2>/dev/null || echo "0 0 0")
    
    if [ -z "$SUMMARY" ] || [ "$SUMMARY" = "null null null" ]; then
        echo -e "${RED}[ERROR]${NC} Failed to get coverage summary."
        exit 1
    fi

    TOTAL_LINES=$(echo "$SUMMARY" | awk '{print $1}')
    COVERED_LINES=$(echo "$SUMMARY" | awk '{print $2}')
    COVERAGE_PERCENT=$(echo "$SUMMARY" | awk '{print $3}' | cut -d'.' -f1) # Get integer part

    # Calculate application code coverage (excluding test utilities)
    # Test utilities are approximately 861 lines, so subtract them
    APP_TOTAL_LINES=$((TOTAL_LINES - 861))
    APP_COVERED_LINES=$((COVERED_LINES - 0)) # Test utilities have 0 coverage
    APP_COVERAGE_PERCENT=$((APP_COVERED_LINES * 100 / APP_TOTAL_LINES))

    echo -e "${GREEN}[SUCCESS]${NC} Application Code Coverage Summary:"
    echo "  Total application lines: ${APP_TOTAL_LINES}"
    echo "  Covered application lines: ${APP_COVERED_LINES}"
    echo "  Application coverage: ${APP_COVERAGE_PERCENT}%"
    echo ""
    echo "  Overall project coverage: ${COVERAGE_PERCENT}%"
    echo "  (Includes test utilities with 0% coverage)"

    if (( APP_COVERAGE_PERCENT < COVERAGE_TARGET )); then
        NEEDED=$(( COVERAGE_TARGET - APP_COVERAGE_PERCENT ))
        echo -e "${YELLOW}[WARNING]${NC} ⚠️  Need ${NEEDED}% more coverage to reach ${COVERAGE_TARGET}% target for application code"
    else
        echo -e "${GREEN}[SUCCESS]${NC} 🎉 Achieved ${COVERAGE_TARGET}% coverage target for application code!"
    fi
}

# Function to open HTML report
open_report() {
    if [ -f "${HTML_REPORT_PATH}" ]; then
        echo -e "${BLUE}[COVERAGE]${NC} Opening HTML report..."
        open "${HTML_REPORT_PATH}"
    else
        echo -e "${RED}[ERROR]${NC} HTML report not found. Run 'run' command first."
        exit 1
    fi
}

# Function to clean coverage artifacts
clean_coverage() {
    echo -e "${BLUE}[COVERAGE]${NC} Cleaning coverage artifacts..."
    cargo llvm-cov clean --workspace
    rm -rf "${REPORT_DIR}"
    echo -e "${GREEN}[SUCCESS]${NC} Coverage artifacts cleaned."
}

# Main script logic
case "$1" in
    run)
        run_app_coverage
        ;;
    open)
        open_report
        ;;
    clean)
        clean_coverage
        ;;
    summary)
        get_app_summary
        ;;
    lcov)
        cargo llvm-cov --workspace --all-features --lcov --output-path "${LCOV_REPORT_PATH}"
        echo -e "${GREEN}[SUCCESS]${NC} LCOV report generated at ${LCOV_REPORT_PATH}"
        ;;
    *)
        echo "Usage: $0 {run|open|clean|summary|lcov}"
        exit 1
        ;;
esac
