# Refactor `list_tools()` Using Category-Based Helper Functions

## Objective
Break down the 1048-line `list_tools()` function in `src/mcp/server.rs` into smaller helper functions organized by tool categories to fix `clippy::too_many_lines` violation.

## Current State
- **File**: `src/mcp/server.rs`
- **Function**: `list_tools()` (lines ~429-1486)
- **Size**: 1048 lines (exceeds 100-line limit)
- **Content**: 40+ MCPTool definitions with detailed JSON schemas

## Target Structure
```rust
pub fn list_tools() -> Vec<MCPTool> {
    let mut tools = Vec::new();
    tools.extend(Self::get_basic_tool_definitions());
    tools.extend(Self::get_project_tool_definitions());
    tools.extend(Self::get_bulk_tool_definitions());
    tools.extend(Self::get_linking_tool_definitions());
    tools.extend(Self::get_attachment_tool_definitions());
    tools.extend(Self::get_worklog_tool_definitions());
    tools.extend(Self::get_watcher_tool_definitions());
    tools.extend(Self::get_label_tool_definitions());
    tools.extend(Self::get_component_tool_definitions());
    tools.extend(Self::get_cloning_tool_definitions());
    tools.extend(Self::get_zephyr_tool_definitions());
    tools
}
```

## Tool Categories to Implement

### 1. Basic Tools (`get_basic_tool_definitions()`)
- `test_jira_auth`
- `search_jira_issues`
- `create_jira_issue`
- `update_jira_issue`
- `get_jira_issue`
- `get_jira_comments`
- `add_jira_comment`
- `get_jira_transitions`
- `transition_jira_issue`

### 2. Project Tools (`get_project_tool_definitions()`)
- `get_project_config`
- `get_project_issue_types`
- `get_issue_type_metadata`
- `get_project_components`
- `get_priorities_and_statuses`
- `get_custom_fields`
- `get_project_metadata`

### 3. Bulk Tools (`get_bulk_tool_definitions()`)
- `bulk_update_issues`
- `bulk_transition_issues`
- `bulk_add_comments`
- `mixed_bulk_operations`

### 4. Linking Tools (`get_linking_tool_definitions()`)
- `get_jira_link_types`
- `get_jira_issue_links`
- `create_jira_issue_link`
- `delete_jira_issue_link`

### 5. Attachment Tools (`get_attachment_tool_definitions()`)
- `get_jira_issue_attachments`
- `upload_jira_attachment`
- `delete_jira_attachment`
- `download_jira_attachment`

### 6. Work Log Tools (`get_worklog_tool_definitions()`)
- `get_jira_issue_work_logs`
- `add_jira_work_log`
- `update_jira_work_log`
- `delete_jira_work_log`

### 7. Watcher Tools (`get_watcher_tool_definitions()`)
- `get_jira_issue_watchers`
- `add_jira_issue_watcher`
- `remove_jira_issue_watcher`

### 8. Label Tools (`get_label_tool_definitions()`)
- `get_jira_labels`
- `create_jira_label`
- `update_jira_label`
- `delete_jira_label`

### 9. Component Tools (`get_component_tool_definitions()`)
- `create_jira_component`
- `update_jira_component`
- `delete_jira_component`

### 10. Cloning Tools (`get_cloning_tool_definitions()`)
- `clone_jira_issue`

### 11. Zephyr Tools (`get_zephyr_tool_definitions()`)
- `get_zephyr_test_steps`
- `create_zephyr_test_step`
- `update_zephyr_test_step`
- `delete_zephyr_test_step`
- `get_zephyr_test_cases`
- `create_zephyr_test_case`
- `get_zephyr_test_executions`
- `create_zephyr_test_execution`
- `get_zephyr_test_cycles`
- `get_zephyr_test_plans`

## Implementation Steps

1. **Start with Basic Tools**
   - Find the first 9 tools in the current `list_tools()` function
   - Extract them into `get_basic_tool_definitions()` helper function
   - Test compilation

2. **Continue with Project Tools**
   - Find the next 7 tools (project-related)
   - Extract into `get_project_tool_definitions()` helper function
   - Test compilation

3. **Repeat for Each Category**
   - Extract tools category by category
   - Test after each extraction
   - Ensure no tools are missed

4. **Final Verification**
   - Run `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
   - Verify all tools are preserved
   - Check that `list_tools()` is under 100 lines

## Helper Function Template
```rust
/// Get [category] tool definitions
fn get_[category]_tool_definitions() -> Vec<MCPTool> {
    vec![
        MCPTool {
            name: "tool_name".to_string(),
            description: "Tool description".to_string(),
            input_schema: json!({
                // ... schema definition
            }),
        },
        // ... more tools
    ]
}
```

## Success Criteria
- [ ] `list_tools()` function is under 100 lines
- [ ] All helper functions are under 100 lines
- [ ] No `#[allow]` attributes are used
- [ ] All 40+ tools are preserved in the same order
- [ ] JSON schemas remain identical
- [ ] Code compiles without warnings
- [ ] Pre-commit hook passes

## Testing Command
```bash
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
```

## Notes
- Each helper function should return `Vec<MCPTool>`
- Use descriptive function names
- Maintain the exact same tool order as the original
- Preserve all JSON schema details exactly
- Test compilation after each category extraction

This approach will systematically break down the large function while maintaining all functionality and ensuring clippy compliance.
