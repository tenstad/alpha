#[derive(Debug)]
pub enum Node {
    Int(u64),
    List(Vec<Node>),
    Expr {
        op: Op,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Define(Mut, String, Box<Node>),
    Assign(String, Box<Node>),
    VarRef(String),
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum Mut {
    Mutable,
    Immutable
}