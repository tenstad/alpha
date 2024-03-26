use eval::Eval;
use parser::AlphaParser;

mod ast;
mod eval;
mod parser;

#[macro_use]
extern crate lazy_static;

fn main() {
    let program = "
    -(1 + 2) / -2 + 1 * 2 + [2 + 3, 2];
    2;
    ";
    let ast = AlphaParser::parse_source(program).unwrap();
    Eval::run(&ast);
}
