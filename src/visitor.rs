// use crate::ast::{AstKind, AstNode};

// pub trait Visitor
// where
//     Self: Sized,
//     Self::Output: Default,
// {
//     type Output;

//     fn visit_program(&mut self, program: &AstNode) -> Self::Output {
//         assert_eq!(AstKind::Program, program.kind);

//         walk_program(self, program)
//     }

//     fn visit_stmt(&mut self, stmt: &AstNode) -> Self::Output {
//         assert_eq!(AstKind::Statement, stmt.kind);

//         walk_stmt(self, stmt)
//     }

//     fn visit_emit(&mut self, emit: &AstNode) -> Self::Output {
//         // walk_id(emit[0]);
//         // walk_int(emit[1]);
//         // walk_int(emit[2]);

//         Self::Output::default()
//     }

//     fn visit_decl_list(&mut self, decl_list: &AstNode) -> Self::Output {}

//     // fn visit_assign(&mut self, assign: &AstNode) -> Self::Output {}
//     // fn visit_if(&mut self, if_: &AstNode) -> Self::Output {}
//     // fn visit_if_else(&mut self, if_else: &AstNode) -> Self::Output {}
//     // fn visit_while(&mut self, while_: &AstNode) -> Self::Output {}
//     // fn visit_brace_stmt(&mut self, brace_stmt: &AstNode) -> Self::Output {}

//     // fn visit_stmts(&mut self, stmts: &AstNode) {
//     //     assert_eq!(AstKind::Stmts, stmts.kind);

//     //     if !stmts.children.is_empty() {
//     //         self.visit_stmts(&stmts[0]);
//     //         self.visit_stmt(&stmts[1]);
//     //     }
//     // }

//     // fn visit_stmt(&mut self, stmt: &AstNode) {
//     //     assert_eq!(AstKind::Statement, stmt.kind);

//     //     match stmt[0].kind {
//     //         AstKind::DecList => {
//     //             self.visit_decl_list(&stmt[0]);
//     //         }
//     //         AstKind::Assign => {}
//     //         AstKind::Emit => {}
//     //         AstKind::If => {}
//     //         AstKind::IfElse => {}
//     //         AstKind::While => {}
//     //         AstKind::BraceStmt => {}
//     //         _ => panic!("Unsupported Stmt Child"),
//     //     }
//     // }

//     // // DeclList
//     // fn visit_decl_list(&mut self, decl_list: &AstNode) {
//     //     self.visit_decl(&decl_list[0], &decl_list[1]);
//     // }

//     // // DeclType DeclIds
//     // fn visit_decl(&mut self, decl_type: &AstNode, decl_ids: &AstNode) {
//     //     // ...
//     // }

//     // fn visit_eoi(&mut self, node: &AstNode) {}
// }

// pub fn walk_program<V: Visitor>(visitor: &mut V, program: &AstNode) -> V::Output {
//     for i in 0..program.children.len() - 1 {
//         visitor.visit_stmt(&program[i]);
//     }

//     visitor.visit_stmt(&program[program.children.len() - 1])
// }

// pub fn walk_stmt<V: Visitor>(visitor: &mut V, stmt: &AstNode) -> V::Output {
//     let child = &stmt[0];

//     match child.kind {
//         AstKind::Emit => visitor.visit_emit(child),
//         AstKind::DecList => visitor.visit_decl_list(child),
//         AstKind::Assign => visitor.visit_assign(child),
//         AstKind::If => visitor.visit_if(child),
//         AstKind::IfElse => visitor.visit_if_else(child),
//         AstKind::While => visitor.visit_while(child),
//         AstKind::BraceStmt => visitor.visit_brace_stmt(child),
//         _ => panic!("Unsupported Statement Child"),
//     }
// }

// // match child.kind {
// //     AstKind::Emit => walk_emit(self, child),
// //     AstKind::DecList => walk_decl_list(self, child),
// //     AstKind::Assign => walk_assign(self, child),
// //     AstKind::If => walk_if(self, child),
// //     AstKind::IfElse => walk_if_else(self, child),
// //     AstKind::While => walk_while(self, child),
// //     AstKind::BraceStmt => walk_brace_stmt(self, child),
// // }

// // pub trait Visitor
// // where
// //     Self: Sized,
// // {
// //     fn visit_program(&mut self, program: &AstNode) {
// //         assert_eq!(AstKind::Program, program.kind);

// //         self.visit_stmts(&program[0]);
// //         self.visit_eoi(&program[1]);
// //     }

// //     fn visit_stmts(&mut self, stmts: &AstNode) {
// //         assert_eq!(AstKind::Stmts, stmts.kind);

// //         if !stmts.children.is_empty() {
// //             self.visit_stmts(&stmts[0]);
// //             self.visit_stmt(&stmts[1]);
// //         }
// //     }

// //     fn visit_stmt(&mut self, stmt: &AstNode) {
// //         assert_eq!(AstKind::Statement, stmt.kind);

// //         match stmt[0].kind {
// //             AstKind::DecList => {
// //                 self.visit_decl_list(&stmt[0]);
// //             }
// //             AstKind::Assign => {}
// //             AstKind::Emit => {}
// //             AstKind::If => {}
// //             AstKind::IfElse => {}
// //             AstKind::While => {}
// //             AstKind::BraceStmt => {}
// //             _ => panic!("Unsupported Stmt Child"),
// //         }
// //     }

// //     // DeclList
// //     fn visit_decl_list(&mut self, decl_list: &AstNode) {
// //         self.visit_decl(&decl_list[0], &decl_list[1]);
// //     }

// //     // DeclType DeclIds
// //     fn visit_decl(&mut self, decl_type: &AstNode, decl_ids: &AstNode) {
// //         // ...
// //     }

// //     fn visit_eoi(&mut self, node: &AstNode) {}
// // }

// // pub struct DeclCounter;

// // impl Visitor<usize> for DeclCounter {
// //     fn visit_decl_list(&mut self, decl_list: &AstNode) {
// //         Visitor::visit_decl(self, &decl_list[0], &decl_list[1]);
// //     }
// // }

// // #[derive(Debug, Copy, Clone, PartialEq)]
// // pub enum AstKind {
// //     Token,
// //     EOI,
// //     Stmts,
// //     Statement,
// //     BraceStmt,
// //     DecList,
// //     Assign,
// //     If,
// //     IfElse,
// //     While,
// //     Emit,
// //     DeclType,
// //     DeclId,
// //     DeclIds,
// //     Expr,
// //     BooleanExpr,
// //     ArithmeticExpr,
// //     Bools,
// //     Plus,
// //     Times,
// //     Sum,
// //     Product,
// //     Value,
// //     Unary,
// //     Cast,
// //     Program,
// //     String,
// //     Identifier,
// //     TypeString,
// // }
