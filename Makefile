.PHONY: help build test clean clippy fmt check install audit docs

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build the project
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

test: ## Run tests
	cargo test

test-release: ## Run tests in release mode
	cargo test --release

clean: ## Clean build artifacts
	cargo clean

clippy: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

clippy-pedantic: ## Run clippy with pedantic warnings
	cargo clippy --all-targets --all-features -- -W clippy::pedantic

fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt -- --check

check: fmt-check clippy test ## Run all checks (format, clippy, test)

install: ## Install the binary
	cargo install --path .

audit: ## Run security audit
	cargo audit

docs: ## Generate documentation
	cargo doc --no-deps --document-private-items --open

docs-build: ## Build documentation without opening
	cargo doc --no-deps --document-private-items

bench: ## Run benchmarks
	cargo bench

run: ## Run the binary
	cargo run

run-release: ## Run the binary in release mode
	cargo run --release

check-all: fmt-check clippy-pedantic test audit ## Run all checks including security audit

# Coverage targets
coverage: ## Run coverage analysis and generate HTML report
	@echo "Running coverage analysis..."
	@./scripts/coverage.sh run

coverage-open: ## Open coverage report in browser
	@./scripts/coverage.sh open

coverage-summary: ## Show coverage summary
	@./scripts/coverage.sh summary

coverage-lcov: ## Generate LCOV format report
	@./scripts/coverage.sh lcov

coverage-clean: ## Clean coverage artifacts
	@./scripts/coverage.sh clean

# New coverage analysis tools
coverage-check: ## Quick coverage status check
	@./scripts/coverage-simple.sh status

coverage-analyze: ## Detailed coverage analysis dashboard
	@./scripts/coverage-simple.sh modules && echo "" && ./scripts/coverage-simple.sh opportunities

coverage-modules: ## Show module coverage breakdown
	@./scripts/coverage-simple.sh modules

coverage-opportunities: ## Show improvement opportunities
	@./scripts/coverage-simple.sh opportunities

coverage-actions: ## Show quick action commands
	@./scripts/coverage-simple.sh actions

coverage-suggest: ## Generate test suggestions for a module (usage: make coverage-suggest MODULE=main)
	@./scripts/coverage-simple.sh suggest $(MODULE)

coverage-dashboard: ## Open coverage dashboard
	@./scripts/coverage.sh open

# Test targets
test-unit: ## Run unit tests only
	cargo test --lib

test-integration: ## Run integration tests only
	cargo test --test '*'

test-all: test-unit test-integration ## Run all tests

# Development targets
dev-setup: ## Set up development environment
	@echo "Setting up development environment..."
	@rustup component add llvm-tools-preview
	@cargo install cargo-llvm-cov --locked
	@echo "Development environment ready!"

test-coverage: coverage-summary ## Quick coverage check
