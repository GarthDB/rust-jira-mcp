#!/bin/bash

set -eu

# Define colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Setting up Codecov.io integration...${NC}"
echo ""

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo -e "${RED}❌ Not in a git repository${NC}"
    exit 1
fi

# Check if we're on GitHub
if ! git remote get-url origin | grep -q "github.com"; then
    echo -e "${YELLOW}⚠️  This doesn't appear to be a GitHub repository${NC}"
    echo "Codecov.io integration works best with GitHub repositories"
    echo ""
fi

echo -e "${BLUE}📋 Setup Checklist:${NC}"
echo ""

# 1. Check if codecov.yml exists
if [ -f "codecov.yml" ]; then
    echo -e "✅ ${GREEN}codecov.yml configuration file exists${NC}"
else
    echo -e "❌ ${RED}codecov.yml configuration file missing${NC}"
    echo "   Run: cp codecov.yml.example codecov.yml"
fi

# 2. Check if GitHub Actions workflows exist
if [ -f ".github/workflows/coverage.yml" ]; then
    echo -e "✅ ${GREEN}Coverage workflow exists${NC}"
else
    echo -e "❌ ${RED}Coverage workflow missing${NC}"
    echo "   Run: cp .github/workflows/coverage.yml.example .github/workflows/coverage.yml"
fi

# 3. Check if PR coverage workflow exists
if [ -f ".github/workflows/pr-coverage.yml" ]; then
    echo -e "✅ ${GREEN}PR coverage workflow exists${NC}"
else
    echo -e "❌ ${RED}PR coverage workflow missing${NC}"
    echo "   Run: cp .github/workflows/pr-coverage.yml.example .github/workflows/pr-coverage.yml"
fi

# 4. Check if coverage tools are installed
if command -v cargo-llvm-cov &> /dev/null; then
    echo -e "✅ ${GREEN}cargo-llvm-cov is installed${NC}"
else
    echo -e "❌ ${RED}cargo-llvm-cov is not installed${NC}"
    echo "   Run: cargo install cargo-llvm-cov --locked"
fi

# 5. Check if llvm-tools is installed
if rustup component list --installed | grep -q "llvm-tools"; then
    echo -e "✅ ${GREEN}llvm-tools is installed${NC}"
else
    echo -e "❌ ${RED}llvm-tools is not installed${NC}"
    echo "   Run: rustup component add llvm-tools-preview"
fi

echo ""
echo -e "${BLUE}🚀 Next Steps:${NC}"
echo ""
echo "1. **Enable Codecov.io for your repository:**"
echo "   - Go to https://codecov.io/gh/$(git remote get-url origin | sed 's/.*github.com[:/]\([^.]*\).*/\1/')"
echo "   - Sign in with GitHub"
echo "   - Enable the repository"
echo ""
echo "2. **Test the integration locally:**"
echo "   make coverage"
echo "   make coverage-check"
echo ""
echo "3. **Push to GitHub to trigger workflows:**"
echo "   git add ."
echo "   git commit -m 'Add Codecov.io integration'"
echo "   git push"
echo ""
echo "4. **Check the results:**"
echo "   - View coverage reports at https://codecov.io/gh/$(git remote get-url origin | sed 's/.*github.com[:/]\([^.]*\).*/\1/')"
echo "   - Check GitHub Actions tab for workflow status"
echo ""

# Test coverage generation
echo -e "${BLUE}🧪 Testing coverage generation...${NC}"
if make coverage-check > /dev/null 2>&1; then
    echo -e "✅ ${GREEN}Coverage generation works locally${NC}"
else
    echo -e "⚠️  ${YELLOW}Coverage generation needs setup${NC}"
    echo "   Run: make coverage to generate initial coverage data"
fi

echo ""
echo -e "${GREEN}🎉 Codecov.io integration setup complete!${NC}"
echo ""
echo "For more information, see:"
echo "- COVERAGE.md - Detailed coverage documentation"
echo "- .github/workflows/coverage.yml - Coverage workflow"
echo "- codecov.yml - Codecov configuration"
