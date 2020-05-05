#![allow(non_snake_case)]

use zobos::parser::Parser;

fn main() {
    let mut parser = Parser::new();

    let tree = parser.parse("res/helloworld.tok").unwrap();
    tree.export_graph("test/hello.dot");
    println!("{:#?}", tree);
}
