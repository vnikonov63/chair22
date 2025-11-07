#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Op1 {
    Add1,
    Sub1,
    IsNum,
    IsBool,
    Print
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Op2 {
    Plus,
    Minus,
    Times,
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Expr {
    Number(i64),
    Boolean(bool),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Loop(Box<Expr>),
    Break(Box<Expr>),
    Set(String, Box<Expr>),
    Block(Vec<Expr>),
    Call(String, Vec<Expr>)
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Defenition {
    Fun(String, Vec<String>, Box<Expr>)
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum ReplExpr {
    Define(String, Box<Expr>),
    Expr(Box<Expr>),
    Fun(String, Vec<String>, Box<Expr>)
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Program {
    pub defs: Vec<Defenition>,
    pub main: Expr
}