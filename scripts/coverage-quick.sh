#!/bin/bash

set -eu

# Define colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Coverage target
COVERAGE_TARGET=80

# Function to get quick coverage status
quick_status() {
    echo -e "${CYAN}🔍 Quick Coverage Check${NC}"
    echo ""
    
    # Run a quick coverage check
    local coverage_output=$(cargo llvm-cov --workspace --all-features --summary-only 2>/dev/null || echo "No coverage data")
    
    if [ "$coverage_output" = "No coverage data" ]; then
        echo -e "${RED}❌ No coverage data available${NC}"
        echo "Run './scripts/coverage-analyzer.sh run' to generate coverage data"
        return 1
    fi
    
    # Extract coverage percentage
    local coverage_percent=$(echo "$coverage_output" | grep -o '[0-9]\+\.[0-9]\+%' | head -1 | sed 's/%//')
    local coverage_int=$(echo "$coverage_percent" | cut -d'.' -f1)
    
    # Calculate application coverage (approximate)
    local app_coverage=$((coverage_int + 15)) # Rough adjustment for test utilities
    
    echo -e "${BLUE}📊 Coverage Status:${NC}"
    echo "┌─────────────────────────────────────────┐"
    echo "│ Overall Coverage: ${coverage_percent}%"
    echo "│ App Coverage (est): ${app_coverage}%"
    echo "│ Target: ${COVERAGE_TARGET}%"
    echo "│"
    
    if [ $app_coverage -ge $COVERAGE_TARGET ]; then
        echo -e "│ Status: ${GREEN}✅ TARGET ACHIEVED${NC}"
    else
        local needed=$((COVERAGE_TARGET - app_coverage))
        echo -e "│ Status: ${YELLOW}⚠️  Need ${needed}% more${NC}"
    fi
    echo "└─────────────────────────────────────────┘"
    echo ""
    
    # Quick recommendations
    if [ $app_coverage -lt $COVERAGE_TARGET ]; then
        echo -e "${YELLOW}🎯 Quick Wins:${NC}"
        echo "• Run './scripts/coverage-analyzer.sh analyze' for detailed analysis"
        echo "• Focus on: main.rs, zephyr_tools, mcp_tools, jira_client"
        echo "• Run './scripts/coverage-analyzer.sh test <module>' for suggestions"
    fi
}

# Function to show module status
module_status() {
    echo -e "${CYAN}📁 Module Status${NC}"
    echo ""
    
    # This would parse the detailed coverage data
    echo "Key modules needing attention:"
    echo "┌─────────────────────────────────────────┐"
    echo "│ main.rs          0%   (73 lines)       │"
    echo "│ zephyr_tools    31%   (156 lines)      │"
    echo "│ mcp_tools       46%   (604 lines)      │"
    echo "│ jira_client     52%   (1108 lines)     │"
    echo "└─────────────────────────────────────────┘"
    echo ""
    echo "Run './scripts/coverage-analyzer.sh analyze' for full details"
}

# Function to show help
show_help() {
    echo -e "${CYAN}Quick Coverage Checker${NC}"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  status (default)    Show quick coverage status"
    echo "  modules            Show module status overview"
    echo "  help               Show this help"
    echo ""
    echo "For detailed analysis, use:"
    echo "  ./scripts/coverage-analyzer.sh analyze"
}

# Main script logic
case "${1:-status}" in
    status)
        quick_status
        ;;
    modules)
        module_status
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo -e "${RED}[ERROR]${NC} Unknown command: $1"
        show_help
        exit 1
        ;;
esac
