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
    Symtable,
    Product,
    Value,
    Unary,
    Cast,
    Eq,
    Program,
    String,
    Identifier,
    Integer,
    Float,
    TypeInt,
    TypeString,
    TypeFloat,
}

impl fmt::Display for AstKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // AstKind::Float(fl) => write!(f, "{}", fl),
            // AstKind::Int(i) => write!(f, "{}", i),
            // AstKind::Bool(b) => write!(f, "{}", b),
            AstKind::Eq => write!(f, "="),
            AstKind::Plus => write!(f, "+"),
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
        for child in node.children.iter() {
            if child.data.len() != 0 {
                let cnode = graph.add_node(child.data.clone());
                graph.add_edge(parent, cnode, 0);
                graph = self.create_pet_graph_rec(graph, child, cnode);
                continue;
            }
            let cnode = graph.add_node(child.kind.to_string());
            graph.add_edge(parent, cnode, 0);

            graph = self.create_pet_graph_rec(graph, child, cnode);
        }
        graph
    }

    pub fn create_ast(&self) -> AstNode {
        assert_eq!(AstKind::Program, self.kind);
        let mut new_node = AstNode::new(AstKind::Program);

        new_node.children.append(&mut self.simplify_stmts(&self[0]));
        new_node.children.push(self.simplify_eoi(&self[1]));
        new_node
    }

    fn simplify_stmts(&self, stmts: &AstNode) -> Vec<AstNode> {
        assert_eq!(AstKind::Stmts, stmts.kind);

        if stmts.children.is_empty() {
            return vec![];
        }
        let mut list = self.simplify_stmts(&stmts.children[0]);
        list.push(self.simplify_stmt(&stmts.children[1]));
        list
    }

    fn simplify_stmt(&self, stmt: &AstNode) -> AstNode {
        assert_eq!(AstKind::Statement, stmt.kind);
        let mut new_node = AstNode::new(AstKind::Statement);

        match stmt[0].kind {
            AstKind::DecList => new_node.children.push(self.simplify_decl_list(&stmt[0])),
            AstKind::Assign => new_node.children.push(self.simplify_assign(&stmt[0])),
            AstKind::Emit => new_node.children.push(self.simplify_emit(&stmt[0])),
            AstKind::If => new_node.children.push(self.simplify_if(&stmt[0])),
            AstKind::IfElse => new_node.children.push(self.simplify_ifelse(&stmt[0])),
            AstKind::While => new_node.children.push(self.simplify_while(&stmt[0])),
            AstKind::BraceStmt => new_node.children.push(self.simplify_brace(&stmt[0])),
            _ => panic!("Unsupported Stmt Child"),
        }
        new_node
    }

    fn simplify_decl_list(&self, decl_list: &AstNode) -> AstNode {
        assert_eq!(AstKind::DecList, decl_list.kind);
        let atype = self.simplify_decl_type(&decl_list.children[0]);
        let mut ids = self.simplify_dec_ids(&decl_list.children[1]); // vec<id>
        let mut new_decl_list = AstNode::new(AstKind::DecList);
        new_decl_list.children.push(atype);
        new_decl_list.children.append(&mut ids);
        new_decl_list
    }

    fn simplify_dec_ids(&self, decl_ids: &AstNode) -> Vec<AstNode> {
        assert_eq!(AstKind::DeclIds, decl_ids.kind);
        if decl_ids.children.len() == 1 {
            // DECLIDS -> DECLID
            let id = self.simplify_dec_id(&decl_ids.children[0]);
            let ids = vec![id];
            return ids;
        } else {
            // DECLIDS -> DECLIDS comma DECLID
            let mut ids = self.simplify_dec_ids(&decl_ids.children[0]);
            let id = self.simplify_dec_id(&decl_ids.children[2]);
            ids.push(id);
            return ids;
        }
    }

    fn simplify_dec_id(&self, decl_id: &AstNode) -> AstNode {
        assert_eq!(AstKind::DeclId, decl_id.kind);
        let mut new_id = AstNode::new(AstKind::DeclId);
        let child = &decl_id.children[0];

        if child.kind == AstKind::Identifier {
            // DECLID -> id
            let ident = child.clone();
            new_id.children.push(ident);
            return new_id;
        } else {
            // DECLID -> ASSIGN
            new_id.children.push(self.simplify_assign(child));
        }
        new_id
    }

    fn simplify_decl_type(&self, atype: &AstNode) -> AstNode {
        if atype.children.len() > 1 {
            let mut new_node = AstNode::new(AstKind::DeclType);
            new_node.data = atype[0].data.to_owned();
            new_node.data.push_str(" ");
            new_node.data.push_str(&atype[1].data.to_owned());
            return new_node;
        }
        atype.children[0].clone()
    }

    fn simplify_assign(&self, assign: &AstNode) -> AstNode {
        // TODO add fisher opt
        let mut retval = AstNode::new(AstKind::Eq);
        retval.span = assign[1].span.clone();
        if assign.children[2].kind == AstKind::Expr {
            // ASSIGN -> id assign EXPR
            retval.children.push(assign.children[0].clone());
            retval
                .children
                .push(self.simplify_expr(&assign.children[2]));
        } else {
            //ASSIGN -> id assign Assign
            retval.children.push(assign.children[0].clone());
            retval
                .children
                .append(&mut self.simplify_assign_rec(&assign.children[2]));
        }
        retval
    }

    fn simplify_assign_rec(&self, assign: &AstNode) -> Vec<AstNode> {
        // TODO add fisher opt
        let mut retval: Vec<AstNode> = Vec::new();
        if assign.children[2].kind == AstKind::Expr {
            // ASSIGN -> id assign EXPR
            retval.push(assign.children[0].clone());
            retval.push(self.simplify_expr(&assign.children[2]));
        } else {
            //ASSIGN -> id assign Assign
            retval.push(assign.children[0].clone());
            retval.append(&mut self.simplify_assign_rec(&assign.children[2]));
        }
        retval
    }

    fn simplify_expr(&self, expr: &AstNode) -> AstNode {
        let child = &expr.children[0];
        match child.kind {
            AstKind::ArithmeticExpr => self.simplify_aexpr(child),
            AstKind::BooleanExpr => self.simplify_bexpr(child),
            _ => panic!("Bad Expr"),
        }
    }

    fn simplify_aexpr(&self, expr: &AstNode) -> AstNode {
        self.simplify_sum(&expr.children[0])
    }

    fn simplify_sum(&self, sum: &AstNode) -> AstNode {
        if sum.children.len() == 1 {
            //SUM -> PRODUCT
            return self.simplify_prod(&sum.children[0]);
        } else {
            //SUM -> SUM PLUS PRODUCT
            let mut plus = AstNode::new(AstKind::Plus);
            plus.children.push(self.simplify_sum(&sum.children[0]));
            plus.children.push(self.simplify_prod(&sum.children[2]));
            return plus;
        }
    }

    fn simplify_prod(&self, prod: &AstNode) -> AstNode {
        if prod.children.len() == 1 {
            // PRODUCT -> VAlUE
            return self.simplify_value(&prod.children[0]);
        } else {
            // PRODUCT -> PRODUCT TIMES VALUE
            let mut times = AstNode::new(AstKind::Times);
            times.children.push(self.simplify_prod(&prod.children[0]));
            times.children.push(self.simplify_value(&prod.children[2]));
            return times;
        }
    }

    fn simplify_plus(&self, plus: &AstNode) -> AstNode {
        plus.children[0].clone()
    }

    fn simplify_value(&self, value: &AstNode) -> AstNode {
        assert_eq!(AstKind::Value, value.kind);
        let left_child = &value.children[0];
        match left_child.kind {
            AstKind::Unary => return self.simplify_unary(left_child),
            AstKind::Cast => return self.simplify_cast(left_child),
            _ => {}
        };

        if value.children.len() == 1 {
            return left_child.clone();
        }

        let middle_child = &value.children[1];
        match middle_child.kind {
            AstKind::ArithmeticExpr => self.simplify_aexpr(middle_child),
            AstKind::BooleanExpr => self.simplify_bexpr(middle_child),
            _ => panic!("Bad Value node"),
        }
    }

    fn simplify_unary(&self, unary: &AstNode) -> AstNode {
        let mut new_node;
        let child = &unary.children[0];
        match child.kind {
            AstKind::Plus => new_node = self.simplify_plus(child),
            _ => new_node = child.clone(),
        };
        new_node
            .children
            .push(self.simplify_value(&unary.children[1]));
        new_node
    }

    fn simplify_cast(&self, cast: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::Cast);
        new_node.children.push(cast.children[0].clone());
        new_node
            .children
            .push(self.simplify_aexpr(&cast.children[1]));
        new_node
    }

    fn simplify_bexpr(&self, expr: &AstNode) -> AstNode {
        let mut bools = self.simplify_bool(&expr[1]);
        bools.children.push(self.simplify_aexpr(&expr.children[0]));
        bools.children.push(self.simplify_aexpr(&expr.children[2]));
        bools
    }

    fn simplify_bool(&self, abool: &AstNode) -> AstNode {
        abool[0].clone()
    }

    fn simplify_emit(&self, emit: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::Emit);
        if emit.children.len() == 4 {
            // EMIT -> emit id AEXPR AEXPR
            new_node.children.push(emit[1].clone());
            new_node
                .children
                .push(self.simplify_aexpr(&emit.children[2]));
            new_node
                .children
                .push(self.simplify_aexpr(&emit.children[3]));
        } else {
            // EMIT -> emit symtable
            new_node.children.push(AstNode::new(AstKind::Symtable));
        }
        new_node
    }

    fn simplify_if(&self, aif: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::If);
        new_node
            .children
            .push(self.simplify_bexpr(&aif.children[2]));
        new_node.children.push(self.simplify_stmt(&aif.children[4]));
        new_node
    }

    fn simplify_ifelse(&self, ifelse: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::IfElse);
        new_node
            .children
            .push(self.simplify_bexpr(&ifelse.children[2]));
        new_node
            .children
            .push(self.simplify_brace(&ifelse.children[4]));
        new_node
            .children
            .push(self.simplify_stmt(&ifelse.children[6]));
        new_node
    }

    fn simplify_while(&self, awhile: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::While);
        new_node
            .children
            .push(self.simplify_bexpr(&awhile.children[2]));
        new_node
            .children
            .push(self.simplify_stmt(&awhile.children[4]));
        new_node
    }

    fn simplify_brace(&self, brace: &AstNode) -> AstNode {
        let mut new_node = AstNode::new(AstKind::BraceStmt);
        new_node
            .children
            .append(&mut self.simplify_stmts(&brace.children[1]));
        new_node
    }

    fn simplify_eoi(&self, node: &AstNode) -> AstNode {
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
