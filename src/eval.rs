use std::collections::HashMap;

use crate::ast;

#[derive(Default)]
pub struct Eval {
    vars: HashMap<String, f64>,
}

impl Eval {
    pub fn run(&mut self, nodes: &Vec<ast::Node>) {
        for node in nodes {
            let result = self.eval(node);
            println!("Result: {:?}", result);
        }
    }

    pub fn eval(&mut self, node: &ast::Node) -> f64 {
        match node {
            ast::Node::Var(name, expr) => {
                let val = self.eval(expr);
                self.vars.insert(name.clone(), val);
                0.0
            }
            ast::Node::Int(n) => *n as f64,
            ast::Node::List(list) => list.iter().map(|n| self.eval(n)).sum(),
            ast::Node::Expr { op, lhs, rhs } => {
                let lhs = self.eval(lhs);
                let rhs = self.eval(rhs);

                match op {
                    ast::Op::Add => lhs + rhs,
                    ast::Op::Sub => lhs - rhs,
                    ast::Op::Mul => lhs * rhs,
                    ast::Op::Div => lhs / rhs,
                }
            }
            ast::Node::VarRef(name) => *self.vars.get(name).unwrap(),
        }
    }
}
