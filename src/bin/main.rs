#![allow(non_snake_case)]

use zobos::parser::Parser;

fn main() {
    let mut parser = Parser::new();

    let tree = parser.parse("res/helloworld.tok").unwrap();
    println!("{:#?}", tree);
}
