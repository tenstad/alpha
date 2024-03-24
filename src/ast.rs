#[derive(Debug)]
pub enum Node {
    Int(u64),
    Expr {
        op: Op,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}
