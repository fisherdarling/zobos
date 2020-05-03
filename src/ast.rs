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
    Lt, // <
    Leq, // <=
    Eq, // ==
    Geq, // >=
    Gt, // >
    Literal,
    Cast,
    Emit,
    Float(f64),
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstNode {
    pub kind: AstKind,
    pub children: Vec<AstNode>,
}