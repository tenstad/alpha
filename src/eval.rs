use std::{borrow::Borrow, collections::HashMap};

use crate::ast;

#[derive(Default)]
pub struct Eval {
    vars: HashMap<String, ast::Node>,
}

impl Eval {
    pub fn run(&mut self, node: &ast::Node, depth: usize) {
        // let pad = " ".repeat(2*depth);
        // println!("{}----- Evaled -----", pad);
        let _result = self.eval(node, depth);
        // println!("{}{:?} - {:?}", pad, result, node);
        // println!("{}------------------", pad);
    }

    fn eval(&mut self, node: &ast::Node, depth: usize) -> ast::Node {
        match node {
            ast::Node::Statements(nodes) => {
                for node in nodes {
                    // let pad = " ".repeat(2*depth);
                    let _result = self.eval(node, depth);
                    // println!("{}{:?} - {:?}", pad, result, node);
                }
                ast::Node::Nada
            }
            ast::Node::Define(_mutable, name, expr) => {
                let val = self.eval(expr, depth);
                self.vars.insert(name.clone(), val);
                ast::Node::Nada
            }
            ast::Node::Assign(name, expr) => {
                let val = self.eval(expr, depth);
                self.vars.insert(name.clone(), val);
                ast::Node::Nada
            }
            ast::Node::Number(n) => ast::Node::Number(*n),
            ast::Node::List(list) => ast::Node::List(
                list.iter()
                    .map(|n| self.eval(n, depth))
                    .collect::<Vec<ast::Node>>(),
            ),
            ast::Node::Loop { var, range, inner } => match range.borrow() {
                ast::Node::VarRef(name) => self.eval(
                    &ast::Node::Loop {
                        var: var.clone(),
                        range: Box::new(self.vars.get(name).unwrap().clone()),
                        inner: inner.clone(),
                    },
                    depth,
                ),
                ast::Node::Range(from, to) => {
                    for i in *from as i64..*to as i64 {
                        self.vars.insert(var.clone(), ast::Node::Number(i as f64));
                        self.run(&ast::Node::Statements(inner.clone()), depth + 1);
                    }
                    ast::Node::Nada
                }
                _ => panic!(),
            },
            ast::Node::Expr { op, lhs, rhs } => {
                let lhs = self.eval(lhs, depth);
                let rhs = self.eval(rhs, depth);

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
                                _ => self.eval(
                                    &ast::Node::Expr {
                                        op: ast::Op::Mul,
                                        lhs: Box::new(ast::Node::Number(a)),
                                        rhs: Box::new(x.clone()),
                                    },
                                    depth,
                                ),
                            })
                            .collect(),
                    ),
                    (ast::Op::Div, ast::Node::List(a), ast::Node::Number(b)) => ast::Node::List(
                        a.iter()
                            .map(|x| match x {
                                ast::Node::Number(n) => ast::Node::Number(n / b),
                                _ => self.eval(
                                    &ast::Node::Expr {
                                        op: ast::Op::Div,
                                        lhs: Box::new(x.clone()),
                                        rhs: Box::new(ast::Node::Number(b)),
                                    },
                                    depth,
                                ),
                            })
                            .collect(),
                    ),
                    _ => panic!(),
                }
            }
            ast::Node::Fun(name, args) => {
                if name == "print" {
                    println!("{:?}", self.eval(args, depth));
                }
                ast::Node::Nada
            }
            ast::Node::VarRef(name) => self
                .vars
                .get(name)
                .expect(format!("Variable '{}' not known", name).as_str())
                .clone(),
            ast::Node::Range(a, b) => ast::Node::Range(*a, *b),
            ast::Node::Nada => ast::Node::Nada,
        }
    }
}
