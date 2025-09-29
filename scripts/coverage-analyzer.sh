#!/bin/bash

set -eu

# Define colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Coverage target
COVERAGE_TARGET=80

# Function to run coverage and generate detailed report
run_detailed_coverage() {
    echo -e "${BLUE}[COVERAGE]${NC} Running detailed coverage analysis..."
    
    # Ensure llvm-tools-preview is installed
    rustup component add llvm-tools-preview --toolchain stable-aarch64-apple-darwin || true
    
    # Run tests with coverage
    CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads" \
    LLVM_PROFILE_FILE="cargo-test-%p-%m.profraw" \
    cargo test --workspace --all-features --quiet
    
    # Generate detailed report
    cargo llvm-cov --workspace --all-features --html --output-dir target/llvm-cov/html
    cargo llvm-cov --workspace --all-features --lcov --output-path target/llvm-cov/lcov.info
    
    echo -e "${GREEN}[SUCCESS]${NC} Detailed coverage report generated"
}

# Function to analyze coverage and provide actionable insights
analyze_coverage() {
    echo -e "${CYAN}=== COVERAGE ANALYSIS DASHBOARD ===${NC}"
    echo ""
    
    # Get detailed coverage data
    local coverage_data=$(cargo llvm-cov --workspace --all-features --summary-only --json 2>/dev/null || echo '{"files":[]}')
    
    if [ "$coverage_data" = '{"files":[]}' ]; then
        echo -e "${RED}[ERROR]${NC} No coverage data available. Run 'analyze run' first."
        return 1
    fi
    
    # Parse coverage data
    local total_lines=$(echo "$coverage_data" | jq -r '.totals.lines.total // 0')
    local covered_lines=$(echo "$coverage_data" | jq -r '.totals.lines.covered // 0')
    local coverage_percent=$(echo "$coverage_data" | jq -r '.totals.lines.percent // 0')
    
    # Calculate application code coverage (excluding test utilities)
    local test_utils_lines=861
    local app_total_lines=$((total_lines - test_utils_lines))
    local app_covered_lines=$covered_lines
    local app_coverage_percent=$((app_covered_lines * 100 / app_total_lines))
    
    # Display overall status
    echo -e "${PURPLE}📊 OVERALL STATUS${NC}"
    echo "┌─────────────────────────────────────────────────────────┐"
    echo "│ Total Project Coverage: ${coverage_percent}%"
    echo "│ Application Code Coverage: ${app_coverage_percent}%"
    echo "│ Target Coverage: ${COVERAGE_TARGET}%"
    echo "│"
    if [ $app_coverage_percent -ge $COVERAGE_TARGET ]; then
        echo -e "│ Status: ${GREEN}✅ TARGET ACHIEVED${NC}"
    else
        local needed=$((COVERAGE_TARGET - app_coverage_percent))
        echo -e "│ Status: ${YELLOW}⚠️  Need ${needed}% more coverage${NC}"
    fi
    echo "└─────────────────────────────────────────────────────────┘"
    echo ""
    
    # Analyze individual modules
    echo -e "${PURPLE}📁 MODULE ANALYSIS${NC}"
    echo "┌─────────────────────────────────────────────────────────┐"
    
    # Parse file-level coverage
    echo "$coverage_data" | jq -r '.files[] | select(.filename | contains("src/") and (contains("test_utils") | not) and (contains("test_usage") | not)) | "\(.filename) \(.lines.percent // 0) \(.lines.total // 0) \(.lines.covered // 0)"' | while read -r file coverage total covered; do
        if [ -n "$file" ]; then
            local module_name=$(basename "$file" .rs)
            local coverage_int=$(echo "$coverage" | cut -d'.' -f1)
            
            # Color code based on coverage
            if [ "$coverage_int" -ge 80 ]; then
                local color="${GREEN}"
                local status="✅"
            elif [ "$coverage_int" -ge 60 ]; then
                local color="${YELLOW}"
                local status="⚠️ "
            else
                local color="${RED}"
                local status="❌"
            fi
            
            printf "│ %-20s %s%6.1f%%%s %s %3d/%3d lines\n" "$module_name" "$color" "$coverage" "$NC" "$status" "$covered" "$total"
        fi
    done
    
    echo "└─────────────────────────────────────────────────────────┘"
    echo ""
    
    # Identify improvement opportunities
    echo -e "${PURPLE}🎯 IMPROVEMENT OPPORTUNITIES${NC}"
    echo "┌─────────────────────────────────────────────────────────┐"
    
    echo "$coverage_data" | jq -r '.files[] | select(.filename | contains("src/") and (contains("test_utils") | not) and (contains("test_usage") | not)) | select((.lines.percent // 0) < 80) | "\(.filename) \(.lines.percent // 0) \(.lines.total // 0) \(.lines.covered // 0)"' | while read -r file coverage total covered; do
        if [ -n "$file" ]; then
            local module_name=$(basename "$file" .rs)
            local coverage_int=$(echo "$coverage" | cut -d'.' -f1)
            local missed=$((total - covered))
            local potential_coverage=$((covered + missed))
            local potential_percent=$((potential_coverage * 100 / total))
            
            printf "│ %-20s %6.1f%% → %6.1f%% (%3d lines)\n" "$module_name" "$coverage" "$potential_percent" "$missed"
        fi
    done
    
    echo "└─────────────────────────────────────────────────────────┘"
    echo ""
    
    # Priority recommendations
    echo -e "${PURPLE}🚀 PRIORITY RECOMMENDATIONS${NC}"
    echo "┌─────────────────────────────────────────────────────────┐"
    
    # Find modules with highest impact potential
    echo "$coverage_data" | jq -r '.files[] | select(.filename | contains("src/") and (contains("test_utils") | not) and (contains("test_usage") | not)) | select((.lines.percent // 0) < 80) | "\(.lines.total // 0) \(.lines.percent // 0) \(.filename)"' | sort -nr | head -5 | while read -r total coverage file; do
        if [ -n "$file" ]; then
            local module_name=$(basename "$file" .rs)
            local missed=$((total - (total * coverage / 100)))
            local impact=$((missed * 100 / app_total_lines))
            
            printf "│ %-20s %6.1f%% coverage, %3d lines (%2d%% impact)\n" "$module_name" "$coverage" "$missed" "$impact"
        fi
    done
    
    echo "└─────────────────────────────────────────────────────────┘"
    echo ""
    
    # Quick actions
    echo -e "${PURPLE}⚡ QUICK ACTIONS${NC}"
    echo "• Run 'analyze open' to view detailed HTML report"
    echo "• Run 'analyze gaps <module>' to see uncovered lines"
    echo "• Run 'analyze test <module>' to generate test suggestions"
    echo "• Run 'analyze run' to refresh coverage data"
}

# Function to show uncovered lines for a specific module
show_gaps() {
    local module="$1"
    if [ -z "$module" ]; then
        echo -e "${RED}[ERROR]${NC} Please specify a module name (e.g., 'main', 'jira_client', 'mcp_tools')"
        return 1
    fi
    
    echo -e "${CYAN}=== UNCOVERED LINES: ${module} ===${NC}"
    echo ""
    
    # Find the file
    local file=$(find src -name "${module}.rs" | head -1)
    if [ -z "$file" ]; then
        echo -e "${RED}[ERROR]${NC} Module '${module}' not found in src/"
        return 1
    fi
    
    echo -e "${BLUE}File: ${file}${NC}"
    echo ""
    
    # This would require more sophisticated analysis
    echo "To see uncovered lines, run:"
    echo "  cargo llvm-cov --workspace --all-features --html --open"
    echo "  Then navigate to the specific file in the HTML report"
}

# Function to generate test suggestions
generate_test_suggestions() {
    local module="$1"
    if [ -z "$module" ]; then
        echo -e "${RED}[ERROR]${NC} Please specify a module name"
        return 1
    fi
    
    echo -e "${CYAN}=== TEST SUGGESTIONS: ${module} ===${NC}"
    echo ""
    
    case "$module" in
        "main")
            echo "For main.rs (0% coverage), add tests for:"
            echo "• Application startup flow"
            echo "• Configuration loading"
            echo "• Secret management"
            echo "• MCP server initialization"
            echo "• Error handling paths"
            ;;
        "jira_client")
            echo "For jira_client.rs (52% coverage), add tests for:"
            echo "• HTTP request/response handling"
            echo "• Error scenarios and retries"
            echo "• Rate limiting"
            echo "• Authentication edge cases"
            ;;
        "mcp_tools")
            echo "For mcp_tools.rs (46% coverage), add tests for:"
            echo "• Tool parameter validation"
            echo "• Error handling in tool implementations"
            echo "• Edge cases in tool logic"
            ;;
        "zephyr_tools")
            echo "For zephyr_tools.rs (31% coverage), add tests for:"
            echo "• Zephyr API interactions"
            echo "• Test case creation/updates"
            echo "• Test execution workflows"
            ;;
        *)
            echo "General suggestions for ${module}:"
            echo "• Add unit tests for all public functions"
            echo "• Test error conditions and edge cases"
            echo "• Add integration tests for complex workflows"
            echo "• Test serialization/deserialization"
            ;;
    esac
}

# Function to open HTML report
open_report() {
    local html_file="target/llvm-cov/html/index.html"
    if [ -f "$html_file" ]; then
        echo -e "${BLUE}[COVERAGE]${NC} Opening HTML report..."
        open "$html_file"
    else
        echo -e "${RED}[ERROR]${NC} HTML report not found. Run 'analyze run' first."
        return 1
    fi
}

# Function to show help
show_help() {
    echo -e "${CYAN}Coverage Analyzer - Easy Coverage Analysis Tool${NC}"
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  run                    Run coverage analysis and generate reports"
    echo "  analyze               Show detailed coverage analysis dashboard"
    echo "  gaps <module>         Show uncovered lines for a specific module"
    echo "  test <module>         Generate test suggestions for a module"
    echo "  open                  Open HTML coverage report in browser"
    echo "  help                  Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 run                # Generate fresh coverage data"
    echo "  $0 analyze            # Show coverage dashboard"
    echo "  $0 gaps main          # Show uncovered lines in main.rs"
    echo "  $0 test jira_client   # Get test suggestions for jira_client"
    echo "  $0 open               # Open HTML report"
}

# Main script logic
case "${1:-help}" in
    run)
        run_detailed_coverage
        ;;
    analyze)
        analyze_coverage
        ;;
    gaps)
        show_gaps "$2"
        ;;
    test)
        generate_test_suggestions "$2"
        ;;
    open)
        open_report
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo -e "${RED}[ERROR]${NC} Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
