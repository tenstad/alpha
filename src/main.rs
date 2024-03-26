use eval::Eval;
use parser::AlphaParser;

mod ast;
mod eval;
mod parser;

#[macro_use]
extern crate lazy_static;

fn main() {
    let program = "
    2 + 3;
    4 - (1 + 2);
    let foo = 5;
    let mut bar = -(10 - 2);
    let baz = bar;

    foo;
    bar;
    baz;
    ";
    match AlphaParser::parse_source(program) {
        Ok(ast) =>  Eval::default().run(&ast),
        Err(e) => println!("{}", e),
    }
}