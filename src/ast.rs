use petgraph::dot::{Config, Dot};
use petgraph::Graph;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstKind {
    Program,
    Stmt,
    BraceStmt,
    DecList,
    Assign,
    DeclIdents,
    If,
    IfElse,
    While,
    Ident,
    Expr,
    BooleanExpr,
    ArithmeticExpr,
    Plus,
    Times,
    Not, // !
    Lt,  // <
    Leq, // <=
    Eq,  // ==
    Geq, // >=
    Gt,  // >
    Literal,
    Cast,
    Emit,
    Float(f64),
    Int(i64),
    Bool(bool),
}

impl fmt::Display for AstKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AstKind::Float(fl) => write!(f, "{}", fl),
            AstKind::int(i) => write!(f, "{}", i),
            AstKind::Bool(b) => write!(f, "{}", b),
            AstKind::Eq => write!(f, "=="),
            AstKind::Leq => write!(f, "<="),
            AstKind::Geq => write!(f, ">="),
            AstKind::Gt => write!(f, ">"),
            AstKind::Lt => write!(f, "<"),
            _ => write!(f, "{:?}", self),
        }
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGraph {
    pub graph: Graph<AstKind, usize>,
}

impl AstGraph {
    //This writes out a .dot file to a path
    pub fn export_graph(&self, file_path: impl AsRef<Path>) {
        let mut f = File::create(file_path).unwrap();
        let output = format!("{}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]));
        f.write_all(&output.as_bytes())
            .expect("could not write file");
    }
}
