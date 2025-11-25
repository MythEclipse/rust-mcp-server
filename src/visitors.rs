use syn::visit::Visit;
use crate::models::*;

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