use clap::Parser;
use eval::Eval;
use parser::AlphaParser;
use std::fs;

mod ast;
mod comp;
mod eval;
mod parser;

#[macro_use]
extern crate lazy_static;

#[derive(clap::Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short = 'f', long)]
    file: String,
    #[arg(short = 'd', long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    let program = fs::read_to_string(args.file).unwrap();

    match AlphaParser::parse_source(program.as_str(), args.debug) {
        Ok(ast) => {
            Eval::default().run(&ast);

            let start = ast::Node::FnDef(Some("main".into()), Vec::new(), Box::new(ast));
            let mut compiler = comp::Compiler::new(args.debug);
            compiler.declare_functions(&start);
            if let ast::Node::FnDef(_, _, node) = start {
                compiler.translate_fn(&Some("main".into()), &Vec::new(), &node, args.debug);
            }
            compiler.compile(args.debug);
        }
        Err(e) => println!("{}", e),
    }
}
