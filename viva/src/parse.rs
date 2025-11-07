use sexp::*;
use sexp::Atom::*;
use std::collections::HashSet;

use crate::expressions::{Op1, Op2, Expr, ReplExpr, Program, Defenition};

fn parse_fun_header(params: &[Sexp]) -> std::io::Result<(String, Vec<String>)> {
    if params.is_empty() {
        return parse_err("there is no function name");
    }

    let fname = match &params[0] {
        Sexp::Atom(S(name)) => {
            if is_keyword(name) {
                return parse_err(&format!(
                    "'{}' is a keyword, and it can't be a function name",
                    name
                ));
            }
            name.clone()
        }
        _ => return parse_err("function name is in the wrong format"),
    };

    let mut seen: HashSet<String> = HashSet::new();
    let mut ps: Vec<String> = Vec::new();
    for param in &params[1..] {
        match param {
            Sexp::Atom(S(name)) => {
                if is_keyword(name) {
                    return parse_err(&format!(
                        "'{}' is a keyword, and it can't be the name of a parameter",
                        name
                    ));
                }
                if seen.contains(name) {
                    return parse_err("Duplicate parameter name");
                }
                seen.insert(name.clone());
                ps.push(name.clone());
            }
            _ => return parse_err("parameter name should be a String"),
        }
    }

    Ok((fname, ps))
}

pub fn parse_prog(s: &Sexp) -> std::io::Result<Program> {
    match s {
        Sexp::List(items) => {
            if items.is_empty() {
                return parse_err("empty program");
            }

            if items.len() == 1 {
                return Ok(Program { defs: vec![], main: parse_expr(&items[0], HashSet::new())? });
            }

            let (last, rest) = items.split_last().unwrap();

            /* First pass: collect all function names to allow mutual and self recursion */
            let mut def_names: HashSet<String> = HashSet::new();
            for item in rest {
                match item {
                    Sexp::List(vec) => match &vec[..] {
                        [Sexp::Atom(S(op)), Sexp::List(params), _body] if op == "fun" => {
                            let (name, _ps) = parse_fun_header(params)?;
                            if !def_names.insert(name) {
                                return parse_err("Duplicate function name");
                            }
                        }
                        _ => return parse_err("one of the function definitions is wrong"),
                    },
                    _ => return parse_err("one of the function definitions is wrong"),
                }
            }

            /* Second pass: parse full function definitions with knowledge of all names */
            let mut defs: Vec<Defenition> = Vec::new();
            for item in rest {
                match parse_fun_def(item, def_names.clone())? {
                    Some(d) => defs.push(d),
                    None => return parse_err("one of the function definitions is wrong"),
                }
            }

            Ok(Program { defs, main: parse_expr(last, def_names)? })
        }
        _ => Ok(Program { defs: vec![], main: parse_expr(s, HashSet::new())? }),
    }
}

fn parse_fun_def(item: &Sexp, def_names: HashSet<String>) -> std::io::Result<Option<Defenition>> {
    match item {
        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(op)), Sexp::List(params), body] if op == "fun" => {
                let (fname, ps) = parse_fun_header(params)?;
                let body_expr = parse_expr(body, def_names.clone())?;
                Ok(Some(Defenition::Fun(fname, ps, Box::new(body_expr))))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

pub fn parse_expr(s: &Sexp, def_names: HashSet<String>) -> std::io::Result<Expr> {
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
                                        let parsed = parse_expr(e, def_names.clone())?;
                                        let pair = (name.clone(), parsed);
                                        bs.push(pair);
                                    }
                                    _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid: parse error")),
                                }
                            }
                            _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid: parse error")),
                        }
                    }
                    Ok(Expr::Let(bs, Box::new(parse_expr(body, def_names.clone())?)))
                }
                [Sexp::Atom(S(op)), e1, e2, e3] if op == "if" => Ok(Expr::If(Box::new(parse_expr(e1, def_names.clone())?), Box::new(parse_expr(e2, def_names.clone())?), Box::new(parse_expr(e3, def_names.clone())?))),

                [Sexp::Atom(S(op)), e] if op == "loop" => Ok(Expr::Loop(Box::new(parse_expr(e, def_names.clone())?))),

                [Sexp::Atom(S(op)), e] if op == "break" => Ok(Expr::Break(Box::new(parse_expr(e, def_names.clone())?))),

                
                [Sexp::Atom(S(op)), Sexp::Atom(S(s)), e] if op == "set!" => {
                    if is_keyword(s) {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("'{}' is a keyword", s)));
                    }
                    Ok(Expr::Set(s.clone(), Box::new(parse_expr(e, def_names.clone())?)))
                },
                
                [Sexp::Atom(S(op)), rest @ ..] if op == "block" => {
                    let mut bs = Vec::new();
                    for b in rest {
                        let parsed = parse_expr(b, def_names.clone())?;
                        bs.push(parsed);
                    }
                    Ok(Expr::Block(bs))
                }

                [Sexp::Atom(S(name)), args @ ..] if !is_keyword(name) && def_names.contains(name) => {
                    let mut parsed_args = Vec::new();
                    for a in args {
                        parsed_args.push(parse_expr(a, def_names.clone())?);
                    }
                    Ok(Expr::Call(name.clone(), parsed_args))
                },

                [Sexp::Atom(S(op)), e] if op == "add1" => Ok(Expr::UnOp(Op1::Add1, Box::new(parse_expr(e, def_names.clone())?))),
                [Sexp::Atom(S(op)), e] if op == "sub1" => Ok(Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e, def_names.clone())?))),
                [Sexp::Atom(S(op)), e] if op == "isnum" => Ok(Expr::UnOp(Op1::IsNum, Box::new(parse_expr(e, def_names.clone())?))),
                [Sexp::Atom(S(op)), e] if op == "isbool" => Ok(Expr::UnOp(Op1::IsBool, Box::new(parse_expr(e, def_names.clone())?))),
                [Sexp::Atom(S(op)), e] if op == "print" => Ok(Expr::UnOp(Op1::Print, Box::new(parse_expr(e, def_names.clone())?))),

                [Sexp::Atom(S(op)), e1, e2] if op == "+" => Ok(Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1, def_names.clone())?), Box::new(parse_expr(e2, def_names.clone())?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "-" => Ok(Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1, def_names.clone())?), Box::new(parse_expr(e2, def_names.clone())?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "*" => Ok(Expr::BinOp(Op2::Times, Box::new(parse_expr(e1, def_names.clone())?), Box::new(parse_expr(e2, def_names.clone())?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "=" => Ok(Expr::BinOp(Op2::Equal, Box::new(parse_expr(e1, def_names.clone())?), Box::new(parse_expr(e2, def_names.clone())?))),
                [Sexp::Atom(S(op)), e1, e2] if op == ">" => Ok(Expr::BinOp(Op2::Greater, Box::new(parse_expr(e1, def_names.clone())?), Box::new(parse_expr(e2, def_names.clone())?))),
                [Sexp::Atom(S(op)), e1, e2] if op == ">=" => Ok(Expr::BinOp(Op2::GreaterEqual, Box::new(parse_expr(e1, def_names.clone())?), Box::new(parse_expr(e2, def_names.clone())?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "<" => Ok(Expr::BinOp(Op2::Less, Box::new(parse_expr(e1, def_names.clone())?), Box::new(parse_expr(e2, def_names.clone())?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "<=" => Ok(Expr::BinOp(Op2::LessEqual, Box::new(parse_expr(e1, def_names.clone())?), Box::new(parse_expr(e2, def_names.clone())?))),


                _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid: parse error")),
            }
        },
        _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid: parse error")),
    }
}

pub fn parse_repl_expr(s: &Sexp, def_names: &HashSet<String>) -> std::io::Result<ReplExpr> {
    match s {
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), Sexp::Atom(S(v)), e] if op == "define" => Ok(ReplExpr::Define(v.clone(), Box::new(parse_expr(e, HashSet::new())?))),
                [Sexp::Atom(S(op)), Sexp::List(params), body] if op == "fun" => {
                    let (fname, ps) = parse_fun_header(params)?;
                    if def_names.contains(&fname) {
                        return parse_err("Duplicate function name");
                    } 
                    let body_expr = parse_expr(body, def_names.clone())?;
                    Ok(ReplExpr::Fun(fname, ps, Box::new(body_expr)))
                }
                _ => Ok(ReplExpr::Expr(Box::new(parse_expr(s, def_names.clone())?))),
            }
        }
        _ => Ok(ReplExpr::Expr(Box::new(parse_expr(s, def_names.clone())?))),
    }
}

fn is_keyword(s: &str) -> bool {
    matches!(s,
        "let" | "if" | "loop" | "break" | "set!" | "block" |
        "add1" | "sub1" | "isnum" | "isbool" | "print" | "define" | "fun" |
        "+" | "-" | "*" | "=" | ">" | ">=" | "<" | "<=" | "true" | "false"
    )
}

fn parse_err<T>(name: &str) -> std::io::Result<T> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Invalid: parse error: {}.", name),
    ))
}