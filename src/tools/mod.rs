pub mod check_file;
pub mod index_workspace;
pub mod goto_definition;
pub mod find_references;
pub mod server_handler;

use rmcp::{
    model::*,
    tool,
    tool_router,
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
};
use crate::models::*;
use crate::cache::MyServer;

#[tool_router]
impl MyServer {
    #[tool(description = "Parse and check a Rust file for syntax errors")]
    pub async fn check_file(
        &self,
        params: Parameters<CheckFileParams>,
    ) -> Result<CallToolResult, McpError> {
        check_file::check_file(self, params).await
    }

    #[tool(description = "Index all Rust files in a directory and build call graph, type usage graph, and module dependency graph for AI navigation and code analysis")]
    pub async fn index_workspace(
        &self,
        params: Parameters<IndexWorkspaceParams>,
    ) -> Result<CallToolResult, McpError> {
        index_workspace::index_workspace(self, params).await
    }

    #[tool(description = "Find definition of a symbol")]
    pub async fn goto_definition(
        &self,
        params: Parameters<GotoDefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        goto_definition::goto_definition(self, params).await
    }

    #[tool(description = "Find references of a symbol")]
    pub async fn find_references(
        &self,
        params: Parameters<FindReferencesParams>,
    ) -> Result<CallToolResult, McpError> {
        find_references::find_references(self, params).await
    }
}

include!("server_handler.rs");