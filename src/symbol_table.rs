use crate::hazards::{ErrorId, Hazard, HazardType, Location, WarnId};
use std::cell::Cell;

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn push_symbol(&mut self, scope: usize, ty: String, ident: String, span: (usize, usize)) {
        let symbol = Symbol::new(scope, ty, ident, span);
        self.symbols.push(symbol);
    }

    pub fn symbols_in_scope(&self, scope: usize) -> Vec<&Symbol> {
        self.symbols.iter().filter(|s| s.scope == scope).collect()
    }

    pub fn symbols_in_scope_mut(&mut self, scope: usize) -> Vec<&mut Symbol> {
        self.symbols
            .iter_mut()
            .filter(|s| s.scope == scope)
            .collect()
    }

    pub fn output(&self) -> String {
        let mut out = String::new();

        for s in &self.symbols {
            out.push_str(&s.output());
            out.push('\n')
        }
        out
    }
}

#[derive(Debug, Default)]
pub struct Symbol {
    pub scope: usize,
    pub ty: String, // ty = type. Should probably not be a string, but don't have types yet
    pub ident: String, // identifier
    pub span: (usize, usize),
    pub used: Cell<bool>,
    pub initialized: Cell<bool>,
}

impl Symbol {
    pub fn new(scope: usize, ty: String, ident: String, span: (usize, usize)) -> Self {
        Self {
            scope,
            ty,
            ident,
            span,
            used: Cell::new(false),
            initialized: Cell::new(false),
        }
    }
    pub fn output(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.scope.to_string());
        out.push(',');
        out.push_str(&self.ty.to_string());
        out.push(',');
        out.push_str(&self.ident.to_string());
        out.clone().to_owned()
    }
}

#[derive(Debug, Default)]
pub struct SymbolVisitor {
    table: SymbolTable,
    scope: usize,
    errored: bool,
}

impl SymbolVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_symbols(&self) -> Vec<&Symbol> {
        self.table.symbols_in_scope(self.scope)
    }

    pub fn exists(&self, ident: &str) -> bool {
        (0..=self.scope)
            .map(|s| self.table.symbols_in_scope(s).into_iter())
            .flatten()
            .any(|s| s.ident == ident)
    }

    pub fn initialized(&self, ident: &str) -> bool {
        for symbols in (0..=self.scope)
            .rev()
            .map(|s| self.table.symbols_in_scope(s))
        {
            for symbol in symbols {
                if symbol.ident == ident {
                    return symbol.initialized.get();
                }
            }
        }

        false
    }

    pub fn set_used(&self, ident: &str) {
        for symbols in (0..=self.scope)
            .rev()
            .map(|s| self.table.symbols_in_scope(s))
        {
            for symbol in symbols {
                if symbol.ident == ident {
                    symbol.used.set(true);
                }
            }
        }
    }

    pub fn report_unused(&self) {
        self.table
            .symbols
            .iter()
            .filter(|s| !s.used.get())
            .for_each(|s| {
                let hazard =
                    Hazard::new_one_loc(HazardType::Warn(WarnId::Unused), s.span.0, s.span.1);
                println!("{}", hazard.show_output());
            })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn symbol_table_output() {
//         let s1 = Symbol::new(0, String::from("test_type"), String::from("test_ident"));
//         let s2 = Symbol::new(
//             0,
//             String::from("test_type_var_2"),
//             String::from("test_ident_var_2"),
//         );
//         assert_eq!(s1.output(), "0,test_type,test_ident");
//         let syms = vec![s1, s2];
//         let table = SymbolTable { symbols: syms };
//         let output = table.output();
//         assert_eq!(
//             output,
//             r#"0,test_type,test_ident
// 0,test_type_var_2,test_ident_var_2
// "#
//         );
//     }
// }
