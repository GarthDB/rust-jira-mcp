# Documentation Index

Welcome to the comprehensive documentation for the Rust Jira MCP Server. This documentation covers everything you need to know to get started, configure, and use the server effectively.

## 📚 Documentation Overview

### Getting Started
- **[Getting Started Guide](getting-started.md)** - Complete setup and usage guide
- **[Quick Start Examples](getting-started.md#quick-start)** - Get up and running in minutes
- **[Installation Guide](getting-started.md#installation)** - Installation options and requirements

### Configuration
- **[Configuration Guide](../CONFIGURATION.md)** - Comprehensive configuration management
- **[Configuration Examples](configuration-examples.md)** - Detailed configuration examples
- **[Environment Variables](configuration-examples.md#environment-variables)** - Complete environment variable reference
- **[Secret Management](configuration-examples.md#secret-management-examples)** - Secure secret handling

### API Reference
- **[API Documentation](https://docs.rs/rust-jira-mcp)** - Full API reference (generated with rustdoc)
- **[Tool Examples](tool-examples.md)** - Detailed examples for all MCP tools
- **[MCP Protocol](tool-examples.md#mcp-protocol)** - Model Context Protocol implementation

### Usage Guides
- **[Tool Usage Examples](tool-examples.md)** - Comprehensive examples for all tools
- **[Bulk Operations](tool-examples.md#bulk-operations-tools)** - Efficient bulk operations
- **[Zephyr Integration](tool-examples.md#zephyr-test-management-tools)** - Test management features
- **[Advanced Patterns](tool-examples.md#advanced-usage-patterns)** - Advanced usage patterns

### Troubleshooting & Performance
- **[Troubleshooting Guide](troubleshooting.md)** - Common issues and solutions
- **[Performance Guide](performance.md)** - Optimization and benchmarking
- **[Debugging Tools](troubleshooting.md#debugging-tools)** - Debugging and diagnostics
- **[Performance Tuning](performance.md#performance-tuning-checklist)** - Performance optimization checklist

## 🚀 Quick Navigation

### By Use Case

**I want to...**
- **Get started quickly** → [Getting Started Guide](getting-started.md)
- **Configure the server** → [Configuration Guide](../CONFIGURATION.md)
- **Use specific tools** → [Tool Examples](tool-examples.md)
- **Troubleshoot issues** → [Troubleshooting Guide](troubleshooting.md)
- **Optimize performance** → [Performance Guide](performance.md)
- **Understand the API** → [API Documentation](https://docs.rs/rust-jira-mcp)

### By Experience Level

**Beginner**
- [Getting Started Guide](getting-started.md)
- [Configuration Examples](configuration-examples.md)
- [Basic Tool Usage](tool-examples.md#basic-issue-operations)

**Intermediate**
- [Advanced Configuration](configuration-examples.md#advanced-configuration-patterns)
- [Bulk Operations](tool-examples.md#bulk-operations-tools)
- [Performance Tuning](performance.md#configuration-optimization)

**Advanced**
- [Custom Validation](configuration-examples.md#validation-examples)
- [Hot-Reloading](configuration-examples.md#hot-reloading-examples)
- [Benchmarking](performance.md#benchmarking)

## 📖 Documentation Structure

```
docs/
├── README.md                    # This file - documentation index
├── getting-started.md          # Complete setup and usage guide
├── tool-examples.md            # Comprehensive tool usage examples
├── troubleshooting.md          # Common issues and solutions
├── performance.md              # Optimization and benchmarking
└── configuration-examples.md   # Detailed configuration examples
```

## 🔧 Configuration Quick Reference

### Environment Variables
```bash
# Required
JIRA_EMAIL=your.email@company.com
JIRA_PERSONAL_ACCESS_TOKEN=your_personal_access_token_here

# Optional
JIRA_API_BASE_URL=https://your-company.atlassian.net/rest/api/2
JIRA_DEFAULT_PROJECT=PROJ
JIRA_MAX_RESULTS=50
JIRA_TIMEOUT_SECONDS=30
```

### Configuration Files
```toml
# config/default.toml
[default]
api_base_url = "https://your-company.atlassian.net/rest/api/2"
max_results = 50
timeout_seconds = 30
strict_ssl = true
```

## 🛠️ Tool Quick Reference

### Core Issue Management
- `test_jira_auth` - Test authentication
- `search_jira_issues` - Search issues with JQL
- `create_jira_issue` - Create new issues
- `update_jira_issue` - Update existing issues
- `get_jira_issue` - Get issue details
- `get_jira_comments` - Get issue comments
- `add_jira_comment` - Add comments to issues
- `get_jira_transitions` - Get available transitions
- `transition_jira_issue` - Transition issues

### Project Configuration
- `get_project_config` - Get project configuration
- `get_project_issue_types` - Get issue types
- `get_issue_type_metadata` - Get issue type details
- `get_project_components` - Get project components
- `get_priorities_and_statuses` - Get priorities and statuses
- `get_custom_fields` - Get custom field definitions
- `get_project_metadata` - Get comprehensive metadata

### Bulk Operations
- `bulk_create_issues` - Create multiple issues
- `bulk_update_issues` - Update multiple issues
- `bulk_transition_issues` - Transition multiple issues
- `bulk_add_comments` - Add comments to multiple issues

### Zephyr Test Management
- `get_test_cycles` - Get test cycles
- `create_test_cycle` - Create test cycles
- `get_test_executions` - Get test executions
- `update_test_execution` - Update test executions

## 📊 Performance Quick Reference

### Typical Performance
- **Response Time**: 50-200ms for typical operations
- **Throughput**: 100-500 requests/minute
- **Memory Usage**: 10-50MB typical, 100MB+ for large operations
- **Concurrent Connections**: Up to 100 simultaneous connections

### Optimization Tips
- Use bulk operations instead of individual calls
- Implement response caching for frequently accessed data
- Use specific fields to reduce response size
- Enable connection pooling
- Use pagination for large result sets

## 🐛 Troubleshooting Quick Reference

### Common Issues
1. **Authentication failed** → Check credentials and URL format
2. **Configuration not found** → Verify file paths and permissions
3. **API rate limiting** → Implement backoff strategies
4. **Memory usage high** → Limit result sets and use pagination
5. **Connection timeout** → Check network connectivity and increase timeout

### Debug Commands
```bash
# Test authentication
cargo run --release -- --test-auth

# Validate configuration
cargo run --release -- --validate-config

# Enable debug logging
RUST_LOG=debug cargo run --release
```

## 📚 Additional Resources

### External Documentation
- [Jira REST API Documentation](https://developer.atlassian.com/cloud/jira/platform/rest/v2/)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [Rust Documentation](https://doc.rust-lang.org/)

### Community
- [GitHub Repository](https://github.com/GarthDB/rust-jira-mcp)
- [Issue Tracker](https://github.com/GarthDB/rust-jira-mcp/issues)
- [Discussions](https://github.com/GarthDB/rust-jira-mcp/discussions)

### Examples
- [Project Examples](../examples/) - Code examples in the repository
- [Tool Examples](tool-examples.md) - Comprehensive tool usage examples
- [Configuration Examples](configuration-examples.md) - Configuration examples

## 🤝 Contributing to Documentation

We welcome contributions to improve the documentation! Here's how you can help:

### Reporting Issues
- Found a typo or error? [Open an issue](https://github.com/GarthDB/rust-jira-mcp/issues)
- Missing information? [Request documentation](https://github.com/GarthDB/rust-jira-mcp/discussions)

### Contributing Changes
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

### Documentation Standards
- Use clear, concise language
- Include code examples where helpful
- Follow the existing documentation structure
- Test all code examples
- Update the documentation index when adding new content

## 📝 Documentation Changelog

### Version 0.1.0
- Initial comprehensive documentation
- Getting started guide
- Tool usage examples
- Configuration guide
- Troubleshooting guide
- Performance guide
- API documentation with rustdoc

---

**Need help?** Check the [Troubleshooting Guide](troubleshooting.md) or [open an issue](https://github.com/GarthDB/rust-jira-mcp/issues) for support!
