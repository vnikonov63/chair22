use sexp::*;
use sexp::Atom::*;

use crate::expressions::{Op1, Op2, Expr, ReplExpr};

pub fn parse_expr(s: &Sexp) -> std::io::Result<Expr> {
    match s {
        Sexp::Atom(I(n)) => Ok(Expr::Number(i32::try_from(*n).unwrap())),
        Sexp::Atom(S(s)) => Ok(Expr::Id(s.clone())),
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), Sexp::List(bindings), body] if op == "let" => {
                    let mut bs = Vec::new();
                    for b in bindings {
                        match b {
                            Sexp::List(pair) => {
                                match &pair[..] {
                                    [Sexp::Atom(S(name)), e] => {
                                        let parsed = parse_expr(e)?;
                                        let pair = (name.clone(), parsed);
                                        bs.push(pair);
                                    }
                                    _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid: parse error")),
                                }
                            }
                            _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid: parse error")),
                        }
                    }
                    Ok(Expr::Let(bs, Box::new(parse_expr(body)?)))
                }
                [Sexp::Atom(S(op)), e] if op == "add1" => Ok(Expr::UnOp(Op1::Add1, Box::new(parse_expr(e)?))),
                [Sexp::Atom(S(op)), e] if op == "sub1" => Ok(Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e)?))),

                [Sexp::Atom(S(op)), e1, e2] if op == "+" => Ok(Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "-" => Ok(Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "*" => Ok(Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),

                _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid: parse error")),
            }
        },
        _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid: parse error")),
    }
}

pub fn parse_repl_expr(s: &Sexp) -> std::io::Result<ReplExpr> {
    match s {
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), Sexp::Atom(S(v)), e] if op == "define" => Ok(ReplExpr::Define(v.clone(), Box::new(parse_expr(e)?))),

                _ => Ok(ReplExpr::Expr(Box::new(parse_expr(s)?))),
            }
        }
        _ => Ok(ReplExpr::Expr(Box::new(parse_expr(s)?))),
    }
}