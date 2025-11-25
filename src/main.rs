use anyhow::Result;
use rmcp::handler::server::router::Router;
use rmcp::handler::server::Server;
use rmcp::transport::StdioTransport;
use rmcp::model::{CallToolRequest, CallToolResult, Content, Tool, ToolInputSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use walkdir::WalkDir;
use syn::{visit::Visit, File, spanned::Spanned};

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

#[derive(Clone)]
struct AstCache {
    map: Arc<RwLock<HashMap<String, Arc<File>>>>,
}

impl AstCache {
    fn new() -> Self {
        Self { map: Arc::new(RwLock::new(HashMap::new())) }
    }

    async fn insert(&self, path: String, ast: File) {
        self.map.write().await.insert(path, Arc::new(ast));
    }

    async fn get(&self, path: &str) -> Option<Arc<File>> {
        self.map.read().await.get(path).cloned()
    }

    async fn get_all(&self) -> HashMap<String, Arc<File>> {
        self.map.read().await.clone()
    }
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub range: Range,
    pub severity: String,
}

#[derive(Serialize)]
pub struct SymbolInfo {
    pub kind: String,
    pub name: String,
    pub file: String,
    pub range: Range,
}

#[derive(Serialize)]
pub struct ReferenceLocation {
    pub file: String,
    pub range: Range,
}

struct SymbolCollector {
    file: String,
    out: Vec<SymbolInfo>,
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

struct ReferenceFinder {
    target_name: String,
    file: String,
    matches: Vec<ReferenceLocation>,
}

impl<'ast> Visit<'ast> for ReferenceFinder {
    fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
        if i.to_string() == self.target_name {
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

#[tokio::main]
async fn main() -> Result<()> {
    let cache = AstCache::new();
    let router = Router::new();

    let check_file_cache = cache.clone();
    let router = router.tool(
        Tool::new(
            "check_file",
            "Parse and check a Rust file for syntax errors",
            ToolInputSchema::new(&json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            })),
        ),
        move |req: CallToolRequest| {
            let cache = check_file_cache.clone();
            Box::pin(async move {
                let path = req.arguments.get("path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing path"))?;
                let code = tokio::fs::read_to_string(path).await?;
                
                let diagnostics = match syn::parse_file(&code) {
                    Ok(ast) => {
                        cache.insert(path.to_string(), ast).await;
                        vec![]
                    },
                    Err(e) => {
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
                    }
                };
                
                Ok(CallToolResult {
                    content: vec![Content::Text(serde_json::to_string(&diagnostics)?)],
                    is_error: false,
                })
            })
        },
    );

    let index_cache = cache.clone();
    let router = router.tool(
        Tool::new(
            "index_workspace",
            "Index all Rust files in a directory",
            ToolInputSchema::new(&json!({
                "type": "object",
                "properties": {
                    "root": { "type": "string" }
                },
                "required": ["root"]
            })),
        ),
        move |req: CallToolRequest| {
            let cache = index_cache.clone();
            Box::pin(async move {
                let root = req.arguments.get("root").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing root"))?;
                let mut symbols = Vec::new();

                for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                    if !entry.file_type().is_file() { continue; }
                    let path = entry.path().to_string_lossy().to_string();
                    if !path.ends_with(".rs") { continue; }

                    let ast_opt = if let Some(ast) = cache.get(&path).await {
                        Some(ast)
                    } else {
                        if let Ok(code) = tokio::fs::read_to_string(&path).await {
                            if let Ok(parsed) = syn::parse_file(&code) {
                                cache.insert(path.clone(), parsed.clone()).await;
                                Some(Arc::new(parsed))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(ast) = ast_opt {
                        let mut collector = SymbolCollector {
                            file: path.clone(),
                            out: Vec::new(),
                        };
                        collector.visit_file(&ast);
                        symbols.extend(collector.out);
                    }
                }
                
                Ok(CallToolResult {
                    content: vec![Content::Text(serde_json::to_string(&symbols)?)],
                    is_error: false,
                })
            })
        },
    );

    let def_cache = cache.clone();
    let router = router.tool(
        Tool::new(
            "goto_definition",
            "Find definition of a symbol",
            ToolInputSchema::new(&json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                },
                "required": ["name"]
            })),
        ),
        move |req: CallToolRequest| {
            let cache = def_cache.clone();
            Box::pin(async move {
                let name = req.arguments.get("name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing name"))?;
                let mut results = Vec::new();
                let map = cache.get_all().await;
                
                for (path, ast) in map.iter() {
                    let mut collector = SymbolCollector {
                        file: path.clone(),
                        out: Vec::new(),
                    };
                    collector.visit_file(ast);
                    for sym in collector.out {
                        if sym.name == name {
                            results.push(sym);
                        }
                    }
                }
                
                Ok(CallToolResult {
                    content: vec![Content::Text(serde_json::to_string(&results)?)],
                    is_error: false,
                })
            })
        },
    );

    let refs_cache = cache.clone();
    let router = router.tool(
        Tool::new(
            "find_references",
            "Find references of a symbol",
            ToolInputSchema::new(&json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                },
                "required": ["name"]
            })),
        ),
        move |req: CallToolRequest| {
            let cache = refs_cache.clone();
            Box::pin(async move {
                let name = req.arguments.get("name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing name"))?;
                let mut refs = Vec::new();
                let map = cache.get_all().await;

                for (path, ast) in map.iter() {
                    let mut finder = ReferenceFinder {
                        target_name: name.to_string(),
                        file: path.clone(),
                        matches: Vec::new(),
                    };
                    finder.visit_file(ast);
                    refs.extend(finder.matches);
                }

                Ok(CallToolResult {
                    content: vec![Content::Text(serde_json::to_string(&refs)?)],
                    is_error: false,
                })
            })
        },
    );

    let transport = StdioTransport::new();
    let server = Server::new(router, transport);

    server.start().await?;
    Ok(())
}