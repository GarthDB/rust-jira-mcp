#!/bin/bash

# Script to check for #[allow] attributes in the codebase
# This ensures we maintain high code quality without suppressing warnings

echo "Checking for #[allow] attributes in the codebase..."

# Search for #[allow] attributes in Rust files
ALLOW_COUNT=$(find src -name "*.rs" -exec grep -n "#\[allow" {} + | wc -l)

if [ "$ALLOW_COUNT" -gt 0 ]; then
    echo "❌ Found $ALLOW_COUNT #[allow] attributes in the codebase:"
    find src -name "*.rs" -exec grep -n "#\[allow" {} + | sed 's/^/  /'
    echo ""
    echo "Please remove these #[allow] attributes and fix the underlying issues instead."
    exit 1
else
    echo "✅ No #[allow] attributes found in the codebase"
    exit 0
fi
