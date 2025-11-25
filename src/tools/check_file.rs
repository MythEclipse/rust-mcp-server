use rmcp::{
    model::*,
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
};
use serde_json::json;
use crate::models::*;
use crate::cache::*;

pub async fn check_file(
    server: &MyServer,
    Parameters(CheckFileParams { path }): Parameters<CheckFileParams>,
) -> Result<CallToolResult, McpError> {
    let code = tokio::fs::read_to_string(&path).await
        .map_err(|e| McpError::invalid_params("Failed to read file", Some(json!({ "error": e.to_string() }))))?;
    
    let diagnostics = if let Err(e) = syn::parse_file(&code) {
        let span = e.span();
        let start = span.start();
        let end = span.end();
        vec![Diagnostic {
            message: e.to_string(),
            range: Range {
                start: Position { line: start.line, character: start.column },
                end: Position { line: end.line, character: end.column },
            },
            severity: "error".to_string(),
        }]
    } else {
        server.cache.insert(path.to_string(), code.clone()).await;
        vec![]
    };
    
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&diagnostics).map_err(|e| McpError::internal_error(e.to_string(), None))?
    )]))
}