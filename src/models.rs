use serde::{Deserialize, Serialize};

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
    pub name: String,
    pub kind: String,
    pub range: Range,
    pub file: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceLocation {
    pub file: String,
    pub range: Range,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CheckFileParams {
    pub path: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct IndexWorkspaceParams {
    pub root: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct GotoDefinitionParams {
    pub name: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct FindReferencesParams {
    pub name: String,
}