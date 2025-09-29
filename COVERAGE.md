# Coverage Analysis Tools

This document describes the comprehensive coverage analysis tools available for the Rust Jira MCP project.

## Quick Start

```bash
# Check current coverage status
make coverage-check

# View detailed analysis
make coverage-analyze

# Get test suggestions for a module
make coverage-suggest MODULE=main

# Open HTML coverage report
make coverage-dashboard
```

## Available Commands

### Basic Coverage Commands

| Command | Description |
|---------|-------------|
| `make coverage` | Run full coverage analysis and generate HTML report |
| `make coverage-check` | Quick coverage status check |
| `make coverage-analyze` | Detailed coverage analysis dashboard |
| `make coverage-dashboard` | Open HTML coverage report in browser |
| `make coverage-clean` | Clean coverage artifacts |

### Module-Specific Commands

| Command | Description |
|---------|-------------|
| `make coverage-modules` | Show module coverage breakdown |
| `make coverage-opportunities` | Show improvement opportunities |
| `make coverage-actions` | Show quick action commands |
| `make coverage-suggest MODULE=<name>` | Get test suggestions for a module |

### Examples

```bash
# Get suggestions for main.rs
make coverage-suggest MODULE=main

# Get suggestions for jira_client
make coverage-suggest MODULE=jira_client

# Get suggestions for mcp_tools
make coverage-suggest MODULE=mcp_tools

# Get suggestions for zephyr_tools
make coverage-suggest MODULE=zephyr_tools
```

## Current Coverage Status

**Overall Application Coverage: ~70-75%** (excluding test utilities)

### High Coverage Modules (✅ Good)
- **Error handling**: 100% coverage
- **Types**: 100% coverage  
- **Config secrets**: 97.3% coverage
- **MCP server**: 85.6% coverage
- **Config Jira**: 85.8% coverage
- **Utils response**: 87.8% coverage

### Medium Coverage Modules (⚠️ Needs improvement)
- **Config manager**: 70.9% coverage
- **Config validation**: 76.7% coverage
- **Logging**: 70-100% coverage

### Low Coverage Modules (❌ Critical gaps)
- **Main.rs**: 0% coverage (73 lines)
- **Jira client**: 52.3% coverage (1108 lines)
- **MCP tools**: 46.4% coverage (604 lines)
- **Zephyr tools**: 31.1% coverage (156 lines)

## Priority Improvement Areas

### 1. Main.rs (0% → 80%)
- **Impact**: High - Core application logic
- **Action**: Add integration tests for application startup
- **Lines**: 73 lines to cover

### 2. Jira Client (52% → 80%)
- **Impact**: High - Core functionality
- **Action**: Add more integration tests
- **Lines**: 455 lines to cover

### 3. MCP Tools (46% → 80%)
- **Impact**: High - Tool implementations
- **Action**: Add unit tests for tools
- **Lines**: 295 lines to cover

### 4. Zephyr Tools (31% → 80%)
- **Impact**: Medium - Zephyr features
- **Action**: Add Zephyr-specific tests
- **Lines**: 97 lines to cover

## Coverage Configuration

The project uses `cargo-llvm-cov` for coverage analysis with the following configuration:

- **Target Coverage**: 80%
- **Excluded Modules**: Test utilities (`src/test_utils/`, `src/test_usage.rs`)
- **Report Formats**: HTML, LCOV
- **Coverage Tool**: `cargo-llvm-cov`

## Understanding Coverage Reports

### HTML Report
- **Location**: `target/llvm-cov/html/index.html`
- **Features**: Interactive file browser, line-by-line coverage
- **Usage**: `make coverage-dashboard`

### LCOV Report
- **Location**: `target/llvm-cov/lcov.info`
- **Features**: Machine-readable format for CI/CD
- **Usage**: `make coverage-lcov`

## Best Practices

### 1. Regular Coverage Checks
```bash
# Run before committing
make coverage-check

# Full analysis weekly
make coverage-analyze
```

### 2. Focus on High-Impact Areas
- Prioritize modules with high line counts and low coverage
- Focus on core application logic first
- Test error handling paths

### 3. Use Test Suggestions
```bash
# Get specific suggestions for each module
make coverage-suggest MODULE=main
make coverage-suggest MODULE=jira_client
make coverage-suggest MODULE=mcp_tools
make coverage-suggest MODULE=zephyr_tools
```

### 4. Incremental Improvement
- Aim for 5-10% coverage improvement per iteration
- Focus on one module at a time
- Test new features immediately

## Troubleshooting

### No Coverage Data
```bash
# Generate fresh coverage data
make coverage
```

### Coverage Tool Issues
```bash
# Install/update coverage tools
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
```

### Module Not Found
```bash
# Check available modules
make coverage-modules
```

## Integration with CI/CD

The project includes comprehensive CI/CD integration with Codecov.io:

### GitHub Actions Workflows

#### 1. Coverage Workflow (`.github/workflows/coverage.yml`)
- **Triggers**: Push to main/develop, PRs, manual dispatch
- **Features**: 
  - Full coverage analysis
  - LCOV report generation
  - Codecov.io upload
  - Coverage summary in GitHub Actions
  - Artifact upload for HTML reports

#### 2. PR Coverage Workflow (`.github/workflows/pr-coverage.yml`)
- **Triggers**: Pull requests to main/develop
- **Features**:
  - Automated PR comments with coverage report
  - Module-by-module breakdown
  - Coverage status indicators
  - Quick action links

#### 3. CI Integration (`.github/workflows/ci.yml`)
- **Features**: Quick coverage check in main CI pipeline
- **Purpose**: Fast feedback on coverage status

### Codecov.io Integration

#### Configuration (`codecov.yml`)
```yaml
coverage:
  status:
    project:
      default:
        target: 80%
        threshold: 70%
    patch:
      default:
        target: 80%
        threshold: 70%

ignore:
  - "src/test_utils/**"
  - "tests/**"
  - "examples/**"
```

#### Features
- **Continuous Monitoring**: Real-time coverage tracking
- **PR Comments**: Automated coverage reports on pull requests
- **Coverage Badges**: Live coverage status in README
- **Historical Tracking**: Coverage trends over time
- **File-level Analysis**: Detailed coverage breakdown

### Setup Codecov Integration

```bash
# Run the setup script
./scripts/setup-codecov.sh

# Or manually:
# 1. Enable repository on codecov.io
# 2. Install coverage tools
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview

# 3. Test locally
make coverage-check
```

### Coverage Badges

The README includes live coverage badges:
- **CI Status**: GitHub Actions build status
- **Coverage**: Codecov.io coverage percentage
- **Coverage Status**: Manual coverage indicator
- **Rust Version**: Supported Rust version
- **License**: MIT license badge

## Contributing

When adding new features:

1. **Write tests first** - Aim for 80%+ coverage
2. **Check coverage** - Run `make coverage-check`
3. **Get suggestions** - Use `make coverage-suggest MODULE=<name>`
4. **Verify improvement** - Run `make coverage-analyze`

## Tools Reference

### Scripts
- `scripts/coverage.sh` - Basic coverage operations
- `scripts/coverage-simple.sh` - Simple analysis tools
- `scripts/coverage-analyzer.sh` - Advanced analysis (experimental)

### Configuration
- `coverage-config.toml` - Coverage configuration
- `.llvm-cov` - LLVM coverage settings

### Makefile Targets
- All coverage commands are available as `make` targets
- Use `make help` to see all available commands
