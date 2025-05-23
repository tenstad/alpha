use crate::ast;

use pest::{
    iterators::{Pair, Pairs},
    pratt_parser::PrattParser,
    Parser,
};
use pest_derive::Parser;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc, Op};

        PrattParser::new()
            .op(Op::postfix(Rule::EOI))
            .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
            .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
            .op(Op::infix(Rule::pow, Assoc::Right))
            .op(Op::postfix(Rule::fac))
            .op(Op::prefix(Rule::neg))
            .op(Op::infix(Rule::gt, Assoc::Right)
                | Op::infix(Rule::ge, Assoc::Right)
                | Op::infix(Rule::lt, Assoc::Right)
                | Op::infix(Rule::le, Assoc::Right))
            .op(Op::infix(Rule::eq, Assoc::Right) | Op::infix(Rule::neq, Assoc::Right))
    };
}

#[derive(Parser)]
#[grammar = "alpha.pest"]
pub struct AlphaParser;

impl AlphaParser {
    fn print_pairs(pairs: &Pairs<'_, Rule>, depth: usize) {
        for p in pairs.clone().into_iter() {
            println!(
                "{: >3} {: >3} {} {: <12} {:?}",
                p.as_span().start(),
                p.as_span().end(),
                " ".repeat(2 * depth),
                format!("{:?}", p.as_rule()),
                p.as_str()
            );
            AlphaParser::print_pairs(&p.into_inner(), depth + 1);
        }
    }

    pub fn parse_source(source: &str, debug: bool) -> Result<ast::Node, String> {
        if debug {
            println!("----- Source -----\n{}\n------------------", source);
        }

        let pairs = AlphaParser::parse(Rule::program, source).map_err(|e| e.to_string())?;
        if debug {
            println!("----- Pairs ------");
            AlphaParser::print_pairs(&pairs, 0);
            println!("------------------");
        }

        let ast = Self::parse_pairs(pairs);

        if debug {
            if let Ok(parsed) = &ast {
                println!("----- Parsed -----");
                println!("{:?}", parsed);
                println!("------------------");
            }
        }

        Ok(ast?)
    }

    fn parse_pair(pair: Pair<'_, Rule>) -> Result<ast::Node, String> {
        match pair.as_rule() {
            Rule::program | Rule::statement | Rule::expr => Self::parse_pairs(pair.into_inner()),
            Rule::statements => Ok(ast::Node::Statements(
                pair.into_inner()
                    .map(Self::parse_pair)
                    .collect::<Result<Vec<ast::Node>, String>>()?,
            )),
            Rule::list => Ok(ast::Node::List(
                pair.into_inner()
                    .map(Self::parse_pair)
                    .collect::<Result<Vec<ast::Node>, String>>()?,
            )),
            Rule::int => pair
                .as_str()
                .parse::<f64>()
                .map_err(|err| err.to_string())
                .map(ast::Node::Number),
            Rule::string => Ok(ast::Node::String(
                pair.as_str()
                    .to_string()
                    .trim_matches('"')
                    .replace("\\n", "\n")
                    .replace("\\0", "\0")
                    + "\0",
            )),
            Rule::range => {
                let mut inner = pair.into_inner();
                let lower = match inner.next().unwrap().as_str() {
                    "[" => ast::Bound::Inclusive,
                    "(" => ast::Bound::Exclusive,
                    _ => unreachable!(),
                };
                let from = Box::new(Self::number_or_var(inner.next().unwrap().as_str())?);
                let to = Box::new(Self::number_or_var(inner.next().unwrap().as_str())?);

                let upper = match inner.next().unwrap().as_str() {
                    "]" => ast::Bound::Inclusive,
                    ")" => ast::Bound::Exclusive,
                    _ => unreachable!(),
                };
                Ok(ast::Node::Range {
                    from,
                    to,
                    lower,
                    upper,
                })
            }
            Rule::varref => Ok(ast::Node::VarRef(pair.as_str().to_string())),
            Rule::looop => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let iterable = Self::parse_pair(inner.next().unwrap())?;
                let inner = Self::parse_pair(inner.next().unwrap())?;
                Ok(ast::Node::Loop {
                    var: name,
                    iterable: Box::new(iterable),
                    inner: Box::new(inner),
                })
            }
            Rule::whiile => {
                let mut inner = pair.into_inner();
                let cond = Self::parse_pair(inner.next().unwrap())?;
                let inner = Self::parse_pair(inner.next().unwrap())?;
                Ok(ast::Node::While {
                    condition: Box::new(cond),
                    inner: Box::new(inner),
                })
            }
            Rule::fundef => {
                let mut inner = pair.into_inner();
                let first = inner.next().unwrap();
                let (name, next) = match first.as_rule() {
                    Rule::name => (Some(first.as_str().to_string()), inner.next().unwrap()),
                    _ => (None, first),
                };
                let (names, inner) = match inner.next() {
                    Some(inner) => (
                        next.into_inner()
                            .map(|n| n.as_str().to_string())
                            .collect::<Vec<String>>(),
                        inner,
                    ),
                    None => (Vec::new(), next),
                };
                Ok(ast::Node::FnDef(
                    name,
                    names,
                    Box::new(Self::parse_pair(inner)?),
                ))
            }
            Rule::var => {
                let mut inner = pair.into_inner();
                let first = inner.next().unwrap();
                let (def, name) = match first.as_rule() {
                    Rule::def => (Some(first), inner.next().unwrap()),
                    Rule::name => (None, first),
                    _ => unreachable!(),
                };
                let name = name.as_str().to_string();
                let expr = Box::new(Self::parse_pair(inner.next().unwrap())?);
                let node = match def {
                    Some(def) => {
                        let mutable = if def.as_str().contains("mut") {
                            ast::Mut::Mutable
                        } else {
                            ast::Mut::Immutable
                        };
                        ast::Node::Define(mutable, name, expr)
                    }
                    None => ast::Node::Assign(name, expr),
                };
                Ok(node)
            }
            Rule::fun => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let args = inner
                    .into_iter()
                    .map(Self::parse_pair)
                    .collect::<Result<Vec<ast::Node>, String>>()?;
                Ok(ast::Node::FnCall(name, args))
            }
            Rule::iif => {
                let mut inner = pair.into_inner();
                let condition = Self::parse_pair(inner.next().unwrap())?;
                let if_block = Self::parse_pair(inner.next().unwrap())?;
                let else_block = inner
                    .next()
                    .map(|pair| Self::parse_pair(pair.into_inner().next().unwrap()))
                    .unwrap_or(Ok(ast::Node::Nada))?;
                Ok(ast::Node::IfElse {
                    condition: Box::new(condition),
                    if_block: Box::new(if_block),
                    else_block: Box::new(else_block),
                })
            }
            _ => {
                dbg!(pair);
                unreachable!()
            }
        }
    }

    fn number_or_var(str: &str) -> Result<ast::Node, String> {
        if str.chars().next().unwrap().is_numeric() {
            return str
                .parse::<f64>()
                .map(|f| ast::Node::Number(f))
                .map_err(|err| err.to_string());
        } else {
            return Ok(ast::Node::VarRef(str.into()));
        }
    }

    fn parse_pairs(pairs: Pairs<Rule>) -> Result<ast::Node, String> {
        PRATT_PARSER
            .map_primary(|primary| Self::parse_pair(primary))
            .map_prefix(|op, rhs| match op.as_rule() {
                Rule::neg => Ok(ast::Node::Expr {
                    op: ast::Op::Mul,
                    lhs: Box::new(ast::Node::Number(-1.0)),
                    rhs: Box::new(rhs?),
                }),
                Rule::name => Ok(ast::Node::Assign(op.as_str().to_string(), Box::new(rhs?))),
                _ => {
                    dbg!(op, rhs?);
                    unreachable!()
                }
            })
            .map_postfix(|lhs, op| match op.as_rule() {
                Rule::EOI => lhs,
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
                        Rule::eq => ast::Op::Eq,
                        Rule::neq => ast::Op::Neq,
                        Rule::gt => ast::Op::Gt,
                        Rule::ge => ast::Op::Ge,
                        Rule::lt => ast::Op::Lt,
                        Rule::le => ast::Op::Le,
                        _ => unreachable!(),
                    },
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                })
            })
            .parse(pairs)
    }
}
