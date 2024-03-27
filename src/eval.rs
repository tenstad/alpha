use std::collections::HashMap;

use crate::ast;

#[derive(Default)]
pub struct Eval {
    vars: HashMap<String, ast::Node>,
}

impl Eval {
    pub fn run(&mut self, nodes: &Vec<ast::Node>) {
        println!("----- Evaled -----");
        for node in nodes {
            let result = self.eval(node);
            println!("{:?} - {:?}", result, node);
        }
        println!("------------------");
    }

    fn eval(&mut self, node: &ast::Node) -> ast::Node {
        match node {
            ast::Node::Define(_mutable, name, expr) => {
                let val = self.eval(expr);
                self.vars.insert(name.clone(), val);
                ast::Node::Nada
            }
            ast::Node::Assign(name, expr) => {
                let val = self.eval(expr);
                self.vars.insert(name.clone(), val);
                ast::Node::Nada
            }
            ast::Node::Number(n) => ast::Node::Number(*n),
            ast::Node::List(list) => ast::Node::List(
                list.iter()
                    .map(|n| self.eval(n))
                    .collect::<Vec<ast::Node>>(),
            ),
            ast::Node::Expr { op, lhs, rhs } => {
                let lhs = self.eval(lhs);
                let rhs = self.eval(rhs);

                match (op, lhs, rhs) {
                    (_, ast::Node::Number(a), ast::Node::Number(b)) => match op {
                        ast::Op::Add => ast::Node::Number(a + b),
                        ast::Op::Sub => ast::Node::Number(a - b),
                        ast::Op::Mul => ast::Node::Number(a * b),
                        ast::Op::Div => ast::Node::Number(a / b),
                    },
                    (ast::Op::Add, ast::Node::List(a), ast::Node::List(b)) => {
                        ast::Node::List(a.iter().chain(b.iter()).cloned().collect())
                    }
                    (ast::Op::Mul, ast::Node::Number(a), ast::Node::List(b)) => ast::Node::List(
                        b.iter()
                            .map(|x| match x {
                                ast::Node::Number(n) => ast::Node::Number(a * n),
                                _ => self.eval(&ast::Node::Expr {
                                    op: ast::Op::Mul,
                                    lhs: Box::new(ast::Node::Number(a)),
                                    rhs: Box::new(x.clone()),
                                }),
                            })
                            .collect(),
                    ),
                    (ast::Op::Div, ast::Node::List(a), ast::Node::Number(b)) => ast::Node::List(
                        a.iter()
                            .map(|x| match x {
                                ast::Node::Number(n) => ast::Node::Number(n / b),
                                _ => self.eval(&ast::Node::Expr {
                                    op: ast::Op::Div,
                                    lhs: Box::new(x.clone()),
                                    rhs: Box::new(ast::Node::Number(b)),
                                }),
                            })
                            .collect(),
                    ),
                    _ => panic!(),
                }
            }
            ast::Node::VarRef(name) => self.vars.get(name).unwrap().clone(),
            ast::Node::Nada => ast::Node::Nada,
        }
    }
}
