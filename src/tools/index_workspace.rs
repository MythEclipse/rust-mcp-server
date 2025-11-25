use rmcp::{
    model::*,
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
};
use walkdir::WalkDir;
use syn::visit::Visit;
use crate::models::*;
use crate::cache::*;
use crate::visitors::{SymbolCollector, CallGraphCollector, TypeUsageCollector, ModuleDependencyCollector};
use std::collections::HashMap;

pub async fn index_workspace(
    server: &MyServer,
    Parameters(IndexWorkspaceParams { root }): Parameters<IndexWorkspaceParams>,
) -> Result<CallToolResult, McpError> {
    let mut call_graph = HashMap::new();
    let mut type_usage = HashMap::new();
    let mut module_deps = HashMap::new();
    let mut all_symbols = Vec::new();
    let mut all_functions = Vec::new();
    let mut all_structs = Vec::new();
    let mut all_enums = Vec::new();

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
                // Collect symbols
                let mut symbol_collector = SymbolCollector {
                    file: path.clone(),
                    out: Vec::new(),
                };
                symbol_collector.visit_file(&ast);
                all_symbols.extend(symbol_collector.out);

                // Collect call graph and function info
                let mut call_collector = CallGraphCollector {
                    file: path.clone(),
                    current_function: None,
                    calls: HashMap::new(),
                    function_info: HashMap::new(),
                };
                call_collector.visit_file(&ast);
                for (caller, callees) in call_collector.calls {
                    call_graph.entry(caller).or_insert(Vec::new()).extend(callees);
                }
                all_functions.extend(call_collector.function_info.values().cloned());

                // Collect type usage and struct/enum info
                let mut type_collector = TypeUsageCollector {
                    file: path.clone(),
                    usages: HashMap::new(),
                    struct_info: HashMap::new(),
                    enum_info: HashMap::new(),
                };
                type_collector.visit_file(&ast);
                for (type_name, locations) in type_collector.usages {
                    type_usage.entry(type_name).or_insert(Vec::new()).extend(locations);
                }
                all_structs.extend(type_collector.struct_info.values().cloned());
                all_enums.extend(type_collector.enum_info.values().cloned());

                // Collect module dependencies
                let mut mod_collector = ModuleDependencyCollector {
                    file: path.clone(),
                    dependencies: HashMap::new(),
                };
                mod_collector.visit_file(&ast);
                for (module, deps) in mod_collector.dependencies {
                    module_deps.entry(module).or_insert(Vec::new()).extend(deps);
                }
            }
        }
    }

    // Advanced code smell detection
    let unused_functions = detect_unused_functions(&all_functions, &call_graph);
    let refactoring_suggestions = generate_refactoring_suggestions(&all_functions, &all_structs, &all_enums, &call_graph, &type_usage);

    let graphs = WorkspaceGraphs {
        call_graph: CallGraph { calls: call_graph },
        type_usage_graph: TypeUsageGraph { usages: type_usage },
        module_dependency_graph: ModuleDependencyGraph { dependencies: module_deps },
        unused_functions,
        refactoring_suggestions,
        function_info: all_functions,
        struct_info: all_structs,
        enum_info: all_enums,
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&graphs).map_err(|e| McpError::internal_error(e.to_string(), None))?
    )]))
}

fn detect_unused_functions(functions: &[FunctionInfo], call_graph: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut used_functions = std::collections::HashSet::new();
    
    // Mark functions that are called
    for callees in call_graph.values() {
        for callee in callees {
            used_functions.insert(callee.clone());
        }
    }
    
    // Also mark main function and public functions as used (they might be entry points)
    for func in functions {
        if func.name == "main" || func.visibility == "public" {
            used_functions.insert(func.name.clone());
        }
    }
    
    // Find unused private functions
    functions.iter()
        .filter(|f| f.visibility == "private" && !used_functions.contains(&f.name))
        .map(|f| f.name.clone())
        .collect()
}

fn generate_refactoring_suggestions(
    functions: &[FunctionInfo], 
    structs: &[StructInfo], 
    enums: &[EnumInfo],
    call_graph: &HashMap<String, Vec<String>>,
    type_usage: &HashMap<String, Vec<ReferenceLocation>>
) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    // 1. Long functions (>50 lines)
    for func in functions {
        if func.line_count > 50 {
            suggestions.push(format!(
                "Function '{}' in {} is too long ({} lines). Consider breaking it into smaller functions.",
                func.name, func.file, func.line_count
            ));
        }
    }
    
    // 2. High complexity functions (>10 cyclomatic complexity)
    for func in functions {
        if func.complexity > 10 {
            suggestions.push(format!(
                "Function '{}' in {} has high complexity ({}). Consider simplifying the logic.",
                func.name, func.file, func.complexity
            ));
        }
    }
    
    // 3. Functions with too many parameters (>5)
    for func in functions {
        if func.param_count > 5 {
            suggestions.push(format!(
                "Function '{}' in {} has too many parameters ({}). Consider using a struct or builder pattern.",
                func.name, func.file, func.param_count
            ));
        }
    }
    
    // 4. Large structs (>10 fields)
    for struct_info in structs {
        if struct_info.field_count > 10 {
            suggestions.push(format!(
                "Struct '{}' in {} has too many fields ({}). Consider splitting into smaller structs.",
                struct_info.name, struct_info.file, struct_info.field_count
            ));
        }
    }
    
    // 5. Large enums (>10 variants)
    for enum_info in enums {
        if enum_info.variant_count > 10 {
            suggestions.push(format!(
                "Enum '{}' in {} has too many variants ({}). Consider using separate enums or structs.",
                enum_info.name, enum_info.file, enum_info.variant_count
            ));
        }
    }
    
    // 6. Functions that call many other functions (>10 callees)
    for (caller, callees) in call_graph {
        if callees.len() > 10 {
            suggestions.push(format!(
                "Function '{}' calls too many other functions ({}). Consider reducing coupling.",
                caller, callees.len()
            ));
        }
    }
    
    // 7. Functions that are called by many others (>10 callers)
    let mut caller_counts = HashMap::new();
    for callees in call_graph.values() {
        for callee in callees {
            *caller_counts.entry(callee.clone()).or_insert(0) += 1;
        }
    }
    
    for (callee, count) in caller_counts {
        if count > 10 {
            suggestions.push(format!(
                "Function '{}' is called by too many functions ({}). Consider introducing an interface or facade.",
                callee, count
            ));
        }
    }
    
    // 8. God object detection (structs used in many places)
    for struct_info in structs {
        let usage_count = type_usage.get(&struct_info.name).map(|locs| locs.len()).unwrap_or(0);
        if usage_count > 10 {
            suggestions.push(format!(
                "Struct '{}' in {} is used in too many places ({}). Consider breaking it into smaller components.",
                struct_info.name, struct_info.file, usage_count
            ));
        }
    }
    
    suggestions
}