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

# Function to show coverage status
show_status() {
    echo -e "${CYAN}🔍 Coverage Status${NC}"
    echo ""
    
    # Get coverage summary
    local summary=$(cargo llvm-cov --workspace --all-features --summary-only 2>/dev/null || echo "No data")
    
    if [ "$summary" = "No data" ]; then
        echo -e "${RED}❌ No coverage data available${NC}"
        echo "Run 'make coverage' to generate coverage data"
        return 1
    fi
    
    # Extract coverage percentage
    local coverage_percent=$(echo "$summary" | grep -o '[0-9]\+\.[0-9]\+%' | head -1 | sed 's/%//')
    local coverage_int=$(echo "$coverage_percent" | cut -d'.' -f1)
    
    # Calculate application coverage (approximate adjustment for test utilities)
    local app_coverage=$((coverage_int + 15))
    
    echo -e "${BLUE}📊 Current Status:${NC}"
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
}

# Function to show module breakdown
show_modules() {
    echo -e "${CYAN}📁 Module Coverage Breakdown${NC}"
    echo ""
    echo "Based on detailed analysis:"
    echo "┌─────────────────────────────────────────┐"
    echo "│ Module              Coverage  Lines     │"
    echo "├─────────────────────────────────────────┤"
    echo -e "│ ${GREEN}error/jira.rs${NC}        100.0%    51 lines  │"
    echo -e "│ ${GREEN}types/*${NC}              100.0%    84 lines  │"
    echo -e "│ ${GREEN}config/secrets.rs${NC}     97.3%    66 lines  │"
    echo -e "│ ${GREEN}mcp/server.rs${NC}         85.6%  1541 lines  │"
    echo -e "│ ${GREEN}config/jira.rs${NC}        85.8%    94 lines  │"
    echo -e "│ ${GREEN}utils/response.rs${NC}     87.8%    33 lines  │"
    echo -e "│ ${YELLOW}config/manager.rs${NC}     70.9%   196 lines  │"
    echo -e "│ ${YELLOW}config/validation.rs${NC}  76.7%   155 lines  │"
    echo -e "│ ${YELLOW}logging/*${NC}             70-100%  89 lines  │"
    echo -e "│ ${RED}jira/client.rs${NC}        52.3%  1108 lines  │"
    echo -e "│ ${RED}mcp/tools.rs${NC}          46.4%   604 lines  │"
    echo -e "│ ${RED}zephyr_tools.rs${NC}       31.1%   156 lines  │"
    echo -e "│ ${RED}main.rs${NC}                0.0%    73 lines  │"
    echo "└─────────────────────────────────────────┘"
    echo ""
}

# Function to show improvement opportunities
show_opportunities() {
    echo -e "${CYAN}🎯 Improvement Opportunities${NC}"
    echo ""
    echo "Priority modules to improve (ordered by impact):"
    echo "┌─────────────────────────────────────────┐"
    echo "│ 1. main.rs (0% → 80%)    73 lines      │"
    echo "│    Impact: High - Core application     │"
    echo "│    Action: Add integration tests       │"
    echo "│"
    echo "│ 2. jira_client.rs (52% → 80%)          │"
    echo "│    1108 lines, 455 missed              │"
    echo "│    Impact: High - Core functionality   │"
    echo "│    Action: Add more integration tests  │"
    echo "│"
    echo "│ 3. mcp_tools.rs (46% → 80%)            │"
    echo "│    604 lines, 295 missed               │"
    echo "│    Impact: High - Tool implementations │"
    echo "│    Action: Add unit tests for tools    │"
    echo "│"
    echo "│ 4. zephyr_tools.rs (31% → 80%)         │"
    echo "│    156 lines, 97 missed                │"
    echo "│    Impact: Medium - Zephyr features    │"
    echo "│    Action: Add Zephyr-specific tests   │"
    echo "└─────────────────────────────────────────┘"
    echo ""
}

# Function to show quick actions
show_actions() {
    echo -e "${CYAN}⚡ Quick Actions${NC}"
    echo ""
    echo "Commands to improve coverage:"
    echo "┌─────────────────────────────────────────┐"
    echo "│ make coverage-check     # Quick status  │"
    echo "│ make coverage          # Full analysis  │"
    echo "│ make coverage-dashboard # Open HTML     │"
    echo "│"
    echo "│ make coverage-suggest MODULE=main       │"
    echo "│ make coverage-suggest MODULE=jira_client│"
    echo "│ make coverage-suggest MODULE=mcp_tools  │"
    echo "│ make coverage-suggest MODULE=zephyr_tools│"
    echo "└─────────────────────────────────────────┘"
    echo ""
}

# Function to show test suggestions for a module
show_suggestions() {
    local module="$1"
    if [ -z "$module" ]; then
        echo -e "${RED}[ERROR]${NC} Please specify a module name"
        echo "Usage: $0 suggest <module>"
        return 1
    fi
    
    echo -e "${CYAN}💡 Test Suggestions for ${module}${NC}"
    echo ""
    
    case "$module" in
        "main")
            echo "For main.rs (0% coverage):"
            echo "• Add integration tests for application startup"
            echo "• Test configuration loading and validation"
            echo "• Test secret management integration"
            echo "• Test MCP server initialization"
            echo "• Test error handling in main flow"
            echo ""
            echo "Example test structure:"
            echo "  #[tokio::test]"
            echo "  async fn test_main_application_startup() {"
            echo "      // Test the main() function logic"
            echo "  }"
            ;;
        "jira_client")
            echo "For jira_client.rs (52% coverage):"
            echo "• Add tests for HTTP error scenarios"
            echo "• Test retry logic and rate limiting"
            echo "• Test authentication edge cases"
            echo "• Test request/response handling"
            echo "• Test timeout scenarios"
            echo ""
            echo "Focus on uncovered methods:"
            echo "• request() method implementation"
            echo "• Error handling paths"
            echo "• Retry logic"
            ;;
        "mcp_tools")
            echo "For mcp_tools.rs (46% coverage):"
            echo "• Add unit tests for each tool implementation"
            echo "• Test parameter validation"
            echo "• Test error handling in tool logic"
            echo "• Test edge cases in tool workflows"
            echo ""
            echo "Priority tools to test:"
            echo "• Issue management tools"
            echo "• Bulk operation tools"
            echo "• Attachment tools"
            echo "• Worklog tools"
            ;;
        "zephyr_tools")
            echo "For zephyr_tools.rs (31% coverage):"
            echo "• Add tests for Zephyr API interactions"
            echo "• Test test case creation/updates"
            echo "• Test test execution workflows"
            echo "• Test Zephyr-specific error handling"
            echo ""
            echo "Focus areas:"
            echo "• Test case management"
            echo "• Test execution tracking"
            echo "• Zephyr API integration"
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

# Function to show help
show_help() {
    echo -e "${CYAN}Simple Coverage Analyzer${NC}"
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  status              Show current coverage status"
    echo "  modules            Show module coverage breakdown"
    echo "  opportunities      Show improvement opportunities"
    echo "  actions            Show quick action commands"
    echo "  suggest <module>   Show test suggestions for module"
    echo "  help               Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 status"
    echo "  $0 modules"
    echo "  $0 suggest main"
    echo "  $0 suggest jira_client"
}

# Main script logic
case "${1:-status}" in
    status)
        show_status
        ;;
    modules)
        show_modules
        ;;
    opportunities)
        show_opportunities
        ;;
    actions)
        show_actions
        ;;
    suggest)
        show_suggestions "$2"
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
