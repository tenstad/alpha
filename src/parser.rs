use crate::ast;

use pest::{iterators::Pairs, pratt_parser::PrattParser, Parser};
use pest_derive::Parser;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc, Op};

        PrattParser::new()
            .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
            .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
            .op(Op::infix(Rule::pow, Assoc::Right))
            .op(Op::postfix(Rule::fac))
            .op(Op::prefix(Rule::neg))
            .op(Op::prefix(Rule::name))
            .op(Op::prefix(Rule::def))
    };
}

#[derive(Parser)]
#[grammar = "alpha.pest"]
pub struct AlphaParser;

impl AlphaParser {
    pub fn parse_source(source: &str) -> Result<Vec<ast::Node>, String> {
        println!("----- Source -----\n{}\n------------------", source);

        let mut pairs = AlphaParser::parse(Rule::program, source).map_err(|e| e.to_string())?;
        println!("----- Pairs ------\n{:?}\n------------------", pairs);

        let ast = pairs
            .next()
            .unwrap()
            .into_inner()
            .next()
            .unwrap()
            .into_inner()
            .map(|pair| Self::parse_expr(pair.into_inner()))
            .collect::<Result<Vec<ast::Node>, String>>();

        if let Ok(parsed) = &ast {
            println!("----- Parsed -----");
            for a in parsed.iter() {
                println!("{:?}", a);
            }
            println!("------------------");
        }

        ast
    }

    fn parse_expr(pairs: Pairs<Rule>) -> Result<ast::Node, String> {
        PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::program | Rule::expr => Self::parse_expr(primary.into_inner()),
                Rule::list => {
                    let list = primary
                        .into_inner()
                        .map(|i| Self::parse_expr(i.into_inner()))
                        .collect::<Result<Vec<ast::Node>, String>>()?;
                    Ok(ast::Node::List(list))
                }
                Rule::int => primary
                    .as_str()
                    .parse::<u64>()
                    .map_err(|err| err.to_string())
                    .map(ast::Node::Int),
                Rule::name => Ok(ast::Node::Var(
                    primary.as_str().to_string(),
                    Box::new(ast::Node::Int(0)),
                )),
                Rule::varref => Ok(ast::Node::VarRef(primary.as_str().to_string())),
                _ => {
                    dbg!(primary);
                    unreachable!()
                }
            })
            .map_prefix(|op, rhs| match op.as_rule() {
                Rule::neg => Ok(ast::Node::Expr {
                    op: ast::Op::Sub,
                    lhs: Box::new(ast::Node::Int(0)),
                    rhs: Box::new(rhs?),
                }),
                Rule::name => Ok(ast::Node::Var(op.as_str().to_string(), Box::new(rhs?))),
                Rule::def => Ok(rhs?),
                _ => {
                    dbg!(op, rhs?);
                    unreachable!()
                }
            })
            .map_postfix(|lhs, op| match op.as_rule() {
                Rule::fac => todo!(),
                _ => {
                    dbg!(lhs?, op);
                    unreachable!()
                }
            })
            .map_infix(|lhs, op, rhs| {
                Ok(ast::Node::Expr {
                    op: match op.as_rule() {
                        Rule::add => ast::Op::Add,
                        Rule::sub => ast::Op::Sub,
                        Rule::mul => ast::Op::Mul,
                        Rule::div => ast::Op::Div,
                        Rule::pow => todo!(),
                        _ => unreachable!(),
                    },
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                })
            })
            .parse(pairs)
    }
}
