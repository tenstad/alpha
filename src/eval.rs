use std::{collections::HashMap, iter::zip};

use crate::ast;

#[derive(Default)]
struct Scope<'a> {
    vars: HashMap<String, ast::Node>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    fn get(&self, key: &String) -> Option<&ast::Node> {
        self.vars.get(key).or_else(|| self.parent.and_then(|p| p.get(key)))
    }
}

#[derive(Default)]
pub struct Eval {}

impl Eval {
    pub fn run(&self, node: &ast::Node) {
        self.eval(node, &mut Scope::default(), 0);
    }

    fn eval(&self, node: &ast::Node, scope: &mut Scope, depth: usize) -> ast::Node {
        match node {
            ast::Node::Statements(nodes) => {
                let mut result = ast::Node::Nada;
                for node in nodes {
                    // let pad = " ".repeat(2*depth);
                    result = self.eval(node, scope, depth);
                    // println!("{}{:?} - {:?}", pad, result, node);
                }
                result
            }
            ast::Node::Define(_mutable, name, expr) => {
                let val = self.eval(expr, scope, depth);
                scope.vars.insert(name.clone(), val);
                ast::Node::Nada
            }
            ast::Node::Assign(name, expr) => {
                let val = self.eval(expr, scope, depth);
                scope.vars.insert(name.clone(), val);
                ast::Node::Nada
            }
            ast::Node::Bool(b) => ast::Node::Bool(*b),
            ast::Node::Number(n) => ast::Node::Number(*n),
            ast::Node::List(list) => ast::Node::List(
                list.iter()
                    .map(|n| self.eval(n, scope, depth))
                    .collect::<Vec<ast::Node>>(),
            ),
            ast::Node::Loop { var, range, inner } => {
                let range = self.eval(range, scope, depth);
                match range {
                    ast::Node::Range(from, to) => {
                        let mut result = ast::Node::Nada;
                        for i in from as i64..to as i64 {
                            scope
                                .vars
                                .insert(var.clone(), ast::Node::Number(i as f64));
                            result = self.eval(inner, scope, depth + 1);
                        }
                        result
                    }
                    _ => panic!(),
                }
            }
            ast::Node::FunDef(name, _, _) => {
                scope.vars.insert(
                    name.clone(),
                    node.clone(),
                );
                node.clone()
            }
            ast::Node::IfElse(cond, iif, eelse) => match self.eval(cond, scope, depth) {
                ast::Node::Bool(true) => self.eval(&iif, scope, depth),
                ast::Node::Bool(false) => self.eval(&eelse, scope, depth),
                _ => panic!("not a bool"),
            },
            ast::Node::Expr { op, lhs, rhs } => {
                let lhs = self.eval(lhs, scope, depth);
                let rhs = self.eval(rhs, scope, depth);

                match (op, lhs, rhs) {
                    (_, ast::Node::Number(a), ast::Node::Number(b)) => match op {
                        ast::Op::Add => ast::Node::Number(a + b),
                        ast::Op::Sub => ast::Node::Number(a - b),
                        ast::Op::Mul => ast::Node::Number(a * b),
                        ast::Op::Div => ast::Node::Number(a / b),
                        ast::Op::Eq => ast::Node::Bool(a == b),
                        ast::Op::Neq => ast::Node::Bool(a != b),
                        ast::Op::Gt => ast::Node::Bool(a > b),
                        ast::Op::Ge => ast::Node::Bool(a >= b),
                        ast::Op::Lt => ast::Node::Bool(a < b),
                        ast::Op::Le => ast::Node::Bool(a <= b),
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
                                    scope,
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
                                    scope,
                                    depth,
                                ),
                            })
                            .collect(),
                    ),
                    _ => panic!()
                }
            }
            ast::Node::Fun(name, args) => {
                let args = args
                    .iter()
                    .map(|arg| self.eval(arg, scope, depth))
                    .collect::<Vec<ast::Node>>();
                match name.as_str() {
                    "print" => {
                        println!(
                            "{}",
                            args.iter()
                                .map(|arg| format!("{:?}", arg))
                                .collect::<Vec<String>>()
                                .join(" ")
                        );
                        ast::Node::Nada
                    }
                    _ => {
                        let fndef = scope
                            .get(name)
                            .expect(format!("Undefined fn: '{}'", name).as_str());
                        let (names, inner) = match fndef {
                            ast::Node::FunDef(_, names, inner) => (names, inner),
                            _ => panic!("Not a  fn: '{}'", name),
                        };
                        let mut inner_scope = Scope {
                            vars: HashMap::new(),
                            parent: Some(scope),
                        };
                        for (name, arg) in zip(names, args) {
                            inner_scope.vars.insert(name.clone(), arg);
                        }
                        self.eval(inner, &mut inner_scope, depth + 1)
                    }
                }
            }
            ast::Node::VarRef(name) => scope
                .get(name)
                .expect(format!("Variable '{}' not known", name).as_str())
                .clone(),
            ast::Node::Range(a, b) => ast::Node::Range(*a, *b),
            ast::Node::Nada => ast::Node::Nada,
        }
    }
}
