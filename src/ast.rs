use crate::eval;

#[derive(Debug, Clone)]
pub enum Node {
    Nada,
    Number(f64),
    Bool(bool),
    Range(f64, f64),
    Loop {
        var: String,
        range: Box<Node>,
        inner: Box<Node>,
    },
    IfElse(Box<Node>, Box<Node>, Box<Node>),
    Statements(Vec<Node>),
    Fun(String, Vec<Node>),
    FunDef(Option<String>, Vec<String>, Box<Node>),
    ScopedFunDef(Option<String>, Vec<String>, Box<Node>, eval::Scope),
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
    Eq,
    Neq,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Debug, Clone)]
pub enum Mut {
    Mutable,
    Immutable,
}
