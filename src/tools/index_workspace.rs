use rmcp::{
    model::*,
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
};
use walkdir::WalkDir;
use syn::visit::Visit;
use crate::models::*;
use crate::cache::*;
use crate::visitors::*;

pub async fn index_workspace(
    server: &MyServer,
    Parameters(IndexWorkspaceParams { root }): Parameters<IndexWorkspaceParams>,
) -> Result<CallToolResult, McpError> {
    let mut symbols = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path().to_string_lossy().to_string();
        if !path.ends_with(".rs") { continue; }

        let code_opt = if let Some(code) = server.cache.get(&path).await {
            Some(code)
        } else {
            if let Ok(code) = tokio::fs::read_to_string(&path).await {
                server.cache.insert(path.clone(), code.clone()).await;
                Some(code)
            } else {
                None
            }
        };

        if let Some(code) = code_opt {
            if let Ok(ast) = syn::parse_file(&code) {
                let mut collector = SymbolCollector {
                    file: path.clone(),
                    out: Vec::new(),
                };
                collector.visit_file(&ast);
                symbols.extend(collector.out);
            }
        }
    }
    
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&symbols).map_err(|e| McpError::internal_error(e.to_string(), None))?
    )]))
}