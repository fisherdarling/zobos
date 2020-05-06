use crate::ast::{AstKind, AstNode};

pub trait Visitor
where
    Self: Sized,
{
    fn visit_program(&mut self, program: &AstNode) {
        assert_eq!(AstKind::Program, program.kind);

        self.visit_stmts(&program[0]);
        self.visit_eoi(&program[1]);
    }

    fn visit_stmts(&mut self, stmts: &AstNode) {
        assert_eq!(AstKind::Stmts, stmts.kind);

        if !stmts.children.is_empty() {
            self.visit_stmts(&stmts[0]);
            self.visit_stmt(&stmts[1]);
        }
    }

    fn visit_stmt(&mut self, stmt: &AstNode) {
        assert_eq!(AstKind::Statement, stmt.kind);

        match stmt[0].kind {
            AstKind::DecList => {
                self.visit_decl_list(&stmt[0]);
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

    fn visit_decl_list(&mut self, decl_list: &AstNode) {
        self.visit_decl(&decl_list[0], &decl_list[1]);
    }

    fn visit_decl(&mut self, decl_type: &AstNode, decl_ids: &AstNode) {
        // ...
    }

    fn visit_eoi(&mut self, node: &AstNode) {}
}

pub struct DeclCounter(usize);

impl Visitor for DeclCounter {
    fn visit_decl_list(&mut self, decl_list: &AstNode) {
        self.0 += 1;
        Visitor::visit_decl(self, &decl_list[0], &decl_list[1]);
    }
}

// #[derive(Debug, Copy, Clone, PartialEq)]
// pub enum AstKind {
//     Token,
//     EOI,
//     Stmts,
//     Statement,
//     BraceStmt,
//     DecList,
//     Assign,
//     If,
//     IfElse,
//     While,
//     Emit,
//     DeclType,
//     DeclId,
//     DeclIds,
//     Expr,
//     BooleanExpr,
//     ArithmeticExpr,
//     Bools,
//     Plus,
//     Times,
//     Sum,
//     Product,
//     Value,
//     Unary,
//     Cast,
//     Program,
//     String,
//     Identifier,
//     TypeString,
// }
