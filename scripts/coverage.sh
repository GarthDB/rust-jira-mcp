#!/bin/bash

# Rust Jira MCP - Coverage Script
# Provides easy commands for running coverage analysis with cargo-llvm-cov

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Set LLVM tools environment variables
export LLVM_COV=/Users/garthdb/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-cov
export LLVM_PROFDATA=/Users/garthdb/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata

# Function to print colored output
print_status() {
    echo -e "${BLUE}[COVERAGE]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  run          Run coverage analysis and generate HTML report"
    echo "  open         Open the HTML coverage report in browser"
    echo "  lcov         Generate LCOV format report"
    echo "  summary      Show coverage summary in terminal"
    echo "  clean        Clean coverage artifacts"
    echo "  help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 run       # Run full coverage analysis"
    echo "  $0 open      # Open coverage report"
    echo "  $0 summary   # Show coverage summary"
}

# Function to run coverage analysis
run_coverage() {
    print_status "Running coverage analysis with cargo-llvm-cov..."
    
    # Clean previous coverage data
    cargo clean --target-dir target/llvm-cov-target 2>/dev/null || true
    
    # Run coverage analysis
    cargo llvm-cov --html --output-dir target/llvm-cov/html
    
    print_success "Coverage analysis complete!"
    print_status "HTML report generated at: target/llvm-cov/html/index.html"
}

# Function to open coverage report
open_report() {
    if [ -f "target/llvm-cov/html/index.html" ]; then
        print_status "Opening coverage report..."
        open target/llvm-cov/html/index.html
        print_success "Coverage report opened in browser"
    else
        print_error "Coverage report not found. Run '$0 run' first."
        exit 1
    fi
}

# Function to generate LCOV report
generate_lcov() {
    print_status "Generating LCOV format report..."
    cargo llvm-cov --lcov --output-path lcov.info
    print_success "LCOV report generated: lcov.info"
}

# Function to show coverage summary
show_summary() {
    print_status "Generating coverage summary..."
    
    # Generate LCOV first
    cargo llvm-cov --lcov --output-path lcov.info > /dev/null 2>&1
    
    if [ -f "lcov.info" ]; then
        # Extract coverage percentage using grep and awk
        local total_lines=$(grep -c "^DA:" lcov.info 2>/dev/null || echo "0")
        local covered_lines=$(grep "^DA:" lcov.info | grep -c ",1$" 2>/dev/null || echo "0")
        
        if [ "$total_lines" -gt 0 ]; then
            local coverage_percent=$((covered_lines * 100 / total_lines))
            print_success "Coverage Summary:"
            echo "  Total lines: $total_lines"
            echo "  Covered lines: $covered_lines"
            echo "  Coverage: ${coverage_percent}%"
            
            if [ "$coverage_percent" -ge 80 ]; then
                print_success "✅ Coverage target (80%) achieved!"
            else
                local needed=$((80 - coverage_percent))
                print_warning "⚠️  Need ${needed}% more coverage to reach 80% target"
            fi
        else
            print_warning "No coverage data found"
        fi
    else
        print_error "Failed to generate LCOV report"
        exit 1
    fi
}

# Function to clean coverage artifacts
clean_coverage() {
    print_status "Cleaning coverage artifacts..."
    cargo clean --target-dir target/llvm-cov-target 2>/dev/null || true
    rm -f lcov.info 2>/dev/null || true
    rm -rf target/llvm-cov 2>/dev/null || true
    print_success "Coverage artifacts cleaned"
}

# Main script logic
case "${1:-help}" in
    run)
        run_coverage
        ;;
    open)
        open_report
        ;;
    lcov)
        generate_lcov
        ;;
    summary)
        show_summary
        ;;
    clean)
        clean_coverage
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        show_usage
        exit 1
        ;;
esac
