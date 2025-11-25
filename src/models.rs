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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallGraph {
    pub calls: std::collections::HashMap<String, Vec<String>>, // caller -> callees
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeUsageGraph {
    pub usages: std::collections::HashMap<String, Vec<ReferenceLocation>>, // type -> usages
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleDependencyGraph {
    pub dependencies: std::collections::HashMap<String, Vec<String>>, // module -> dependencies
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub line_count: usize,
    pub complexity: usize,
    pub param_count: usize,
    pub visibility: String,
    pub file: String,
    pub range: Range,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub field_count: usize,
    pub file: String,
    pub range: Range,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumInfo {
    pub name: String,
    pub variant_count: usize,
    pub file: String,
    pub range: Range,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceGraphs {
    pub call_graph: CallGraph,
    pub type_usage_graph: TypeUsageGraph,
    pub module_dependency_graph: ModuleDependencyGraph,
    pub unused_functions: Vec<String>,
    pub refactoring_suggestions: Vec<String>,
    pub function_info: Vec<FunctionInfo>,
    pub struct_info: Vec<StructInfo>,
    pub enum_info: Vec<EnumInfo>,
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