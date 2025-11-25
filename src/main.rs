use anyhow::Result;
use rmcp::{
    ErrorData as McpError,
    ServerHandler,
    handler::server::wrapper::Parameters,
    model::*,
    tool,
    tool_router,
    ServiceExt,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use walkdir::WalkDir;
use syn::visit::Visit;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub range: Range,
    pub severity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub kind: String,
    pub name: String,
    pub file: String,
    pub range: Range,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceLocation {
    pub file: String,
    pub range: Range,
}

#[derive(Clone)]
pub struct AstCache {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl AstCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, path: String, code: String) {
        let mut map = self.cache.write().await;
        map.insert(path, code);
    }

    pub async fn get(&self, path: &str) -> Option<String> {
        let map = self.cache.read().await;
        map.get(path).cloned()
    }

    pub async fn get_all(&self) -> HashMap<String, String> {
        let map = self.cache.read().await;
        map.clone()
    }
}

pub struct SymbolCollector {
    pub file: String,
    pub out: Vec<SymbolInfo>,
}

impl<'ast> Visit<'ast> for SymbolCollector {
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        let span = i.sig.ident.span();
        let start = span.start();
        let end = span.end();
        
        self.out.push(SymbolInfo {
            kind: "fn".to_string(),
            name: i.sig.ident.to_string(),
            file: self.file.clone(),
            range: Range {
                start: Position { line: start.line, character: start.column },
                end: Position { line: end.line, character: end.column },
            },
        });
        syn::visit::visit_item_fn(self, i);
    }

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        let span = i.ident.span();
        let start = span.start();
        let end = span.end();

        self.out.push(SymbolInfo {
            kind: "struct".to_string(),
            name: i.ident.to_string(),
            file: self.file.clone(),
            range: Range {
                start: Position { line: start.line, character: start.column },
                end: Position { line: end.line, character: end.column },
            },
        });
        syn::visit::visit_item_struct(self, i);
    }

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        let span = i.ident.span();
        let start = span.start();
        let end = span.end();

        self.out.push(SymbolInfo {
            kind: "enum".to_string(),
            name: i.ident.to_string(),
            file: self.file.clone(),
            range: Range {
                start: Position { line: start.line, character: start.column },
                end: Position { line: end.line, character: end.column },
            },
        });
        syn::visit::visit_item_enum(self, i);
    }

    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {
        let span = i.ident.span();
        let start = span.start();
        let end = span.end();

        self.out.push(SymbolInfo {
            kind: "trait".to_string(),
            name: i.ident.to_string(),
            file: self.file.clone(),
            range: Range {
                start: Position { line: start.line, character: start.column },
                end: Position { line: end.line, character: end.column },
            },
        });
        syn::visit::visit_item_trait(self, i);
    }
}

pub struct ReferenceFinder {
    pub target_name: String,
    pub file: String,
    pub matches: Vec<ReferenceLocation>,
}

impl<'ast> Visit<'ast> for ReferenceFinder {
    fn visit_ident(&mut self, i: &'ast syn::Ident) {
        if i == &self.target_name {
            let span = i.span();
            let start = span.start();
            let end = span.end();
            self.matches.push(ReferenceLocation {
                file: self.file.clone(),
                range: Range {
                    start: Position { line: start.line, character: start.column },
                    end: Position { line: end.line, character: end.column },
                },
            });
        }
    }
    
    fn visit_type_path(&mut self, i: &'ast syn::TypePath) {
        if let Some(seg) = i.path.segments.last() {
            if seg.ident.to_string() == self.target_name {
                let span = seg.ident.span();
                let start = span.start();
                let end = span.end();
                self.matches.push(ReferenceLocation {
                    file: self.file.clone(),
                    range: Range {
                        start: Position { line: start.line, character: start.column },
                        end: Position { line: end.line, character: end.column },
                    },
                });
            }
        }
        syn::visit::visit_type_path(self, i);
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CheckFileParams {
    pub path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IndexWorkspaceParams {
    pub root: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GotoDefinitionParams {
    pub name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindReferencesParams {
    pub name: String,
}

#[derive(Clone)]
pub struct MyServer {
    #[allow(dead_code)]
    cache: AstCache,
}

impl MyServer {
    pub fn new() -> Self {
        Self {
            cache: AstCache::new(),
        }
    }
}

#[tool_router]
impl MyServer {
    #[tool(description = "Parse and check a Rust file for syntax errors")]
    async fn check_file(
        &self,
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
            self.cache.insert(path.to_string(), code.clone()).await;
            vec![]
        };
        
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string(&diagnostics).map_err(|e| McpError::internal_error(e.to_string(), None))?
        )]))
    }

    #[tool(description = "Index all Rust files in a directory")]
    async fn index_workspace(
        &self,
        Parameters(IndexWorkspaceParams { root }): Parameters<IndexWorkspaceParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut symbols = Vec::new();

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() { continue; }
            let path = entry.path().to_string_lossy().to_string();
            if !path.ends_with(".rs") { continue; }

            let code_opt = if let Some(code) = self.cache.get(&path).await {
                Some(code)
            } else {
                if let Ok(code) = tokio::fs::read_to_string(&path).await {
                    self.cache.insert(path.clone(), code.clone()).await;
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

    #[tool(description = "Find definition of a symbol")]
    async fn goto_definition(
        &self,
        Parameters(GotoDefinitionParams { name }): Parameters<GotoDefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut results = Vec::new();
        let code_map = self.cache.get_all().await;
        
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

    #[tool(description = "Find references of a symbol")]
    async fn find_references(
        &self,
        Parameters(FindReferencesParams { name }): Parameters<FindReferencesParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut refs = Vec::new();
        let code_map = self.cache.get_all().await;

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
}

impl ServerHandler for MyServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides Rust code analysis tools.".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let server = MyServer::new();
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;

    #[tokio::test]
    async fn test_check_file_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let valid_code = r#"fn main() {
    println!("Hello, world!");
}"#;
        temp_file.write_all(valid_code.as_bytes()).unwrap();
        let path = temp_file.path().to_string_lossy().to_string();

        let server = MyServer::new();
        let params = Parameters(CheckFileParams { path });
        let result = server.check_file(params).await.unwrap();

        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_check_file_invalid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_code = r#"fn main() {
    println!("Hello, world!" // missing )
}"#;
        temp_file.write_all(invalid_code.as_bytes()).unwrap();
        let path = temp_file.path().to_string_lossy().to_string();

        let server = MyServer::new();
        let params = Parameters(CheckFileParams { path });
        let result = server.check_file(params).await.unwrap();

        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_goto_definition() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let code = r#"fn foo() {
    println!("foo");
}

fn main() {
    foo();
}"#;
        temp_file.write_all(code.as_bytes()).unwrap();
        let path = temp_file.path().to_string_lossy().to_string();

        let server = MyServer::new();
        // First, check_file to cache the code
        let params_check = Parameters(CheckFileParams { path: path.clone() });
        server.check_file(params_check).await.unwrap();

        // Now, goto_definition for "foo"
        let params_goto = Parameters(GotoDefinitionParams { name: "foo".to_string() });
        let result = server.goto_definition(params_goto).await.unwrap();

        assert_eq!(result.content.len(), 1);
        // Should have found the definition
    }

    #[tokio::test]
    async fn test_index_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Create some Rust files in the temp dir
        let file1_path = temp_dir.path().join("lib.rs");
        let mut file1 = std::fs::File::create(&file1_path).unwrap();
        file1.write_all(b"fn foo() {}\n").unwrap();

        let file2_path = temp_dir.path().join("main.rs");
        let mut file2 = std::fs::File::create(&file2_path).unwrap();
        file2.write_all(b"fn main() { foo(); }\n").unwrap();

        let server = MyServer::new();
        let params = Parameters(IndexWorkspaceParams { root: dir_path });
        let result = server.index_workspace(params).await.unwrap();

        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_find_references() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Create Rust files
        let file1_path = temp_dir.path().join("lib.rs");
        let mut file1 = std::fs::File::create(&file1_path).unwrap();
        file1.write_all(b"fn foo() {}\n").unwrap();

        let file2_path = temp_dir.path().join("main.rs");
        let mut file2 = std::fs::File::create(&file2_path).unwrap();
        file2.write_all(b"fn main() { foo(); }\n").unwrap();

        let server = MyServer::new();
        // Index first
        let params_index = Parameters(IndexWorkspaceParams { root: dir_path });
        server.index_workspace(params_index).await.unwrap();

        // Now find references for "foo"
        let params_find = Parameters(FindReferencesParams { name: "foo".to_string() });
        let result = server.find_references(params_find).await.unwrap();

        assert_eq!(result.content.len(), 1);
        // Should have found references
    }
}