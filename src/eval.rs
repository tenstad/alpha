use std::{collections::HashMap, iter::zip};

use crate::ast;

#[derive(Default, Debug, Clone)]
pub struct Scope {
    vars: HashMap<String, ast::Node>,
    parent: HashMap<String, ast::Node>,
}

impl Scope {
    fn get(&self, key: &String) -> Option<&ast::Node> {
        self.vars.get(key).or_else(|| self.parent.get(key))
    }

    fn combined(&self) -> HashMap<String, ast::Node> {
        let mut combined = self.parent.clone();
        combined.extend(self.vars.clone().into_iter());
        combined
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
            ast::Node::Define(_mutable, name, expr, _typename) => {
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
            ast::Node::String(s) => ast::Node::String(s.clone()),
            ast::Node::List(list) => ast::Node::List(
                list.iter()
                    .map(|n| self.eval(n, scope, depth))
                    .collect::<Vec<ast::Node>>(),
            ),
            ast::Node::While { condition, inner } => {
                let mut result = ast::Node::Nada;
                loop {
                    let cond = self.eval(condition, scope, depth);
                    match cond {
                        ast::Node::Bool(b) => {
                            if !b {
                                break;
                            }
                        }
                        _ => panic!("condition is not a bool"),
                    };
                    result = self.eval(&inner, scope, depth);
                }
                result
            }
            ast::Node::Loop {
                var,
                iterable,
                inner,
            } => {
                let iterable = self.eval(iterable, scope, depth);
                let mut results = Vec::<ast::Node>::new();
                match iterable {
                    ast::Node::Range {
                        from,
                        to,
                        lower,
                        upper,
                    } => {
                        let mut start = match from.as_ref() {
                            ast::Node::Number(n) => *n,
                            ast::Node::VarRef(_) => {
                                match self.eval(from.as_ref(), scope, depth + 1) {
                                    ast::Node::Number(n) => n,
                                    _ => panic!("Not a number: '{:?}'", node),
                                }
                            }
                            _ => panic!("unsupported range start"),
                        };

                        let mut end: f64 = match to.as_ref() {
                            ast::Node::Number(n) => *n,
                            ast::Node::VarRef(_) => {
                                match self.eval(to.as_ref(), scope, depth + 1) {
                                    ast::Node::Number(n) => n,
                                    _ => panic!("Not a number: '{:?}'", node),
                                }
                            }
                            _ => panic!("unsupported range start"),
                        };

                        start += match lower {
                            ast::Bound::Inclusive => 0.0,
                            ast::Bound::Exclusive => 1.0,
                        };
                        end += match upper {
                            ast::Bound::Inclusive => 1.0,
                            ast::Bound::Exclusive => 0.0,
                        };

                        for i in start as i64..end as i64 {
                            scope.vars.insert(var.clone(), ast::Node::Number(i as f64));
                            match self.eval(inner, scope, depth + 1) {
                                ast::Node::Nada => {}
                                node => results.push(node),
                            }
                        }
                    }
                    ast::Node::List(list) => {
                        for i in list {
                            scope.vars.insert(var.clone(), i);
                            match self.eval(inner, scope, depth + 1) {
                                ast::Node::Nada => {}
                                node => results.push(node),
                            }
                        }
                    }
                    node => panic!("Not an iterable: '{:?}'", node),
                }
                ast::Node::List(results)
            }
            ast::Node::FnDef(name, params, inner, _typename) => {
                let fn_scope = Scope {
                    vars: HashMap::new(),
                    parent: scope.combined(),
                };
                let def =
                    ast::Node::ScopedFnDef(name.clone(), params.clone(), inner.clone(), fn_scope);

                if let Some(name) = name {
                    scope.vars.insert(name.clone(), def.clone());
                }
                def
            }
            ast::Node::ScopedFnDef(_, _, _, _) => node.clone(),
            ast::Node::TypeName(_) => node.clone(),
            ast::Node::IfElse {
                condition,
                if_block,
                else_block,
            } => match self.eval(condition, scope, depth) {
                ast::Node::Bool(true) => self.eval(if_block, scope, depth),
                ast::Node::Bool(false) => self.eval(else_block, scope, depth),
                node => panic!("Not a bool: '{:?}'", node),
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
                    _ => panic!(),
                }
            }
            ast::Node::FnCall(name, args) => {
                let args = args
                    .iter()
                    .map(|arg| self.eval(arg, scope, depth))
                    .collect::<Vec<ast::Node>>();
                match name.as_str() {
                    "printf" => {
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
                            .expect(format!("Undefined function: '{}'", name).as_str());
                        let (defname, names, inner, mut fn_scope) = match fndef {
                            ast::Node::ScopedFnDef(defname, names, inner, scope) => {
                                (defname, names, inner, scope.clone())
                            }
                            _ => panic!("Not a function: '{}'", name),
                        };
                        if let Some(defname) = defname {
                            fn_scope.vars.insert(defname.clone(), fndef.clone());
                        }
                        for (name, arg) in zip(names, args) {
                            fn_scope.vars.insert(name.clone(), arg);
                        }
                        self.eval(inner, &mut fn_scope, depth + 1)
                    }
                }
            }
            ast::Node::VarRef(name) => scope
                .get(name)
                .expect(format!("Undefined variable: '{}'", name).as_str())
                .clone(),
            ast::Node::Range { .. } => node.clone(),
            ast::Node::Nada => ast::Node::Nada,
        }
    }
}
