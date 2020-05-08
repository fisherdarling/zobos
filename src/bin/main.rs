#![allow(non_snake_case)]

use std::path::PathBuf;
use structopt::StructOpt;
use zobos::parser::Parser;
use zobos::symbol_table::*;

#[derive(Debug, Clone, StructOpt)]
pub struct Args {
    pub token_input: PathBuf,
    pub ast_output: PathBuf,
    pub table_output: PathBuf,
}

fn main() {
    let args = Args::from_args();

    let mut dot_out = args.token_input.clone();
    dot_out.set_extension("dot");

    let mut parser = Parser::new();

    let tree = parser.parse(args.token_input).unwrap();
    let ast = tree.create_ast();
    std::fs::create_dir("test");
    ast.export_graph(PathBuf::from("test").join(dot_out.file_name().unwrap()));

    let mut sv = SymbolVisitor::new();
    sv.program(&ast);

    sv.report_unused();
    //println!("{:#?}", tree);
}
