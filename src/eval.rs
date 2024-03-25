use crate::ast;

pub struct Eval;

impl Eval {
    pub fn eval(node: &ast::Node) -> f64 {
        match node {
            ast::Node::Int(n) => *n as f64,
            ast::Node::List(list) => list.iter().map(Self::eval).sum(),
            ast::Node::Expr { op, lhs, rhs } => {
                let lhs = Self::eval(lhs);
                let rhs = Self::eval(rhs);

                match op {
                    ast::Op::Add => lhs + rhs,
                    ast::Op::Sub => lhs - rhs,
                    ast::Op::Mul => lhs * rhs,
                    ast::Op::Div => lhs / rhs,
                }
            }
        }
    }
}
