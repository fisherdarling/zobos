use crate::ast::{AstKind, AstNode};
use crate::hazards::{ErrorId, Hazard, HazardType, Location, WarnId};
use std::cell::Cell;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

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
        if !self.check_for_redeclare(&ident, scope, span) {
            let symbol = Symbol::new(scope, ty, ident, span, const_);
            self.symbols.push(symbol);
        }
    }

    pub fn push_symbol_init(
        &mut self,
        scope: usize,
        ty: String,
        ident: String,
        span: (usize, usize),
        const_: bool,
    ) {
        if !self.check_for_redeclare(&ident, scope, span) {
            let symbol = Symbol::new(scope, ty, ident, span, const_);
            symbol.initialized.set(true);
            self.symbols.push(symbol);
        }
    }

    /// will emit an error if is redeclare, and not add it to the scope
    pub fn check_for_redeclare(&self, ident: &str, scope: usize, span: (usize, usize)) -> bool {
        let current_valid_symbols = self.symbols_in_valid_scope(scope);
        let is_redeclare = current_valid_symbols
            .iter()
            .any(|s| s.ident == ident && s.scope == scope);
        if is_redeclare {
            let redeclare_warn =
                Hazard::new_one_loc(HazardType::Warn(WarnId::RedeclareVar), span.0, span.1);
            println!("{}", redeclare_warn.show_output());
        }
        println!(
            "chkcing if ident: {}, scope: {} is redeclare: {}",
            ident, scope, is_redeclare
        );
        // gitprintln!("is it? ")
        is_redeclare
    }

    /// tells you if a ident is in valid scope and has been initialized. This
    /// should be run before we use a variable in an expr
    pub fn has_been_initialized(&self, ident: String, current_scope: usize, span: (usize, usize)) {
        if self
            .symbols_in_valid_scope(current_scope)
            .iter()
            .filter(|s| s.initialized.get())
            .any(|s| s.ident == ident)
        {
            let h = Hazard::new_one_loc(HazardType::Warn(WarnId::Uninit), span.0, span.1);
            println!("{}", h.show_output());
        }
    }

    pub fn symbols_in_valid_scope(&self, current_scope: usize) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .rev()
            .filter(|s| s.scope <= current_scope)
            .collect()
    }

    // pub fn symbols_in_scope(&self, scope: usize) -> Vec<&Symbol> {
    //     self.symbols.iter().filter(|s| s.scope == scope).collect() // FISHER should we change this to 'in or below scope and make it a <?
    // }

    pub fn get_symbol(&self, ident: &str, scope: usize) -> Option<&Symbol> {
        self.symbols_in_valid_scope(scope)
            .iter()
            .find(|s| s.ident == ident)
            .cloned()
    }

    pub fn symbols_in_scope_mut(&mut self, scope: usize) -> Vec<&mut Symbol> {
        self.symbols
            .iter_mut()
            .filter(|s| s.scope == scope)
            .collect()
    }

    pub fn clean_table(&mut self, scope: usize) {
        self.symbols.retain(|s| s.scope < scope);
    }

    pub fn write_to_file(&self, path: &PathBuf) {
        let out = self.output();
        let mut file = File::with_options()
            .write(true)
            .append(true)
            .create(true)
            .open(path)
            .unwrap_or_else(|e| std::process::exit(1));

        file.write_all(out.as_bytes()).unwrap();
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

#[derive(Debug, Default, Clone)]
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
        if self.const_ {
            out.push_str("const");
        }
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
    pub errored: bool,
    output_path: std::path::PathBuf,
}

impl SymbolVisitor {
    pub fn new(output_path: std::path::PathBuf) -> Self {
        let mut new = Self::default();
        new.output_path = output_path;
        new
    }

    pub fn write_table_to_file(&self, path: &PathBuf) {
        self.table.write_to_file(path);
    }

    pub fn current_symbols(&self) -> Vec<&Symbol> {
        self.table.symbols_in_valid_scope(self.scope)
    }

    pub fn exists(&self, ident: &str) -> bool {
        (0..=self.scope)
            .map(|s| self.table.symbols_in_valid_scope(s).into_iter())
            .flatten()
            .any(|s| s.ident == ident)
    }

    pub fn initialized(&self, ident: &str) -> bool {
        for symbols in (0..=self.scope)
            .rev()
            .map(|s| self.table.symbols_in_valid_scope(s))
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
            .map(|s| self.table.symbols_in_valid_scope(s))
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

    pub fn program(&mut self, program: &AstNode) {
        // println!("Program: {}", program.kind.to_string());
        for c in &program.children {
            if c.kind == AstKind::Statement {
                self.stmt(c);
            }
        }
    }

    /// expected_type is what the expr should evaluate too. Returns a type
    pub fn get_expr_type(&mut self, expr: &AstNode) -> Result<String, Vec<Hazard>> {
        // println!("Expr: {}", expr.kind.to_string());
        // println!(
        //     "Valid Scope: {:?}\n",
        //     self.table.symbols_in_valid_scope(self.scope)
        // );

        // a < (b + c)

        if expr.children.is_empty() {
            match expr.kind {
                AstKind::Integer => return Ok("int".to_string()),
                AstKind::Float => return Ok("float".to_string()),
                AstKind::String => return Ok("string".to_string()),
                AstKind::Identifier => {
                    if let Some(ident) = self
                        .table
                        .symbols_in_valid_scope(self.scope)
                        .iter()
                        .find(|s| s.ident == expr.data)
                    {
                        ident.used.set(true);

                        if ident.initialized.get() {
                            return Ok(ident.ty.clone());
                        } else {
                            let warn = Hazard::new_one_loc(
                                HazardType::Warn(WarnId::Uninit),
                                expr.span.0,
                                expr.span.1,
                            );

                            println!("{}", warn.show_output());

                            return Ok(ident.ty.clone());
                        }
                    } else {
                        // Identifier doesn't even exist:
                        let h = Hazard::new_one_loc(
                            HazardType::ErrorT(ErrorId::NoVar),
                            expr.span.0,
                            expr.span.1,
                        );
                        self.errored = true;
                        return Err(vec![h]);
                    }
                }
                e => panic!(format!(
                    "invalid kind for bottom node in expr tree: {:?}",
                    e
                )),
            }
        }

        if expr.children.len() == 1 {
            match expr.data.as_str() {
                "+" | "-" => {
                    let child_kind = &self.get_expr_type(&expr[0])?;
                    if is_numeric(child_kind) {
                        return Ok(child_kind.clone());
                    } else {
                        self.errored = true;
                        return Err(vec![Hazard::new_one_loc(
                            HazardType::ErrorT(ErrorId::Expr),
                            expr.span.0,
                            expr.span.1,
                        )]);
                    }
                }
                "~" => {
                    // bitwise complement
                    let child_kind = &self.get_expr_type(&expr[0])?;
                    if child_kind == "int" {
                        return Ok("bool".to_string());
                    } else {
                        self.errored = true;
                        return Err(vec![Hazard::new_one_loc(
                            HazardType::ErrorT(ErrorId::Expr),
                            expr.span.0,
                            expr.span.1,
                        )]);
                    }
                }
                "!" => {
                    let child_kind = &self.get_expr_type(&expr[0])?;
                    if child_kind == "bool" {
                        return Ok("bool".to_string());
                    } else {
                        self.errored = true;
                        return Err(vec![Hazard::new_one_loc(
                            HazardType::ErrorT(ErrorId::Expr),
                            expr.span.0,
                            expr.span.1,
                        )]);
                    }
                }
                _ => panic!("Invalid string of expr_data when dealing with one child in expr"),
            }
        }

        let mut errs = Vec::new();

        if let AstKind::Cast = expr.kind {
            let cast_type = expr[0].data.clone();
            let rhs = self.get_expr_type(&expr[1]);

            match rhs {
                Ok(_) => return Ok(cast_type),
                Err(e) => return Err(e),
            }
        }

        // TODO if Expr has no children then we return what?
        let lhs = self.get_expr_type(&expr[0]).map_err(|e| errs.extend(e));
        let rhs = self.get_expr_type(&expr[1]).map_err(|e| errs.extend(e));
        if !errs.is_empty() {
            return Err(errs);
        }
        let lhs = lhs.unwrap();
        let rhs = rhs.unwrap();
        // assert_eq!(AstKind::Expr, expr.kind);
        let op = expr.data.as_str();
        println!("Op: {}, Kind: {}", op, expr.kind);
        match expr.kind {
            AstKind::Plus => {
                if lhs == rhs && (lhs == "float" || lhs == "int") {
                    Ok(lhs)
                } else if (lhs == "float" || lhs == "int") && (rhs == "float" || rhs == "int") {
                    Ok("float".to_string())
                } else {
                    self.errored = true;
                    Err(vec![Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::Expr),
                        expr.span.0,
                        expr.span.1,
                    )])
                }
            }
            AstKind::Bools | AstKind::Eq => {
                let op = expr.data.as_str();
                if is_numeric(&lhs) && is_numeric(&rhs) {
                    Ok("bool".to_string())
                } else if lhs == rhs
                    // || (lhs.contains("bool") && rhs.contains("int")
                        // || rhs.contains("bool") && lhs.contains("int"))
                        && (op == "==" || op == "!=")
                {
                    Ok("bool".to_string())
                } else {
                    self.errored = true;
                    Err(vec![Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::Expr),
                        expr.span.0,
                        expr.span.1,
                    )])
                }
            }
            AstKind::Times => {
                if op == "%" {
                    if lhs == rhs && lhs == "int" {
                        Ok(lhs)
                    } else {
                        self.errored = true;
                        Err(vec![Hazard::new_one_loc(
                            HazardType::ErrorT(ErrorId::Expr),
                            expr.span.0,
                            expr.span.1,
                        )])
                    }
                } else if lhs == rhs && (lhs == "float" || lhs == "int") {
                    Ok(lhs)
                } else if (lhs == "float" || lhs == "int") && (rhs == "float" || rhs == "int") {
                    Ok("float".to_string())
                } else {
                    self.errored = true;
                    Err(vec![Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::Expr),
                        expr.span.0,
                        expr.span.1,
                    )])
                }
            }
            e => panic!(format!(
                "Bad astkind where there should be expr: {:?}",
                expr
            )),
        }
    }

    // fn compare_result(lhs: &str, rhs: &str) -> String {

    //     if lhs == "float" && rhs == "int" {
    //         return "float".to_string()
    //     }

    // }

    fn handle_emit(&mut self, emit: &AstNode) {
        // This is a specific identifier:
        if emit.children.len() == 3 {
            for child in &emit.children {
                if let Err(e) = self.get_expr_type(child) {
                    e.iter().for_each(|h| println!("{}", h.show_output()));
                }
            }
        // let ident = &emit[0];

        // let symbol = self.table.get_symbol(&ident.data, self.scope);

        // if let Some(symbol) = symbol {
        //     symbol.used.set(true);
        // } else {
        //     // ERROR: NOVAR
        // }
        } else if emit.children.len() == 1 {
            self.table.write_to_file(&self.output_path);
        }
    }

    fn stmt(&mut self, stmt: &AstNode) {
        // println!("Stmt: {}", stmt.kind.to_string());

        assert_eq!(AstKind::Statement, stmt.kind);

        match stmt[0].kind {
            AstKind::DecList => {
                self.decl_list(&stmt[0]);
            }
            AstKind::Assign | AstKind::Eq => {
                self.assign_stmt(&stmt[0]);
            }
            AstKind::Emit => {
                self.handle_emit(&stmt[0]);
            }
            AstKind::If => {
                self.if_stmt_stmt(&stmt[0]);
            }
            AstKind::IfElse => {
                self.if_else_stmt(&stmt[0]);
            }
            AstKind::While => {
                self.while_stmt(&stmt[0]);
            }
            AstKind::BraceStmt => {
                self.brace_stmt(&stmt[0]);
            }
            s => panic!(format!(
                "Unsupported Stmt Child: {}: {:?}",
                s.to_string(),
                stmt[0]
            )),
        }
    }

    // Pushing and popping scopes and stuff:
    fn brace_stmt(&mut self, brace: &AstNode) {
        self.scope += 1;

        for child in &brace.children {
            self.stmt(child); // call stmt on all of the brace child children
        }

        self.table.clean_table(self.scope);
        self.scope -= 1;
        // TODO: Clean table of all symbols with scope = self.scope + 1
    }

    fn assign_stmt(&mut self, assign: &AstNode) {
        let rhs_type = self.get_expr_type(&assign.children.last().unwrap()).clone();
        // Get the identifier and its type

        for child in 0..assign.children.len() - 1 {
            let ident = &assign[child].data;
            let symbol = self.table.get_symbol(ident, self.scope);

            match symbol {
                Some(symbol) => {
                    let lhs_ty = &symbol.ty;

                    if symbol.const_ {
                        let h = Hazard::new_one_loc(
                            HazardType::Warn(WarnId::Const),
                            assign[child].span.0,
                            assign[child].span.1,
                        );

                        println!("{}", h.show_output());
                    }

                    symbol.used.set(true);

                    if let Ok(ref rhs_ty) = rhs_type {
                        if !is_valid_conversion(&lhs_ty, rhs_ty) {
                            let h = Hazard::new_one_loc(
                                HazardType::ErrorT(ErrorId::Conversion),
                                assign.span.0,
                                assign.span.1,
                            );

                            symbol.initialized.set(true);

                            println!("{}", h.show_output());
                            self.errored = true;
                        } else {
                            if !symbol.const_ {
                                symbol.initialized.set(true);
                            }
                        }
                    }
                }
                None => {
                    let h = Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::NoVar),
                        assign[child].span.0,
                        assign[child].span.1,
                    );

                    self.errored = true;
                    println!("{}", h.show_output());
                }
            }
        }

        if let Err(ref e) = rhs_type {
            e.iter().for_each(|h| println!("{}", h.show_output()));
        }
    }

    fn if_stmt_stmt(&mut self, if_: &AstNode) {
        let predicate = &if_[0];

        let predicate_type = self.get_expr_type(predicate);

        if let Err(ref e) = predicate_type {}

        match predicate_type {
            Err(e) => {
                self.errored = true;
                e.iter().for_each(|h| println!("{}", h.show_output()));
            }
            Ok(p) => {
                if !p.contains("bool") {
                    let h = Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::Conversion),
                        predicate.span.0,
                        predicate.span.1,
                    );

                    println!("{}", h.show_output());
                    self.errored = true;
                }
            }
        }

        if if_[1][0].kind == AstKind::BraceStmt {
            let brace_stmt = &if_[1][0];
            self.brace_stmt(brace_stmt);
        } else {
            let stmt = &if_[1];
            self.stmt(stmt);
        }
    }

    fn if_else_stmt(&mut self, if_else: &AstNode) {
        assert_eq!(if_else.children.len(), 3); // first is predicate, second is true, third is false
        let predicate = &if_else[0];

        let predicate_type = self.get_expr_type(predicate);

        if let Err(ref e) = predicate_type {}

        match predicate_type {
            Err(e) => {
                self.errored = true;
                e.iter().for_each(|h| println!("{}", h.show_output()));
            }
            Ok(p) => {
                if !p.contains("bool") {
                    let h = Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::Conversion),
                        predicate.span.0,
                        predicate.span.1,
                    );

                    println!("{}", h.show_output());
                    self.errored = true;
                }
            }
        }

        if if_else[1].kind == AstKind::BraceStmt {
            let brace_stmt = &if_else[1];
            self.brace_stmt(brace_stmt);
        } else {
            let stmt = &if_else[1];
            self.stmt(stmt);
        }
        // probably have something like above but for now I am going to assume #2 is something like
        // Statement -> BraceStmt -> Statement
        if if_else[2][0].kind == AstKind::BraceStmt {
            let false_brace = &if_else[2][0];
            self.brace_stmt(&false_brace);
        } else if if_else[2][0].kind == AstKind::IfElse {
            self.if_else_stmt(&if_else[2][0]);
        }

        // let true_brace_stmt = &if_else[1][0];
        // let false_brace_stmt = &if_else[2][0];

        // self.brace_stmt(false_brace_stmt);
    }

    fn while_stmt(&mut self, while_: &AstNode) {
        // println!("While: {}", while_.kind.to_string());

        // println!("{:#?}", self.table);

        let predicate = &while_[0];

        let predicate_type = self.get_expr_type(predicate);

        if let Err(ref e) = predicate_type {}

        match predicate_type {
            Err(e) => {
                self.errored = true;
                e.iter().for_each(|h| println!("{}", h.show_output()));
            }
            Ok(p) => {
                if !p.contains("bool") {
                    let h = Hazard::new_one_loc(
                        HazardType::ErrorT(ErrorId::Conversion),
                        predicate.span.0,
                        predicate.span.1,
                    );

                    println!("{}", h.show_output());
                    self.errored = true;
                }
            }
        }

        let brace_stmt = &while_[1][0];

        self.brace_stmt(brace_stmt);
    }

    fn decl_list(&mut self, stmt: &AstNode) {
        // println!("DeclList: {}", stmt.kind.to_string());
        // At this point the lhs child is a type and
        // some identifier
        // The rhs is either an id or ASSIGN
        // Get the type of the LHS, check for CONV with lhs == rhs

        let lhs = &stmt.children[0];

        // We have nice flattened tree:
        // let comma_list = &stmt[1].children;

        let ty = lhs;

        for decl_list_child in stmt.children.iter().skip(1) {
            for comma in &decl_list_child.children {
                // println!("Comma: {:?}", comma);
                if let Err(h) = self.handle_comma(ty, comma) {
                    h.iter().for_each(|h| println!("{}", h.show_output()));
                }
                // println!("{}\n", self.table.output());
            }
        }
    }
    fn handle_comma(&mut self, ty: &AstNode, comma: &AstNode) -> Result<(), Vec<Hazard>> {
        // println!("DeclId: {}", comma.kind.to_string());
        // println!("{:?}", comma);
        // Either an identifier or an assign list
        // let child = &comma[0];        // Either an identifier or an assign list
        // let child = &comma[0];

        // println!("{}\n", self.table.output());

        let (is_const, string_ty) = get_decl_type(ty);

        // There is a single identifier
        match comma.children.as_slice() {
            // It is a single identifier
            [] => self.table.push_symbol(
                self.scope,
                string_ty,
                comma.data.clone(),
                comma.span,
                is_const,
            ),
            // [ident] => self.table.push_symbol(
            //     self.scope,
            //     string_ty,
            //     ident.data.clone(),
            //     ident.span,
            //     is_const,
            // ),
            [ids @ .., expr] => {
                let mut errors = Vec::new();
                let expr_ty = self.get_expr_type(expr);
                for ident in ids {
                    self.table.push_symbol_init(
                        self.scope,
                        string_ty.clone(),
                        ident.data.clone(),
                        ident.span,
                        is_const,
                    );

                    if let Ok(ref expr_ty) = expr_ty {
                        if !is_valid_conversion(&string_ty, &expr_ty) {
                            // println!("NOT VALID: {} <- {}", string_ty, expr_ty);
                            let h = Hazard::new_one_loc(
                                HazardType::ErrorT(ErrorId::Conversion),
                                comma.span.0, // TODO have to point this to assign node? Liam needs to change assignment node to keep span
                                comma.span.1,
                            );
                            self.errored = true;
                            errors.push(h);
                        }
                    }

                    // TODO: Determine expr type and check it
                    // with string_ty
                }
                if let Err(e) = expr_ty {
                    errors.extend(e.into_iter());
                }

                if !errors.is_empty() {
                    return Err(errors);
                }
            }
            [] => panic!("There must be at least one child"),
        }

        // println!("After Comma: {:?}", self.table);

        Ok(())
    }
}

pub fn is_numeric(ty: &str) -> bool {
    ty == "float" || ty == "int"
}

pub fn get_decl_type(ty: &AstNode) -> (bool, String) {
    let is_const = ty.data.contains("const");
    let string_ty = ty.data.split(' ').rev().next().unwrap().clone().to_string();

    (is_const, string_ty)
}

pub fn is_valid_conversion(var_type: &str, val_type: &str) -> bool {
    match val_type {
        "string" => match var_type {
            "int" => false,
            "float" => false,
            "bool" => false,
            _ => true,
        },
        "float" => match var_type {
            "int" => false,
            "bool" => false,
            "string" => false,
            _ => true,
        },
        "bool" => match var_type {
            "float" => false,
            "string" => false,
            _ => true,
        },
        "int" => match var_type {
            // Check table with keith
            "bool" => false,
            "string" => false,
            _ => true,
        },
        _ => panic!("unknown lhs passed into valid conversion"),
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
