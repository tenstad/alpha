#[derive(Debug, Clone)]
pub enum Node {
    Nada,
    Number(f64),
    Range(f64, f64),
    Loop {
        var: String,
        range: Box<Node>,
        inner: Vec<Node>,
    },
    Statements(Vec<Node>),
    Fun(String, Vec<Node>),
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

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Mut {
    Mutable,
    Immutable,
}
