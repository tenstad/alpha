use clap::Parser;
use eval::Eval;
use parser::AlphaParser;
use std::fs;
use trans::Translator;

mod ast;
mod eval;
mod parser;
mod trans;

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
            Translator::translate(&ast);
        }
        Err(e) => println!("{}", e),
    }
}
