pub mod items;
pub mod production;
pub mod symbol;
pub mod token;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use self::production::Production;
use self::symbol::{NonTerminal, Symbol, Terminal};
use self::token::{Token, TokenStream};
use crate::ast::{AstKind, AstNode};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Shift(usize),
    Reduce(usize),
    ReduceTerminate(usize),
}

impl Action {
    fn parse(input: &str) -> Self {
        let tokens: Vec<&str> = input.split('-').collect();

        match tokens.as_slice() {
            &["r", n] => Action::Reduce(n.parse::<usize>().unwrap() - 1), // -1 because rules are 1 indexed
            &["R", n] => Action::ReduceTerminate(n.parse::<usize>().unwrap() - 1),
            &["sh", n] => Action::Shift(n.parse().unwrap()),
            _ => panic!(format!("Unsupported Action: {:?}", tokens)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseState {
    pub state: usize,
    pub tree: Option<ParseInput>,
}

impl ParseState {
    pub fn start() -> Self {
        Self {
            state: 0,
            tree: None,
        }
    }

    pub fn tree(&self) -> Option<AstNode> {
        self.tree.as_ref().map(|t| t.clone().node())
    }
}

impl ParseState {
    pub fn new(state: usize, tree: ParseInput) -> Self {
        Self {
            state,
            tree: Some(tree),
        }
    }

    fn state(&self) -> usize {
        self.state
    }
}

#[derive(Debug, Clone)]
pub enum ParseInput {
    Tree(Symbol, AstNode),
    Token(Symbol, Token),
    EOI,
}

impl ParseInput {
    pub fn node(self) -> AstNode {
        match self {
            ParseInput::Tree(s, n) => n,
            ParseInput::Token(s, t) => ast_node_from_token(&t),
            ParseInput::EOI => AstNode::new(AstKind::EOI),
        }
    }

    pub fn symbol(&self) -> Symbol {
        match self {
            ParseInput::Tree(s, _) => s.clone(),
            ParseInput::Token(s, _) => s.clone(),
            ParseInput::EOI => Symbol::from_parse("$").unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    pub items: Vec<(NonTerminal, Production)>,
    pub table: Vec<BTreeMap<Symbol, Action>>,
    // pub tokens: TokenStream,
}

impl Parser {
    pub fn new() -> Self {
        // let tokens = TokenStream::from_file(token_src_path);
        let items = parse_items();
        let table = parse_table();

        Self {
            items,
            table,
            // tokens,
        }
    }

    pub fn parse(&mut self, token_src_path: impl AsRef<std::path::Path>) -> Option<AstNode> {
        let mut stack: Vec<ParseState> = Vec::new();
        stack.push(ParseState::start());

        let mut tokens: Vec<ParseInput> = TokenStream::from_file(token_src_path)
            .into_iter()
            .map(|t| ParseInput::Token(Symbol::from_parse(&t.id).unwrap(), t))
            .collect();

        tokens.reverse();

        while let Some(token) = tokens.last().cloned().or_else(|| Some(ParseInput::EOI)) {
            println!("Stack: {:#?}", stack);

            // let t = Symbol::from_parse(&token.id).unwrap();

            if let Some(top_state) = stack.last().map(|s| s.state) {
                let action = self.table[top_state].get(&token.symbol()).expect(&format!(
                    "Syntax Error Here: State: [{}] Token: {:?}",
                    top_state, token
                ));

                println!(
                    "State: {}, Token: {:?}, Action: {:?}\n",
                    top_state, token, action
                );

                match *action {
                    Action::Shift(to_state) => {
                        // Advance the token state:
                        // match tokens.remove
                        tokens.pop();
                        stack.push(ParseState::new(to_state, token));
                    }
                    Action::Reduce(rule) => {
                        // stack.push(ParseState::Token(0, token));
                        self.reduce(rule, top_state, &mut stack, &mut tokens)
                    }
                    Action::ReduceTerminate(rule) => {
                        // stack.push(ParseState::Token(0, token));
                        self.reduce(rule, top_state, &mut stack, &mut tokens);

                        return tokens.last().cloned().map(|t| t.node());
                    }
                }
            } else {
                panic!("Stack is empty");
            }
        }

        None
    }

    fn reduce(
        &mut self,
        rule: usize,
        state: usize,
        stack: &mut Vec<ParseState>,
        input: &mut Vec<ParseInput>,
    ) {
        println!("Reduce({}): Stack: {:?}", rule, stack);

        let (non_terminal, production) = &self.items[rule];

        println!("{:?} -> {:?}\n", non_terminal, production);

        let ast_kind = ast_kind_from_str(non_terminal.non_terminal());
        let mut node = AstNode::new(ast_kind);

        if production.only_lambda() {
            input.push(ParseInput::Tree(
                Symbol::from_non_terminal(non_terminal.clone()),
                AstNode::new(ast_kind_from_str(non_terminal.non_terminal())),
            ));
            // stack.push(ParseState::new(state, node));
            return;
        }

        node.children.extend(
            stack[stack.len() - production.len()..]
                .iter()
                .map(|s| s.tree().unwrap()),
        );

        stack.truncate(stack.len() - production.len());
        input.push(ParseInput::Tree(
            Symbol::from_non_terminal(non_terminal.clone()),
            node,
        ));
    }
}

fn parse_items() -> Vec<(NonTerminal, Production)> {
    let file = File::open("zlang-rules.lis").unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().flatten().filter(|l| !l.is_empty());

    let mut items = Vec::new();

    for line in lines {
        let mut split = line.split_whitespace();

        let idx: usize = split
            .next()
            .unwrap()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .parse()
            .unwrap();

        let non_terminal = NonTerminal::new(split.next().unwrap());

        let _arrow = split.next().unwrap();

        let production: Vec<Symbol> = split.map(|s| Symbol::from_parse(s).unwrap()).collect();

        assert_eq!(items.len(), idx - 1);

        items.push((non_terminal, Production(production)));
    }

    items
}

fn parse_table() -> Vec<BTreeMap<Symbol, Action>> {
    let file = File::open("zlang.lr").unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines().flatten().filter(|l| !l.is_empty());

    let first_line = lines.next().unwrap();

    let associated_symbols: Vec<Symbol> = first_line
        .split(',')
        .skip(1)
        .map(|s| Symbol::from_parse(s).unwrap())
        .collect();

    let mut rows = Vec::new();

    for line in lines {
        let mut split = line.split(',');
        let state: usize = split.next().unwrap().parse().unwrap();

        let row: BTreeMap<Symbol, Action> = split
            .zip(associated_symbols.iter())
            .flat_map(|(value, symbol)| {
                if !value.is_empty() {
                    Some((symbol.clone(), Action::parse(value)))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(rows.len(), state);

        rows.push(row);
    }

    rows
}

fn ast_kind_from_str(symbol: &str) -> AstKind {
    match symbol {
        "assign" | "emit" | "bool" | "comma" | "compl" | "const" | "div" | "else" | "eq"
        | "float" | "floatval" | "if" | "int" | "intval" | "lbrace" | "lparen" | "minus"
        | "mod" | "mult" | "not" | "plus" | "rbrace" | "rparen" | "sc" | "symtable" | "while"
        | "lt" | "leq" | "eq" | "geq" | "gt" => AstKind::Token,
        "string" => AstKind::TypeString,
        "$" => AstKind::EOI,
        "id" => AstKind::Identifier,
        "stringval" => AstKind::String,
        "STMTS" => AstKind::Stmts,
        "PROGRAM" => AstKind::Program,
        "STATEMENT" => AstKind::Statement,
        "BRACESTMTS" => AstKind::BraceStmt,
        "DECLLIST" => AstKind::DecList,
        "ASSIGN" => AstKind::Assign,
        "IF" => AstKind::If,
        "IFELSE" => AstKind::IfElse,
        "WHILE" => AstKind::While,
        "EMIT" => AstKind::Emit,
        "DECLTYPE" => AstKind::DeclType,
        "DECLID" => AstKind::DeclId,
        "DECLIDS" => AstKind::DeclIds,
        "EXPR" => AstKind::Expr,
        "BEXPR" => AstKind::BooleanExpr,
        "AEXPR" => AstKind::ArithmeticExpr,
        "BOOLS" => AstKind::Bools,
        "PLUS" => AstKind::Plus,
        "TIMES" => AstKind::Times,
        "SUM" => AstKind::Sum,
        "PRODUCT" => AstKind::Product,
        "VALUE" => AstKind::Value,
        "UNARY" => AstKind::Unary,
        "CAST" => AstKind::Cast,
        e => panic!(format!("Unsupported Symbol: {:?}", e)),
    }
}

fn ast_node_from_token(token: &Token) -> AstNode {
    let kind = ast_kind_from_str(&token.id);

    let mut chars = token.data.chars();

    let mut data = String::with_capacity(token.data.len());
    while let Some(c) = decode_char(&mut chars) {
        data.push(c);
    }

    AstNode {
        kind,
        data: data,
        span: token.span,
        children: Vec::new(),
    }
}

// fn get_char(data: &str) -> char {
//     decode_char(&mut data.chars()).unwrap()
// }

fn decode_char(chars: &mut dyn Iterator<Item = char>) -> Option<char> {
    Some(match chars.next()? {
        'x' => {
            let first = chars.next()?;
            let second = chars.next()?;
            let mut first = first.to_string();
            first.push(second);

            let value: u32 = u32::from_str_radix(&first, 16).unwrap();
            let char = std::char::from_u32(value).unwrap();

            char
        }
        c => c,
    })
}
