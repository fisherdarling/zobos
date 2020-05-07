use petgraph::dot::{Config, Dot};
use petgraph::Graph;
use std::cmp::PartialEq;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::ops::Index;
use std::path::Path;

// #[derive(Debug, Clone, PartialEq)]
// pub enum AstKind {
//     Program,
//     Stmt,
//     BraceStmt,
//     DecList,
//     Assign,
//     DeclIdents,
//     If,
//     IfElse,
//     While,
//     Ident,
//     Expr,
//     BooleanExpr,
//     ArithmeticExpr,
//     Plus,
//     Times,
//     Not, // !
//     Lt,  // <
//     Leq, // <=
//     Eq,  // ==
//     Geq, // >=
//     Gt,  // >
//     Literal,
//     Cast,
//     Emit,
//     Float(f64),
//     Int(i64),
//     Bool(bool),
// }

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AstKind {
    Token,
    EOI,
    Stmts,
    Statement,
    BraceStmt,
    DecList,
    Assign,
    If,
    IfElse,
    While,
    Emit,
    DeclType,
    DeclId,
    DeclIds,
    Expr,
    BooleanExpr,
    ArithmeticExpr,
    Bools,
    Plus,
    Times,
    Sum,
    Symtable
    Product,
    Value,
    Unary,
    Cast,
    Program,
    String,
    Identifier,
    TypeString,
}

impl fmt::Display for AstKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // AstKind::Float(fl) => write!(f, "{}", fl),
            // AstKind::Int(i) => write!(f, "{}", i),
            // AstKind::Bool(b) => write!(f, "{}", b),
            // AstKind::Eq => write!(f, "=="),
            // AstKind::Leq => write!(f, "<="),
            // AstKind::Geq => write!(f, ">="),
            // AstKind::Gt => write!(f, ">"),
            // AstKind::Lt => write!(f, "<"),
            _ => write!(f, "{:?}", self),
        }
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstNode {
    pub kind: AstKind,
    pub data: String,
    pub span: (usize, usize),
    pub children: Vec<AstNode>,
}

impl Index<usize> for AstNode {
    type Output = AstNode;

    fn index(&self, index: usize) -> &Self::Output {
        &self.children[index]
    }
}

impl AstNode {
    pub fn new(kind: AstKind) -> Self {
        Self {
            kind,
            span: (0, 0),
            data: String::new(),
            children: Vec::new(),
        }
    }

    // Export a graph to something that Graphvis can us
    pub fn export_graph(&self, file_path: impl AsRef<Path>) {
        let graph = self.create_pet_graph();
        let mut f = File::create(file_path).unwrap();
        let output = format!("{}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
        f.write_all(&output.as_bytes())
            .expect("could not write file");
    }

    fn create_pet_graph(&self) -> Graph<String, usize> {
        let mut graph = Graph::<String, usize>::new();
        let root = graph.add_node(self.kind.to_string());

        for child in self.children.iter() {
            let cnode = graph.add_node(child.kind.to_string());
            graph.add_edge(root, cnode, 0);
            graph = self.create_pet_graph_rec(graph, child, cnode);
        }
        graph
    }

    fn create_pet_graph_rec(
        &self,
        mut graph: Graph<String, usize>,
        node: &AstNode,
        parent: petgraph::graph::NodeIndex,
    ) -> Graph<String, usize> {
        if node.data.len() != 0 {
            println!("Hi!");
            let cnode = graph.add_node(node.data.clone());
            graph.add_edge(parent, cnode, 0);
        }
        for child in node.children.iter() {
            let cnode = graph.add_node(child.kind.to_string());
            graph.add_edge(parent, cnode, 0);

            graph = self.create_pet_graph_rec(graph, child, cnode);
        }
        graph
    }

    pub fn simplify_program(program: &AstNode) -> AstNode {
        assert_eq!(AstKind::Program, program.kind);
        let mut new_node = AstNode::new(Astkind::Program);

        new_node.children.append(simplify_stmts(&program[0]));
        new_node.children.push(simplify_eoi(&program[1]));
    }

    fn simplify_stmts(stmts: &AstNode) -> vec<AstNode> {
        assert_eq!(AstKind::Stmts, stmts.kind);

        if !stmts.children.is_empty() {
            return vec![]
        }
        let mut list = simplify_stmts(stmts.children[0]);
        list.push(simplify_stmt(stmts.children[1])) 
        list
    }

    fn simplify_stmt(stmt: &AstNode) -> AstNode {
        assert_eq!(AstKind::Statement, stmt.kind);
        let mut new_node = stmt.clone(); 

        match stmt[0].kind {
            AstKind::DecList => new_node.push(simplify_decl_list(&stmt[0])),
            AstKind::Assign => new_node.push(simplify_assign(&stmt[0])),
            AstKind::Emit => new_node.push(simplify_emit(&stmt[0])),
            AstKind::If => new_node.push(simplify_if(&stmt[0])),
            AstKind::IfElse => new_node.push(simplify_ifelse(&stmt[0])),
            AstKind::While => new_node.push(simplify_while(&stmt[0])),
            AstKind::BraceStmt => new_node.push(simplify_brace(&stmt[0])),
            _ => panic!("Unsupported Stmt Child"),
        }
    }

    fn simplify_decl_list(decl_list: &AstNode) -> AstNode {
        let type = simplify_decl_type(decl_list.children[0]);
        let mut ids = simplify_decl_ids(decl_list.children[1]); // vec<id>
        let new_decl_list = AstNode::new(AstKind::DecList);
        new_decl_list.children.push(type);
        new_decl_list.children.append(ids);
        new_decl_list
    }

    fn simplify_dec_ids(decl_ids: &AstNode) -> vec<AstNode> {
        if decl_ids.children.len() == 1 {  // DECLIDS   -> DECLID
            let id = simplify_dec_ids(decl_ids.children[0]);
            let ids vec![id];
            return ids;
        } else {  // DECLIDS   -> DECLIDS comma DECLID
            let mut ids = simplify_dec_ids(decl_ids.chlidren[0]);
            let id = simplify_dec_ids(decl_ids.children[2]);
            ids.push(id);
            return ids;
        }
    }

    fn simplify_dec_id(decl_id: &AstNode) -> AstNode {
        let mut new_id = AstNode::new(AstKind::DeclId);
        let child = decl_id.children[0];

        if child.type == AstKind::Identifier {  // DECLID -> id
            let ident = child.clone();
            new_id.children.push(ident);
            return new_id;
        } else {  // DECLID -> ASSIGN
            new_id.children.push(simplify_assign(child));
        }
        new_id
    }

    fn simplify_assign(assign: &AstNode) -> AstNode { // TODO add fisher opt
        let mut retval = AstNode::new(AstKind::Eq);
        if assign.children[2].kind == AstKind::Expr {  // ASSIGN -> id assign EXPR
            retval.children.push(assign.children[0].clone());
            retval.children.push(simplify_expr(assign.children[2]));
        } else {  //ASSIGN -> id assign Assign
            retval.children.push(assign.children[0].clone());
            retval.children.push(simplify_assign(assign.children[2]));
        }
        retval
    }

    fn simplify_expr(expr: &AstNode) -> AstNode {
        let child = expr.children[0]
        match child.kind {
            AstKind::ArithmeticExpr -> simplify_aexpr(child),
            AstKind::BooleanExpr -> simplify_bexpr(child),
            _ -> panic!("Bad Expr")
        }
    }

    fn simplify_aexpr(expr: &AstNode) -> AstNode {
        simplify_sum(expr.children[0])
    }

    fn simplify_sum(sum: &AstNode) -> AstNode {
        if sum.children.len() == 1 { //SUM -> PRODUCT
            return simplify_prod(sum.children[0]);
        } else { //SUM -> SUM PLUS PRODUCT
            let mut plus = AstNode::new(AstKind::Plus);
            plus.children.push(simplify_sum(sum.children[0]));
            plus.children.push(simplify_prod(sum.children[3]));
            return plus;
        }
    }
    
    fn simplify_prod(prod: &AstNode) -> AstNode {
        if prod.children.len() == 1 {  // PRODUCT -> VAlUE
            return simplify_value(prod.children[0])
        } else {  // PRODUCT -> PRODUCT TIMES VALUE
            let mut times = AstNode(AstKind::Times);
            times.children.push(simplify_prod(prod.children[0]));
            times.children.push(simplify_value(prod.children[3]));
            return times;
        }
    }

    fn simplify_value(value: &AstNode) -> AstNode {
        let left_child = value.children[0];
        match left_child.kind {
            AstKind::Unary => return simplify_unary(left_child),
            Astkind::Cast => return simplify_cast(left_child),
            _ => {}
        };

        if value.children.len() == 1 {
            return left_child.clone();
        }

        let middle_child = value.children[1];
        match middle_child {
            Astkind::ArithmeticExpr => simplify_aexpr(middle_child),
            Astkind::BooleanExpr => simplify_bexpr(middle_child),
        }
    }

    fn simplify_unary(unary: &AstNode) -> AstNode {
        let mut new_node;
        let child = unary.children[0];
        match child.kind {
            AstKind::Plus => new_node = simplify_plus(child),
            _ => new_node = child.clone(),
        };
        new_node.children.push(simplify_value(unary.children[1]));
        new_node
    }

    fn simplify_cast(cast: &AstNode) -> AstNode {
        new_node = AstNode::new(Astkind::Cast);
        new_node.children.push(cast.children[0].clone());
        new_node.children.push(simplify_aexpr(cast.children[1]));
        new_node
    }

    fn simplify_bexpr(expr: &AstNode) -> AstNode {
        let mut bools:AstNode = expr.children[1].clone();
        bools.children.push(simplify_aexpr(expr.children[0]));
        bools.children.push(simplify_aexpr(expr.children[2]));
        bools
    }

    fn simplify_emit(emit: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::Emit);
        if emit.children.len() == 4 {  // EMIT -> emit id AEXPR AEXPR
            new_node.children.push(AstNode::new(AstKind::Identifier));
            new_node.children.push(simplify_aexpr(emit.children[2]));
            new_node.children.push(simplify_aexpr(emit.children[3]));
        } else {  // EMIT -> emit symtable
            new_node.children.push(AstNode::new(AstKind::Symtable));
        }
        new_node
    }

    fn simplify_if(aif: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::If);
        new_node.children.push(simplify_bexpr(aif.children[2]));
        new_node.children.push(simplify_stmt(aif.children[4]));
        new_node
    }

    fn simplify_ifelse(ifelse: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::IfElse);
        new_node.children.push(simplify_bexpr(ifelse.children[2]));
        new_node.children.push(simplify_brace(ifelse.children[4]));
        new_node.children.push(simplify_stmt(ifelse.children[6]));
        new_node
    }

    fn simplify_while(awhile: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::While);
        new_node.children.push(simplify_bexpr(awhile.children[2]));
        new_node.children.push(simplify_stmt(awhile.children[4]));
        new_node
    }

    fn simplify_brace(brace: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::BraceStmt);
        new_node.children.append(simplify_stmts(brace.children[1]));
        new_node
    }

    fn simplify_eoi(node: &AstNode) -> AstNode {
        AstNode::new(AstKind::EOI)
    }
}

#[derive(Debug, Clone)]
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
