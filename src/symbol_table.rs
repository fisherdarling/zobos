use crate::ast::{AstKind, AstNode};
use crate::hazards::{ErrorId, Hazard, HazardType, Location, WarnId};
use std::cell::Cell;

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub valid_scopes: Vec<usize>, // which scopes can we currently 'see'
}

impl SymbolTable {
    pub fn push_symbol(
        &mut self,
        scope: usize,
        ty: String,
        ident: String,
        span: (usize, usize),
        const_: bool,
    ) {
        self.check_for_redeclare(&ident, span);
        let symbol = Symbol::new(scope, ty, ident, span, const_);
        self.symbols.push(symbol);
    }

    pub fn push_symbol_init(
        &mut self,
        scope: usize,
        ty: String,
        ident: String,
        span: (usize, usize),
        const_: bool,
    ) {
        self.check_for_redeclare(&ident, span);
        let symbol = Symbol::new(scope, ty, ident, span, const_);
        symbol.initialized.set(true);
        self.symbols.push(symbol);
    }

    pub fn check_for_redeclare(&self, ident: &str, span: (usize, usize)) {
        let current_valid_symbols = self.symbols_in_valid_scope();
        let is_redeclare = current_valid_symbols.iter().any(|s| s.ident == ident);
        if is_redeclare {
            let redeclare_warn =
                Hazard::new_one_loc(HazardType::Warn(WarnId::RedeclareVar), span.0, span.1);
            println!("{}", redeclare_warn.show_output());
        }
    }

    /// tells you if a ident is in valid scope and has been initialized. This
    /// should be run before we use a variable in an expr
    pub fn has_been_initialized(&self, ident: String, span: (usize, usize)) {
        if self
            .symbols_in_valid_scope()
            .iter()
            .filter(|s| s.initialized.get())
            .any(|s| s.ident == ident)
        {
            let h = Hazard::new_one_loc(HazardType::Warn(WarnId::Uninit), span.0, span.1);
            println!("{}", h.show_output());
        }
    }

    pub fn symbols_in_valid_scope(&self) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| self.valid_scopes.contains(&s.scope))
            .collect()
    }

    pub fn symbols_in_scope(&self, scope: usize) -> Vec<&Symbol> {
        self.symbols.iter().filter(|s| s.scope == scope).collect() // FISHER should we change this to 'in or below scope and make it a <?
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
    pub const_: bool,
    pub ty: String, // ty = type. Should probably not be a string, but don't have types yet
    pub ident: String, // identifier
    pub span: (usize, usize),
    pub used: Cell<bool>,
    pub initialized: Cell<bool>,
}

impl Symbol {
    pub fn new(
        scope: usize,
        ty: String,
        ident: String,
        span: (usize, usize),
        const_: bool,
    ) -> Self {
        Self {
            scope,
            const_,
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

    pub fn program(&mut self, program: &AstNode) {}

    /// expected_type is what the expr should evaluate too. Returns a type
    pub fn get_expr_type(&mut self, expr: &AstNode) -> Result<String, Hazard> {
        // a < (b + c)

        if expr.children.is_empty() {
            match expr.kind {
                AstKind::Integer => return Ok("int".to_string()),
                AstKind::Float => return Ok("float".to_string()),
                AstKind::String => return Ok("string".to_string()),
                _ => panic!("invalid kind for bottom node in expr tree"),
            }
        }

        if expr.children.len() == 1 {
            match expr.data.as_str() {
                "+" | "-" => {
                    let child_kind = &self.get_expr_type(&expr[0])?;
                    if is_numeric(child_kind) {
                        return Ok(child_kind.clone());
                    } else {
                        return Err(Hazard::new_one_loc(
                            HazardType::ErrorT(ErrorId::Expr),
                            expr.span.0,
                            expr.span.1,
                        ));
                    }
                }
                "~" => {
                    // bitwise complement
                    let child_kind = &self.get_expr_type(&expr[0])?;
                    if child_kind == "int" {
                        return Ok("bool".to_string());
                    } else {
                        return Err(Hazard::new_one_loc(
                            HazardType::ErrorT(ErrorId::Expr),
                            expr.span.0,
                            expr.span.1,
                        ));
                    }
                }
                "!" => {
                    let child_kind = &self.get_expr_type(&expr[0])?;
                    if child_kind == "bool" {
                        return Ok("bool".to_string());
                    } else {
                        return Err(Hazard::new_one_loc(
                            HazardType::ErrorT(ErrorId::Expr),
                            expr.span.0,
                            expr.span.1,
                        ));
                    }
                }
                _ => panic!("Invalid string of expr_data when dealing with one child in expr"),
            }
        }

        // TODO if Expr has no children then we return what?
        let lhs = self.get_expr_type(&expr[0])?;
        let rhs = self.get_expr_type(&expr[1])?;
        // assert_eq!(AstKind::Expr, expr.kind);
        let op = expr.data.as_str();
        match expr.kind {
            AstKind::Plus => {
                if lhs == rhs && (lhs == "float" || lhs == "int") {
                    Ok(lhs)
                } else if (lhs == "float" || lhs == "int") && (rhs == "float" || rhs == "int") {
                    Ok("float".to_string())
                } else {
                    Err(Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::Expr),
                        expr.span.0,
                        expr.span.1,
                    ))
                }
            }
            AstKind::Bools => {
                let op = expr.data.as_str();
                if is_numeric(&lhs) && is_numeric(&rhs) {
                    Ok("bool".to_string())
                } else if lhs == rhs && (op == "==" || op == "!=") {
                    Ok("bool".to_string())
                } else {
                    Err(Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::Expr),
                        expr.span.0,
                        expr.span.1,
                    ))
                }
            }
            AstKind::Times => {
                if op == "mod" {
                    if lhs == rhs && lhs == "int" {
                        Ok(lhs)
                    } else {
                        Err(Hazard::new_one_loc(
                            HazardType::ErrorT(ErrorId::Expr),
                            expr.span.0,
                            expr.span.1,
                        ))
                    }
                } else if lhs == rhs && (lhs == "float" || lhs == "int") {
                    Ok(lhs)
                } else if (lhs == "float" || lhs == "int") && (rhs == "float" || rhs == "int") {
                    Ok("float".to_string())
                } else {
                    Err(Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::Expr),
                        expr.span.0,
                        expr.span.1,
                    ))
                }
            }
            _ => panic!("Bad astkind where there should be expr"),
        }
    }

    // fn compare_result(lhs: &str, rhs: &str) -> String {

    //     if lhs == "float" && rhs == "int" {
    //         return "float".to_string()
    //     }

    // }

    fn stmt(&mut self, stmt: &AstNode) {
        assert_eq!(AstKind::Statement, stmt.kind);

        match stmt[0].kind {
            AstKind::DecList => {
                self.decl_list(&stmt[0]);
            }
            AstKind::Assign => {}
            AstKind::Emit => {}
            AstKind::If => {}
            AstKind::IfElse => {}
            AstKind::While => {}
            AstKind::BraceStmt => {}
            _ => panic!("Unsupported Stmt Child"),
        }
    }

    fn decl_list(&mut self, stmt: &AstNode) {
        // At this point the lhs child is a type and
        // some identifier
        // The rhs is either an id or ASSIGN
        // Get the type of the LHS, check for CONV with lhs == rhs

        let lhs = &stmt.children[0];

        // We have nice flattened tree:
        let comma_list = &stmt.children[1].children;

        let ty = lhs;

        for comma in comma_list {
            self.handle_comma(ty, comma);
        }
    }

    fn handle_comma(&mut self, ty: &AstNode, comma: &AstNode) {
        // Either an identifier or an assign list
        let child = &comma[0];

        let (is_const, string_ty) = get_decl_type(ty);

        // There is a single identifier
        match child.children.as_slice() {
            [ident] => self.table.push_symbol(
                self.scope,
                string_ty,
                ident.data.clone(),
                ident.span,
                is_const,
            ),
            [ids @ .., expr] => {
                for ident in ids {
                    self.table.push_symbol_init(
                        self.scope,
                        string_ty.clone(),
                        ident.data.clone(),
                        ident.span,
                        is_const,
                    );

                    // TODO: Determine expr type and check it
                    // with string_ty
                }
            }
            [] => panic!("There must be at least one child"),
        }
    }
}

pub fn is_numeric(ty: &str) -> bool {
    ty == "float" || ty == "int"
}

pub fn get_decl_type(ty: &AstNode) -> (bool, String) {
    let is_const = ty.children.len() > 1;
    let string_ty = ty[1].data.clone();

    (is_const, string_ty)
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
