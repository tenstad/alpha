use clap::Parser;
use eval::Eval;
use parser::AlphaParser;
use std::{fs, process::Command};

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
    #[arg(short = 'i', long)]
    interpret: bool,
    #[arg(short = 'r', long)]
    run: bool,
    #[arg(short = 'd', long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    let program = fs::read_to_string(args.file).unwrap();

    match AlphaParser::parse_source(program.as_str(), args.debug) {
        Ok(ast) => {
            if args.interpret {
                Eval::default().run(&ast);
            } else {
                let start = ast::Node::FnDef(Some("main".into()), Vec::new(), Box::new(ast));
                let mut compiler = comp::Compiler::new(args.debug);
                compiler.declare_functions(&start);
                if let ast::Node::FnDef(_, _, node) = start {
                    compiler.translate_fn(&Some("main".into()), &Vec::new(), &node, args.debug);
                }
                compiler.compile();
                if args.run {
                    Command::new("./build/out").status().unwrap();
                }
            }
        }
        Err(e) => println!("{}", e),
    }
}
