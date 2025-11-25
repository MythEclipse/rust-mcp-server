# Rust MCP Server - AI Agent Instructions

## Architecture Overview

This is a **Model Context Protocol (MCP) server** written in Rust that provides AI-assisted code analysis tools. The server uses the `rmcp` crate for MCP protocol handling and `syn` for Rust AST parsing.

**Key Components:**
- `MyServer`: Main server struct with AST caching
- `AstCache`: Thread-safe cache for parsed source code
- Tools: Individual analysis functions registered via `#[tool_router]` macro
- Visitors: AST traversal implementations for code analysis

## Development Workflow

### Building & Running
```bash
cargo build          # Build the server
cargo run           # Run MCP server (communicates via stdio)
cargo test          # Run tests with temporary file fixtures
```

### Adding New Tools
1. Create tool implementation in `src/tools/your_tool.rs`
2. Add module declaration in `src/tools/mod.rs`
3. Register tool in `#[tool_router]` impl block with `#[tool(description = "...")]` attribute
4. Tool functions receive `&self` and `Parameters<YourParams>` and return `Result<CallToolResult, McpError>`

### Code Analysis Patterns

#### AST Parsing & Caching
```rust
// Always check cache first, then parse with syn
let code_opt = if let Some(code) = server.cache.get(&path).await {
    Some(code)
} else if let Ok(code) = tokio::fs::read_to_string(&path).await {
    server.cache.insert(path.clone(), code.clone()).await;
    Some(code)
} else {
    None
};

if let Some(code) = code_opt {
    if let Ok(ast) = syn::parse_file(&code) {
        // Use visitor pattern for analysis
        let mut visitor = YourVisitor::new();
        visitor.visit_file(&ast);
    }
}
```

#### Visitor Pattern Implementation
```rust
pub struct YourCollector {
    pub file: String,
    pub results: Vec<YourData>,
}

impl<'ast> Visit<'ast> for YourCollector {
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        // Extract data from AST nodes
        let span = i.sig.ident.span();
        // ... collect information
        syn::visit::visit_item_fn(self, i); // Continue traversal
    }
}
```

#### Error Handling
- Use `McpError` for tool errors
- Return `CallToolResult::success(vec![Content::text(json)])` for successful responses
- Use `serde_json::to_string()` for serializing results

### Testing Conventions

Use `tempfile` crate for creating test fixtures:
```rust
#[tokio::test]
async fn test_your_tool() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let code = r#"fn test() { println!("hello"); }"#;
    temp_file.write_all(code.as_bytes()).unwrap();
    let path = temp_file.path().to_string_lossy().to_string();

    let server = MyServer::new();
    let params = Parameters(YourParams { path });
    let result = server.your_tool(params).await.unwrap();
    // Assert on result.content
}
```

### Key Files to Reference

- `src/cache.rs`: Server and caching implementation
- `src/tools/mod.rs`: Tool registration pattern
- `src/visitors.rs`: AST visitor implementations
- `src/models.rs`: Data structures for analysis results
- `Cargo.toml`: Dependencies (note: uses git dependency for rmcp)

### MCP Protocol Notes

- Server communicates via stdio using MCP protocol
- Tools are discovered automatically via `#[tool]` attributes
- Use `rmcp::transport::stdio()` for transport layer
- Server info includes capabilities and tool descriptions

### Code Style Patterns

- Async functions throughout (tokio runtime)
- Visitor pattern for AST traversal
- Builder pattern for complex data structures
- Comprehensive error handling with `anyhow`
- JSON serialization for all tool outputs

## Quality Assurance Guidelines

### Implementation Verification
**ALWAYS verify existing code before making changes:**
- Search for placeholder comments like "TODO", "FIXME", "In a real implementation"
- Check for incomplete implementations marked with placeholder text
- Ensure all functions have real logic, not just `unimplemented!()` or empty bodies
- Verify that complex analysis functions actually perform the promised computations

### Code Smell Detection
**Be skeptical of:**
- Functions that return hardcoded values instead of computed results
- Comments indicating incomplete implementation ("placeholder", "stub", etc.)
- Test functions that don't actually validate behavior
- Error handling that just returns generic messages

### Validation Steps
Before committing changes:
1. Run `cargo check` to ensure compilation
2. Run `cargo test` to verify functionality
3. Search codebase for placeholder patterns:
   ```bash
   grep -r "TODO\|FIXME\|placeholder\|In a real implementation" src/
   ```
4. Verify that complex analysis functions (like `generate_refactoring_suggestions`) contain actual algorithmic logic, not just string concatenation

### Common Pitfalls to Avoid
- Placeholder refactoring suggestions that are just generic advice
- Incomplete graph analysis that doesn't traverse actual AST nodes
- Cache implementations that don't actually store/retrieve data
- Tool registrations without proper parameter validation</content>
<parameter name="filePath">d:\rust-mcp-server\.github\copilot-instructions.md