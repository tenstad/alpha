#[derive(Debug)]
pub enum Node {
    Int(u64),
    List(Vec<Node>),
    Expr {
        op: Op,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Var(String, Box<Node>),
    VarRef(String),
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}
