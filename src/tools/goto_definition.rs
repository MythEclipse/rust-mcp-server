use rmcp::{
    model::*,
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
};
use syn::visit::Visit;
use crate::models::*;
use crate::cache::*;
use crate::visitors::*;

pub async fn goto_definition(
    server: &MyServer,
    Parameters(GotoDefinitionParams { name }): Parameters<GotoDefinitionParams>,
) -> Result<CallToolResult, McpError> {
    let mut results = Vec::new();
    let code_map = server.cache.get_all().await;
    
    for (path, code) in code_map.iter() {
        if let Ok(ast) = syn::parse_file(code) {
            let mut collector = SymbolCollector {
                file: path.clone(),
                out: Vec::new(),
            };
            collector.visit_file(&ast);
            for sym in collector.out {
                if sym.name == name {
                    results.push(sym);
                }
            }
        }
    }
    
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&results).map_err(|e| McpError::internal_error(e.to_string(), None))?
    )]))
}