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
