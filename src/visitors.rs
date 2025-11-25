use syn::visit::Visit;
use crate::models::*;
use std::collections::HashMap;

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

pub struct CallGraphCollector {
    pub file: String,
    pub current_function: Option<String>,
    pub calls: HashMap<String, Vec<String>>,
    pub function_info: HashMap<String, crate::models::FunctionInfo>,
}

impl<'ast> Visit<'ast> for CallGraphCollector {
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        let fn_name = i.sig.ident.to_string();
        self.current_function = Some(fn_name.clone());
        
        // Calculate function metrics
        let span = i.sig.ident.span();
        let start_line = span.start().line;
        let end_line = span.end().line;
        let line_count = end_line - start_line + 1;
        
        // Calculate complexity (simplified cyclomatic complexity)
        let mut complexity = 1; // base complexity
        self.calculate_complexity(&i.block, &mut complexity);
        
        let param_count = i.sig.inputs.len();
        
        let visibility = if matches!(i.vis, syn::Visibility::Public(_)) {
            "public"
        } else {
            "private"
        };
        
        let info = FunctionInfo {
            name: fn_name.clone(),
            line_count,
            complexity,
            param_count,
            visibility: visibility.to_string(),
            file: self.file.clone(),
            range: Range {
                start: Position { line: start_line, character: span.start().column },
                end: Position { line: end_line, character: span.end().column },
            },
        };
        
        self.function_info.insert(fn_name.clone(), info);
        self.calls.entry(fn_name).or_insert(Vec::new());
        
        syn::visit::visit_item_fn(self, i);
        self.current_function = None;
    }

    fn visit_expr_call(&mut self, i: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*i.func {
            if let Some(segment) = path.path.segments.last() {
                let callee = segment.ident.to_string();
                if let Some(caller) = &self.current_function {
                    self.calls.entry(caller.clone()).or_insert(Vec::new()).push(callee);
                }
            }
        }
        syn::visit::visit_expr_call(self, i);
    }
    
    fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        let method_name = i.method.to_string();
        if let Some(caller) = &self.current_function {
            self.calls.entry(caller.clone()).or_insert(Vec::new()).push(method_name);
        }
        syn::visit::visit_expr_method_call(self, i);
    }
}

impl CallGraphCollector {
    fn calculate_complexity(&mut self, block: &syn::Block, complexity: &mut usize) {
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Expr(expr, _) => {
                    self.calculate_expr_complexity(expr, complexity);
                }
                syn::Stmt::Local(local) => {
                    if let Some(init) = &local.init {
                        self.calculate_expr_complexity(&init.expr, complexity);
                    }
                }
                _ => {}
            }
        }
    }
    
    fn calculate_expr_complexity(&mut self, expr: &syn::Expr, complexity: &mut usize) {
        match expr {
            syn::Expr::If(_) | syn::Expr::Match(_) => *complexity += 1,
            syn::Expr::Loop(_) | syn::Expr::While(_) | syn::Expr::ForLoop(_) => *complexity += 1,
            syn::Expr::Binary(bin) if matches!(bin.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) => *complexity += 1,
            syn::Expr::Block(block) => self.calculate_complexity(&block.block, complexity),
            _ => {}
        }
        syn::visit::visit_expr(self, expr);
    }
}

pub struct TypeUsageCollector {
    pub file: String,
    pub usages: HashMap<String, Vec<ReferenceLocation>>,
    pub struct_info: HashMap<String, crate::models::StructInfo>,
    pub enum_info: HashMap<String, crate::models::EnumInfo>,
}

impl<'ast> Visit<'ast> for TypeUsageCollector {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        let struct_name = i.ident.to_string();
        let span = i.ident.span();
        let start = span.start();
        let end = span.end();
        
        let field_count = match &i.fields {
            syn::Fields::Named(fields) => fields.named.len(),
            syn::Fields::Unnamed(fields) => fields.unnamed.len(),
            syn::Fields::Unit => 0,
        };
        
        let info = StructInfo {
            name: struct_name.clone(),
            field_count,
            file: self.file.clone(),
            range: Range {
                start: Position { line: start.line, character: start.column },
                end: Position { line: end.line, character: end.column },
            },
        };
        
        self.struct_info.insert(struct_name, info);
        syn::visit::visit_item_struct(self, i);
    }
    
    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        let enum_name = i.ident.to_string();
        let span = i.ident.span();
        let start = span.start();
        let end = span.end();
        
        let variant_count = i.variants.len();
        
        let info = EnumInfo {
            name: enum_name.clone(),
            variant_count,
            file: self.file.clone(),
            range: Range {
                start: Position { line: start.line, character: start.column },
                end: Position { line: end.line, character: end.column },
            },
        };
        
        self.enum_info.insert(enum_name, info);
        syn::visit::visit_item_enum(self, i);
    }

    fn visit_type_path(&mut self, i: &'ast syn::TypePath) {
        if let Some(seg) = i.path.segments.last() {
            let type_name = seg.ident.to_string();
            let span = seg.ident.span();
            let start = span.start();
            let end = span.end();
            self.usages.entry(type_name).or_insert(Vec::new()).push(ReferenceLocation {
                file: self.file.clone(),
                range: Range {
                    start: Position { line: start.line, character: start.column },
                    end: Position { line: end.line, character: end.column },
                },
            });
        }
        syn::visit::visit_type_path(self, i);
    }
    
    fn visit_path(&mut self, i: &'ast syn::Path) {
        if let Some(seg) = i.segments.last() {
            let type_name = seg.ident.to_string();
            let span = seg.ident.span();
            let start = span.start();
            let end = span.end();
            self.usages.entry(type_name).or_insert(Vec::new()).push(ReferenceLocation {
                file: self.file.clone(),
                range: Range {
                    start: Position { line: start.line, character: start.column },
                    end: Position { line: end.line, character: end.column },
                },
            });
        }
        syn::visit::visit_path(self, i);
    }
}

pub struct ModuleDependencyCollector {
    pub file: String,
    pub dependencies: HashMap<String, Vec<String>>,
}

impl<'ast> Visit<'ast> for ModuleDependencyCollector {
    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        // Simple extraction of use statements
        let mut deps = Vec::new();
        extract_use_paths(&i.tree, &mut deps);
        let module_name = self.file.clone(); // or extract module name
        self.dependencies.entry(module_name).or_insert(Vec::new()).extend(deps);
        syn::visit::visit_item_use(self, i);
    }
}

fn extract_use_paths(tree: &syn::UseTree, deps: &mut Vec<String>) {
    match tree {
        syn::UseTree::Path(path) => {
            deps.push(path.ident.to_string());
            extract_use_paths(&path.tree, deps);
        }
        syn::UseTree::Name(name) => {
            deps.push(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) => {
            deps.push(rename.ident.to_string());
        }
        syn::UseTree::Glob(_) => {
            // For glob, we might not extract specific names
        }
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                extract_use_paths(tree, deps);
            }
        }
    }
}