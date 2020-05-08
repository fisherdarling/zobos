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

    let tree = parser
        .parse(args.token_input)
        .unwrap_or_else(|| std::process::exit(1));
    let ast = tree.create_ast();
    ast.export_graph(&args.ast_output);

    let mut sv = SymbolVisitor::new(args.table_output);
    sv.program(&ast);

    sv.report_unused();

    if sv.errored {
        std::process::exit(1);
    }

    // TODO do check to see if 'emit symtable' is anywhere in table
    // sv.write_table_to_file(&args.table_output);

    // output symbol table at end

    //println!("{:#?}", tree);
}
