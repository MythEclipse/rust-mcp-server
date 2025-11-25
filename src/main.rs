mod models;
mod visitors;
mod cache;
mod tools;

use anyhow::Result;
use cache::MyServer;
use rmcp::ServiceExt;

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
    use rmcp::handler::server::wrapper::Parameters;
    use crate::models::*;

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