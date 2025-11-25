use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rmcp::{
    model::*,
    ServerHandler,
};

#[derive(Clone)]
pub struct AstCache {
    map: Arc<RwLock<HashMap<String, String>>>,
}

impl AstCache {
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, path: String, code: String) {
        let mut map = self.map.write().await;
        map.insert(path, code);
    }

    pub async fn get(&self, path: &str) -> Option<String> {
        let map = self.map.read().await;
        map.get(path).cloned()
    }

    pub async fn get_all(&self) -> HashMap<String, String> {
        let map = self.map.read().await;
        map.clone()
    }
}

#[derive(Clone)]
pub struct MyServer {
    pub cache: AstCache,
}

impl MyServer {
    pub fn new() -> Self {
        Self {
            cache: AstCache::new(),
        }
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