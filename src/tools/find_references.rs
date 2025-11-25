use rmcp::{
    model::*,
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
};
use syn::visit::Visit;
use crate::models::*;
use crate::cache::*;
use crate::visitors::*;

pub async fn find_references(
    server: &MyServer,
    Parameters(FindReferencesParams { name }): Parameters<FindReferencesParams>,
) -> Result<CallToolResult, McpError> {
    let mut refs = Vec::new();
    let code_map = server.cache.get_all().await;

    for (path, code) in code_map.iter() {
        if let Ok(ast) = syn::parse_file(code) {
            let mut finder = ReferenceFinder {
                target_name: name.to_string(),
                file: path.clone(),
                matches: Vec::new(),
            };
            finder.visit_file(&ast);
            refs.extend(finder.matches);
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&refs).map_err(|e| McpError::internal_error(e.to_string(), None))?
    )]))
}