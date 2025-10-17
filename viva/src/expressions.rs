#[derive(Debug)]
pub enum Op1 {
    Add1,
    Sub1
}

#[derive(Debug)]
pub enum Op2 {
    Plus,
    Minus,
    Times
}

#[derive(Debug)]
pub enum Expr {
    Number(i32),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum ReplExpr {
    Define(String, Box<Expr>),
    Expr(Box<Expr>),
}