use crate::eval;

#[derive(Debug, Clone)]
pub enum Node {
    Nada,
    Number(f64),
    Bool(bool),
    String(String),
    Range {
        from: Box<Node>,
        to: Box<Node>,
        lower: Bound,
        upper: Bound,
    },
    Loop {
        var: String,
        iterable: Box<Node>,
        inner: Box<Node>,
    },
    While {
        condition: Box<Node>,
        inner: Box<Node>,
    },
    IfElse {
        condition: Box<Node>,
        if_block: Box<Node>,
        else_block: Box<Node>,
    },
    Statements(Vec<Node>),
    FnCall(String, Vec<Node>),
    FnDef(Option<String>, Vec<String>, Box<Node>),
    ScopedFnDef(Option<String>, Vec<String>, Box<Node>, eval::Scope),
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
pub enum Bound {
    Inclusive,
    Exclusive,
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
