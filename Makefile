# Makefile for rust-jira-mcp

.PHONY: help build test clean install xtask

# Default target
help:
	@echo "Available targets:"
	@echo "  build          - Build the MCP server"
	@echo "  build-release  - Build the MCP server in release mode"
	@echo "  test           - Run tests"
	@echo "  test-readonly  - Run read-only tests"
	@echo "  test-issues    - Run issue tests"
	@echo "  test-write     - Run write tests (safe project only)"
	@echo "  test-suite     - Run comprehensive test suite"
	@echo "  test-cleanup   - Clean up test data"
	@echo "  collect-fixtures - Collect test fixtures from live API"
	@echo "  generate-fixtures - Generate synthetic test fixtures"
	@echo "  clean          - Clean build artifacts"
	@echo "  install        - Install dependencies"
	@echo "  xtask-help     - Show xtask help"

# Build targets
build:
	cargo build

build-release:
	cargo build --release

# Test targets
test:
	cargo test

test-readonly:
	cargo run --package xtask -- test --suite read-only

test-issues:
	cargo run --package xtask -- test --suite issues

test-write:
	cargo run --package xtask -- test --suite write --safe

test-suite:
	cargo run --package xtask -- test-suite --project TEST-MCP

test-cleanup:
	cargo run --package xtask -- cleanup --project TEST-MCP

# Fixture targets
collect-fixtures:
	cargo run --package xtask -- collect-fixtures --project DNA

generate-fixtures:
	cargo run --package xtask -- generate-fixtures --project TEST-MCP --count 5

# Utility targets
clean:
	cargo clean

install:
	cargo build

xtask-help:
	cargo run --bin xtask -- --help

# Development targets
dev-setup: install
	@echo "Development setup complete"

# CI targets
ci-test: build test
	@echo "CI tests completed"

ci-collect-fixtures: build-release collect-fixtures
	@echo "Fixture collection completed"