use sexp::*;
use sexp::Atom::*;

use crate::expressions::{Op1, Op2, Expr, ReplExpr};

fn is_keyword(s: &str) -> bool {
    matches!(s,
        "let" | "if" | "loop" | "break" | "set!" | "block" |
        "add1" | "sub1" | "isnum" | "isbool" | "define" |
        "+" | "-" | "*" | "=" | ">" | ">=" | "<" | "<=" | "true" | "false"
    )
}

pub fn parse_expr(s: &Sexp) -> std::io::Result<Expr> {
    match s {
        Sexp::Atom(I(n)) => Ok(Expr::Number(i64::try_from(*n).unwrap())),
        Sexp::Atom(S(s)) => {
            match s.as_str() {
                "true" => Ok(Expr::Boolean(true)),
                "false" => Ok(Expr::Boolean(false)),
                _ => {
                    if is_keyword(s) {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("'{}' is a keyword", s)));
                    }
                    Ok(Expr::Id(s.clone()))
                }
            }
        }
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), Sexp::List(bindings), body] if op == "let" => {
                    let mut bs = Vec::new();
                    for b in bindings {
                        match b {
                            Sexp::List(pair) => {
                                match &pair[..] {
                                    [Sexp::Atom(S(name)), e] => {
                                        if is_keyword(name) {
                                            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("'{}' is a keyword", name)));
                                        }
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
                [Sexp::Atom(S(op)), e1, e2, e3] if op == "if" => Ok(Expr::If(Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?), Box::new(parse_expr(e3)?))),


                [Sexp::Atom(S(op)), e] if op == "loop" => Ok(Expr::Loop(Box::new(parse_expr(e)?))),

                [Sexp::Atom(S(op)), e] if op == "break" => Ok(Expr::Break(Box::new(parse_expr(e)?))),

                [Sexp::Atom(S(op)), Sexp::Atom(S(s)), e] if op == "set!" => {
                    if is_keyword(s) {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("'{}' is a keyword", s)));
                    }
                    Ok(Expr::Set(s.clone(), Box::new(parse_expr(e)?)))
                },

                [Sexp::Atom(S(op)), rest @ ..] if op == "block" => {
                    let mut bs = Vec::new();
                    for b in rest {
                        let parsed = parse_expr(b)?;
                        bs.push(parsed);
                    }
                    Ok(Expr::Block(bs))
                }

                [Sexp::Atom(S(op)), e] if op == "add1" => Ok(Expr::UnOp(Op1::Add1, Box::new(parse_expr(e)?))),
                [Sexp::Atom(S(op)), e] if op == "sub1" => Ok(Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e)?))),
                [Sexp::Atom(S(op)), e] if op == "isnum" => Ok(Expr::UnOp(Op1::IsNum, Box::new(parse_expr(e)?))),
                [Sexp::Atom(S(op)), e] if op == "isbool" => Ok(Expr::UnOp(Op1::IsBool, Box::new(parse_expr(e)?))),

                [Sexp::Atom(S(op)), e1, e2] if op == "+" => Ok(Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "-" => Ok(Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "*" => Ok(Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "=" => Ok(Expr::BinOp(Op2::Equal, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == ">" => Ok(Expr::BinOp(Op2::Greater, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == ">=" => Ok(Expr::BinOp(Op2::GreaterEqual, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "<" => Ok(Expr::BinOp(Op2::Less, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "<=" => Ok(Expr::BinOp(Op2::LessEqual, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),


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